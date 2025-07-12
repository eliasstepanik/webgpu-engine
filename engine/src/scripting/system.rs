//! Script execution system

use crate::core::entity::{Entity, World};
use crate::scripting::commands::{CommandQueue, SharedComponentCache};
use crate::scripting::component_access::populate_cache_for_scripts;
use crate::scripting::modules::world::{create_world_module, register_material_type};
use crate::scripting::{ScriptEngine, ScriptInputState, ScriptRef};
use rhai::{Dynamic, Module, Scope};
use std::collections::HashSet;
use tracing::{debug, error, info, trace, warn};

/// Tracks script lifecycle state for entities
#[derive(Default)]
pub struct ScriptLifecycleTracker {
    /// Entities that have had on_start called
    started_entities: HashSet<Entity>,
    /// Entities that need on_destroy called
    active_entities: HashSet<Entity>,
}

/// System to execute scripts on entities
pub fn script_execution_system(
    world: &mut World,
    script_engine: &mut ScriptEngine,
    input_state: &ScriptInputState,
    delta_time: f32,
) {
    // Use thread-local storage for the tracker to avoid unsafe mutable statics
    thread_local! {
        static TRACKER: std::cell::RefCell<ScriptLifecycleTracker> = std::cell::RefCell::new(ScriptLifecycleTracker::default());
        static COMMAND_QUEUE: CommandQueue = CommandQueue::default();
        static COMPONENT_CACHE: SharedComponentCache = SharedComponentCache::default();
    }

    TRACKER.with(|tracker_cell| {
        let mut tracker = tracker_cell.borrow_mut();

        // Get command queue and component cache
        let command_queue = COMMAND_QUEUE.with(|q| q.clone());
        let component_cache = COMPONENT_CACHE.with(|c| c.clone());

        // Clear command queue from previous frame
        command_queue.write().unwrap().clear();

        // Populate component cache with current world state
        {
            let mut cache = component_cache.write().unwrap();
            populate_cache_for_scripts(world.inner(), &mut cache);
        }

        // Collect entities with scripts first to avoid borrow conflicts
        let mut entities_with_scripts = Vec::new();
        for (entity, script_ref) in world.query::<&ScriptRef>().iter() {
            entities_with_scripts.push((entity, script_ref.clone()));
        }

        debug!(
            count = entities_with_scripts.len(),
            "Executing scripts on entities"
        );

        // Process each entity with a script
        for (entity, script_ref) in entities_with_scripts {
            // Ensure script is loaded
            if !script_engine.is_loaded(&script_ref.name) {
                match script_engine.load_script_by_name(&script_ref.name) {
                    Ok(_) => {
                        debug!(script = script_ref.name, "Loaded script successfully");
                    }
                    Err(e) => {
                        error!(script = script_ref.name, error = %e, "Failed to load script");
                        continue;
                    }
                }
            }

            // Create a scope for this entity
            let mut scope = Scope::new();

            // Add entity ID to scope
            let entity_id = entity.to_bits().get() as i64;
            scope.push("entity", entity_id);

            // Create world module with command queue and cache
            let world_module = create_world_module(command_queue.clone(), component_cache.clone());

            // Create input module with current state
            let input_module = create_input_module(input_state);

            // Register modules in the engine temporarily
            // We need mutable access to the engine to register modules
            if let Some(engine) = script_engine.engine_mut() {
                engine.register_static_module("world", world_module.into());
                engine.register_static_module("input", input_module.into());
            } else {
                // If we can't get mutable access, skip this entity
                warn!(entity = ?entity, "Cannot get mutable access to script engine");
                continue;
            }

            // Check if this is a new entity that needs on_start
            if !tracker.started_entities.contains(&entity) {
                trace!(entity = ?entity, script = script_ref.name, "Calling on_start");

                match script_engine.call_on_start(&script_ref.name, entity.to_bits().get(), &mut scope)
                {
                    Ok(_) => {
                        tracker.started_entities.insert(entity);
                        tracker.active_entities.insert(entity);
                    }
                    Err(e) => {
                        warn!(entity = ?entity, script = script_ref.name, error = %e, "Script on_start failed");
                    }
                }
            }

            // Call on_update
            trace!(entity = ?entity, script = script_ref.name, delta_time = delta_time, "Calling on_update");

            if let Err(e) = script_engine.call_on_update(
                &script_ref.name,
                entity.to_bits().get(),
                &mut scope,
                delta_time,
            ) {
                warn!(entity = ?entity, script = script_ref.name, error = %e, "Script on_update failed");
            }
        }

        // Apply all queued commands after all scripts have run
        let commands = command_queue.write().unwrap().drain(..).collect::<Vec<_>>();
        if !commands.is_empty() {
            info!(count = commands.len(), "Applying script commands");
            for command in commands {
                if let Err(e) = command.apply(world.inner_mut()) {
                    error!(error = %e, "Failed to apply script command");
                }
            }
        }

        // Clear component cache to prevent stale data
        component_cache.write().unwrap().clear();

        // Check for entities that were destroyed and need on_destroy
        let mut destroyed_entities = Vec::new();
        for entity in &tracker.active_entities {
            if world.get::<&ScriptRef>(*entity).is_err() {
                destroyed_entities.push(*entity);
            }
        }

        for entity in destroyed_entities {
            trace!(entity = ?entity, "Entity destroyed, calling on_destroy");

            // We can't get the script ref anymore, so we'll skip on_destroy for now
            // In a real implementation, we'd track this separately
            tracker.started_entities.remove(&entity);
            tracker.active_entities.remove(&entity);
        }
    }); // End of TRACKER.with closure
}

/// Create an input module with current input state
fn create_input_module(input_state: &ScriptInputState) -> Module {
    let mut module = Module::new();

    // Clone the state for the closures
    let keys = input_state.keys_pressed.clone();
    let mouse_pos = input_state.mouse_position;
    let mouse_delta = input_state.mouse_delta;
    let mouse_buttons = input_state.mouse_buttons.clone();

    // Register functions using FnMut closures
    let keys_clone = keys.clone();
    module.set_native_fn("is_key_pressed", move |key: &str| {
        Ok(keys_clone.contains(key))
    });

    module.set_native_fn("mouse_position", move || {
        Ok(vec![
            Dynamic::from(mouse_pos.0 as f64),
            Dynamic::from(mouse_pos.1 as f64),
        ])
    });

    module.set_native_fn("mouse_delta", move || {
        Ok(vec![
            Dynamic::from(mouse_delta.0 as f64),
            Dynamic::from(mouse_delta.1 as f64),
        ])
    });

    let buttons_clone = mouse_buttons.clone();
    module.set_native_fn("is_mouse_button_pressed", move |button: i64| {
        Ok(buttons_clone.contains(&(button as u8)))
    });

    module
}

/// Initialize the script engine with all necessary types and modules
pub fn initialize_script_engine(script_engine: &mut ScriptEngine) {
    use crate::scripting::modules;

    debug!("Initializing script engine");

    // Get mutable access to the engine
    if let Some(engine) = script_engine.engine_mut() {
        // Register all modules
        modules::register_all_modules(engine);

        // Register additional types
        register_material_type(engine);

        debug!("Script engine initialized");
    } else {
        error!("Cannot get mutable access to engine - are there other references?");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::Name;

    #[test]
    fn test_script_execution_basic() {
        let mut world = World::new();
        let mut script_engine = ScriptEngine::new();
        let input_state = ScriptInputState::new();

        // Initialize the engine
        initialize_script_engine(&mut script_engine);

        // Create a test entity with a script
        let entity = world.spawn((Name::new("Test Entity"), ScriptRef::new("test_script")));

        // This would normally load an actual script file
        // For testing, we'd need to mock this

        // Execute the system (it should handle missing scripts gracefully)
        script_execution_system(&mut world, &mut script_engine, &input_state, 0.016);

        // Entity should still exist
        assert!(world.contains(entity));
    }

    #[test]
    fn test_input_module_creation() {
        let mut input_state = ScriptInputState::new();
        input_state.keys_pressed.insert("W".to_string());
        input_state.mouse_position = (100.0, 200.0);

        let module = create_input_module(&input_state);

        // Test that the module has the expected functions
        // Note: We can't easily test the actual function calls without a full Rhai engine
        assert!(!module.is_empty());
    }
}
