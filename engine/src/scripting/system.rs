//! Script execution system

use crate::core::entity::World;
use crate::scripting::commands::{CommandQueue, ScriptCommand, SharedComponentCache};
use crate::scripting::component_access::populate_cache_for_scripts;
use crate::scripting::lifecycle_tracker::get_tracker;
use crate::scripting::modules::mesh::create_mesh_module;
use crate::scripting::modules::world::{create_world_module, register_material_type};
use crate::scripting::property_types::{PropertyType, PropertyValue, ScriptProperties};
use crate::scripting::{ScriptEngine, ScriptInputState, ScriptRef};
use rhai::{Dynamic, Module, Scope};
use tracing::{debug, error, trace, warn};

// Import profiling macro
use crate::profile_zone;

/// System to execute scripts on entities
pub fn script_execution_system(
    world: &mut World,
    script_engine: &mut ScriptEngine,
    input_state: &ScriptInputState,
    delta_time: f32,
) {
    profile_zone!("ScriptSystem::update");
    // Use thread-local storage for command queue and cache
    thread_local! {
        static COMMAND_QUEUE: CommandQueue = CommandQueue::default();
        static COMPONENT_CACHE: SharedComponentCache = SharedComponentCache::default();
    }

    // Log initial tracker state
    {
        let tracker = get_tracker().lock().unwrap();
        debug!(
            "Script execution system starting. Tracker state: {} started entities, {} active entities, counter: {}",
            tracker.started_count(),
            tracker.active_entities.len(),
            tracker.debug_counter
        );
    }

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

    // Collect entities with scripts first using compound query to avoid borrow conflicts
    let mut entities_with_scripts = Vec::new();
    for (entity, (script_ref, properties)) in world
        .query::<(&ScriptRef, Option<&ScriptProperties>)>()
        .iter()
    {
        trace!(
            entity = ?entity,
            entity_id = entity.to_bits().get(),
            script = script_ref.name,
            has_properties = properties.is_some(),
            "Found entity with script"
        );
        entities_with_scripts.push((entity, script_ref.clone(), properties.cloned()));
    }

    debug!(
        count = entities_with_scripts.len(),
        "Executing scripts on entities"
    );

    // Process each entity with a script
    for (entity, script_ref, script_properties) in entities_with_scripts {
        trace!(
            "Processing entity {:?} (ID: {}) with script {}",
            entity,
            entity.to_bits().get(),
            script_ref.name
        );
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

        // Add properties to scope if available
        if let Some(ref properties) = script_properties {
            let props_map = properties.to_rhai_map();
            debug!(
                entity = ?entity,
                script = script_ref.name,
                script_name_in_props = ?properties.script_name,
                properties = ?properties.values,
                "🎯 Using script properties for execution"
            );
            scope.push("properties", props_map);
        } else {
            // If no properties component, check if script defines properties
            // and create default values
            if let Some(definitions) = script_engine.get_property_definitions(&script_ref.name) {
                if !definitions.is_empty() {
                    debug!(
                        entity = ?entity,
                        script = script_ref.name,
                        property_count = definitions.len(),
                        "Script has property definitions but entity has no ScriptProperties component"
                    );
                    // Create empty properties map for scripts that expect properties
                    let empty_props = ScriptProperties::from_definitions(&definitions);
                    scope.push("properties", empty_props.to_rhai_map());
                }
            }
        }

        // Create world module with command queue and cache
        let world_module = create_world_module(command_queue.clone(), component_cache.clone());

        // Create input module with current state
        let input_module = create_input_module(input_state);

        // Create mesh module with mesh registry and command queue
        let mesh_module =
            create_mesh_module(script_engine.mesh_registry.clone(), command_queue.clone());

        // Create physics module
        let physics_module = crate::scripting::modules::physics::create_physics_module();

        // Create profiling module
        let profiling_module = crate::scripting::modules::profiling::create_profiling_module();

        // Register modules in the engine temporarily
        // We need mutable access to the engine to register modules
        if let Some(engine) = script_engine.engine_mut() {
            engine.register_static_module("world", world_module.into());
            engine.register_static_module("input", input_module.into());
            engine.register_static_module("Mesh", mesh_module.into());
            engine.register_static_module("physics", physics_module.into());
            engine.register_static_module("profiling", profiling_module.into());
        } else {
            // If we can't get mutable access, skip this entity
            warn!(entity = ?entity, "Cannot get mutable access to script engine");
            continue;
        }

        // Check if this is a new entity that needs on_start
        let needs_start = {
            let tracker = get_tracker().lock().unwrap();
            !tracker.has_started(entity)
        };

        if needs_start {
            {
                let tracker = get_tracker().lock().unwrap();
                warn!(
                    "🔄 Entity {:?} not in started_entities (size: {}). Calling on_start for script: {}",
                    entity,
                    tracker.started_count(),
                    script_ref.name
                );
            }

            {
                profile_zone!("Script::on_start");
                match script_engine.call_on_start(
                    &script_ref.name,
                    entity.to_bits().get(),
                    &mut scope,
                ) {
                    Ok(_) => match get_tracker().lock() {
                        Ok(mut tracker) => {
                            tracker.mark_started(entity);
                            debug!(
                                "✅ Marked entity {:?} as started. Total started: {}",
                                entity,
                                tracker.started_count()
                            );
                        }
                        Err(e) => {
                            error!("Failed to lock tracker mutex: {}", e);
                        }
                    },
                    Err(e) => {
                        warn!(entity = ?entity, script = script_ref.name, error = %e, "Script on_start failed");
                    }
                }
            }
        }

        // Call on_update
        trace!(entity = ?entity, script = script_ref.name, delta_time = delta_time, "Calling on_update");

        {
            profile_zone!("Script::on_update");
            if let Err(e) = script_engine.call_on_update(
                &script_ref.name,
                entity.to_bits().get(),
                &mut scope,
                delta_time,
            ) {
                warn!(entity = ?entity, script = script_ref.name, error = %e, "Script on_update failed");
            }
        }

        // Check if properties were modified and persist changes
        if let Some(ref original_properties) = script_properties {
            // Try to get modified properties from scope
            if let Some(modified_props) = scope.get_value::<rhai::Map>("properties") {
                let mut changed = false;
                let mut updated_properties: ScriptProperties = original_properties.clone();

                // Check each property for changes
                for (name, original_value) in &original_properties.values {
                    if let Some(new_dynamic) = modified_props.get(name.as_str()) {
                        // Determine the expected type from the original value
                        let prop_type = match original_value {
                            PropertyValue::Float(_) => PropertyType::Float,
                            PropertyValue::Integer(_) => PropertyType::Integer,
                            PropertyValue::Boolean(_) => PropertyType::Boolean,
                            PropertyValue::String(_) => PropertyType::String,
                            PropertyValue::Vector3(_) => PropertyType::Vector3,
                            PropertyValue::Color(_) => PropertyType::Color,
                        };

                        // Try to convert back to PropertyValue
                        if let Some(new_value) = PropertyValue::from_dynamic(new_dynamic, prop_type)
                        {
                            if &new_value != original_value {
                                updated_properties.values.insert(name.clone(), new_value);
                                changed = true;
                                trace!(
                                    entity = ?entity,
                                    property = name,
                                    "Property value changed"
                                );
                            }
                        } else {
                            warn!(
                                entity = ?entity,
                                property = name,
                                expected_type = ?prop_type,
                                "Failed to convert property value from dynamic"
                            );
                        }
                    }
                }

                // Queue update command if properties changed
                if changed {
                    command_queue
                        .write()
                        .unwrap()
                        .push(ScriptCommand::SetProperties {
                            entity: entity.to_bits().get(),
                            properties: updated_properties,
                        });
                    debug!(entity = ?entity, "Queued script properties update");
                }
            }
        }
    }

    // Apply all queued commands after all scripts have run
    let commands = command_queue.write().unwrap().drain(..).collect::<Vec<_>>();
    if !commands.is_empty() {
        debug!(count = commands.len(), "Applying script commands");
        for command in commands {
            if let Err(e) = command.apply(world.inner_mut()) {
                error!(error = %e, "Failed to apply script command");
            }
        }
    }

    // Clear component cache to prevent stale data
    component_cache.write().unwrap().clear();

    // Clean up destroyed entities using two-phase approach to avoid borrow conflicts
    {
        // Phase 1: Collect entities that need to be checked
        let mut entities_to_check = Vec::new();
        {
            let tracker = get_tracker().lock().unwrap();
            entities_to_check.extend(tracker.active_entities.clone());
        }

        // Phase 2: Check which entities no longer have ScriptRef components
        let mut entities_to_remove = Vec::new();
        for entity in entities_to_check {
            // Check if entity still exists and has a ScriptRef component
            let still_has_script = world.contains(entity) && world.get::<ScriptRef>(entity).is_ok();

            if !still_has_script {
                entities_to_remove.push(entity);
                trace!(
                    entity_id = entity.to_bits().get(),
                    "Entity no longer has ScriptRef, marking for cleanup"
                );
            }
        }

        // Phase 3: Remove entities from tracker
        if !entities_to_remove.is_empty() {
            let mut tracker = get_tracker().lock().unwrap();
            for entity in &entities_to_remove {
                tracker.remove_entity(*entity);
                debug!(
                    entity_id = entity.to_bits().get(),
                    "Removed entity from script tracker"
                );
            }
            debug!(
                count = entities_to_remove.len(),
                "Cleaned up destroyed script entities"
            );
        }
    }

    debug!("Script execution system completed");
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
