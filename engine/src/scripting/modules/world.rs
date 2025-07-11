//! World API for accessing entities and components from scripts

use crate::core::entity::{Transform, World};
use crate::graphics::Material;
use crate::scripting::commands::{
    CommandQueue, ComponentData, ScriptCommand, SharedComponentCache,
};
use rhai::{Dynamic, Engine, EvalAltResult, Module};
use std::sync::{Arc, RwLock};
use tracing::{debug, trace};

/// Thread-safe reference to the ECS world
pub type WorldRef = Arc<RwLock<*mut World>>;

/// Register world API with Rhai engine
pub fn register_world_api(engine: &mut Engine) {
    debug!("Registering world API");

    // Create world module
    let mut world_module = Module::new();

    // Note: The actual world reference will be set per-script execution
    // For now, we register the functions that will be available

    // Register component getter/setter functions
    // These will be dynamically linked during script execution
    world_module.set_native_fn("get_component", |_entity: i64, _component_type: &str| {
        // This is a placeholder - actual implementation will be injected
        Ok(Dynamic::UNIT)
    });

    world_module.set_native_fn(
        "set_component",
        |_entity: i64, _component_type: &str, _value: Dynamic| {
            // This is a placeholder - actual implementation will be injected
            Ok(())
        },
    );

    world_module.set_native_fn("find_entities_with_component", |_component_type: &str| {
        // This is a placeholder - actual implementation will be injected
        Ok(Vec::<i64>::new())
    });

    world_module.set_native_fn("get_entity_name", |_entity: i64| {
        // This is a placeholder - actual implementation will be injected
        Ok(String::new())
    });

    engine.register_static_module("world", world_module.into());

    debug!("World API registered");
}

/// Create a world module with actual world access through command queue
pub fn create_world_module(
    command_queue: CommandQueue,
    component_cache: SharedComponentCache,
) -> Module {
    let mut module = Module::new();

    // Get Transform component
    let cache = component_cache.clone();
    module.set_native_fn(
        "get_transform",
        move |entity: i64| -> Result<Dynamic, Box<EvalAltResult>> {
            let entity_id = entity as u64;
            let cache_guard = cache.read().unwrap();

            if let Some(transform) = cache_guard.transforms.get(&entity_id) {
                trace!(entity = entity_id, "Retrieved Transform from cache");
                Ok(Dynamic::from(*transform))
            } else {
                Err(format!("Entity {entity_id} not found or missing Transform component").into())
            }
        },
    );

    // Set Transform component
    let queue = command_queue.clone();
    module.set_native_fn(
        "set_transform",
        move |entity: i64, transform: Transform| -> Result<(), Box<EvalAltResult>> {
            let entity_id = entity as u64;
            queue.write().unwrap().push(ScriptCommand::SetTransform {
                entity: entity_id,
                transform,
            });
            trace!(entity = entity_id, "Queued Transform update");
            Ok(())
        },
    );

    // Get Material component
    let cache = component_cache.clone();
    module.set_native_fn(
        "get_material",
        move |entity: i64| -> Result<Dynamic, Box<EvalAltResult>> {
            let entity_id = entity as u64;
            let cache_guard = cache.read().unwrap();

            if let Some(material) = cache_guard.materials.get(&entity_id) {
                trace!(entity = entity_id, "Retrieved Material from cache");
                Ok(Dynamic::from(*material))
            } else {
                Err(format!("Entity {entity_id} not found or missing Material component").into())
            }
        },
    );

    // Set Material component
    let queue = command_queue.clone();
    module.set_native_fn(
        "set_material",
        move |entity: i64, material: Material| -> Result<(), Box<EvalAltResult>> {
            let entity_id = entity as u64;
            queue.write().unwrap().push(ScriptCommand::SetMaterial {
                entity: entity_id,
                material,
            });
            trace!(entity = entity_id, "Queued Material update");
            Ok(())
        },
    );

    // Get entity name
    let cache = component_cache.clone();
    module.set_native_fn(
        "get_entity_name",
        move |entity: i64| -> Result<String, Box<EvalAltResult>> {
            let entity_id = entity as u64;
            let cache_guard = cache.read().unwrap();

            if let Some(name) = cache_guard.names.get(&entity_id) {
                trace!(entity = entity_id, name = name, "Retrieved Name from cache");
                Ok(name.clone())
            } else {
                Ok(format!("Entity_{entity_id}"))
            }
        },
    );

    // Generic get_component function for compatibility
    let cache = component_cache.clone();
    module.set_native_fn(
        "get_component",
        move |entity: i64, component_type: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            let entity_id = entity as u64;
            let cache_guard = cache.read().unwrap();

            match component_type {
                "Transform" => {
                    if let Some(transform) = cache_guard.transforms.get(&entity_id) {
                        Ok(Dynamic::from(*transform))
                    } else {
                        Err(format!("Entity {entity_id} missing Transform").into())
                    }
                }
                "Material" => {
                    if let Some(material) = cache_guard.materials.get(&entity_id) {
                        Ok(Dynamic::from(*material))
                    } else {
                        Err(format!("Entity {entity_id} missing Material").into())
                    }
                }
                "Name" => {
                    if let Some(name) = cache_guard.names.get(&entity_id) {
                        Ok(Dynamic::from(name.clone()))
                    } else {
                        Ok(Dynamic::from(format!("Entity_{entity_id}")))
                    }
                }
                _ => Err(format!("Unknown component type: {component_type}").into()),
            }
        },
    );

    // Generic set_component function for compatibility
    let queue = command_queue.clone();
    module.set_native_fn(
        "set_component",
        move |entity: i64,
              component_type: &str,
              value: Dynamic|
              -> Result<(), Box<EvalAltResult>> {
            let entity_id = entity as u64;

            match component_type {
                "Transform" => {
                    if let Some(transform) = value.clone().try_cast::<Transform>() {
                        queue.write().unwrap().push(ScriptCommand::SetTransform {
                            entity: entity_id,
                            transform,
                        });
                        Ok(())
                    } else {
                        Err("Invalid Transform value".into())
                    }
                }
                "Material" => {
                    if let Some(material) = value.clone().try_cast::<Material>() {
                        queue.write().unwrap().push(ScriptCommand::SetMaterial {
                            entity: entity_id,
                            material,
                        });
                        Ok(())
                    } else {
                        Err("Invalid Material value".into())
                    }
                }
                _ => Err(format!("Cannot set component type: {component_type}").into()),
            }
        },
    );

    // Find entities with component
    let cache = component_cache.clone();
    module.set_native_fn(
        "find_entities_with_component",
        move |component_type: &str| -> Result<Vec<i64>, Box<EvalAltResult>> {
            let cache_guard = cache.read().unwrap();

            let entities: Vec<i64> = match component_type {
                "Transform" => cache_guard.transforms.keys().map(|&id| id as i64).collect(),
                "Material" => cache_guard.materials.keys().map(|&id| id as i64).collect(),
                "Name" => cache_guard.names.keys().map(|&id| id as i64).collect(),
                _ => vec![],
            };

            debug!(
                component_type,
                count = entities.len(),
                "Found entities with component"
            );
            Ok(entities)
        },
    );

    // Create entity
    let queue = command_queue.clone();
    module.set_native_fn(
        "create_entity",
        move || -> Result<(), Box<EvalAltResult>> {
            queue.write().unwrap().push(ScriptCommand::CreateEntity {
                components: vec![ComponentData::Transform(Transform::default())],
            });
            debug!("Queued entity creation");
            Ok(())
        },
    );

    // Create entity with components
    let queue = command_queue.clone();
    module.set_native_fn(
        "create_entity_with",
        move |transform: Transform, material: Material| -> Result<(), Box<EvalAltResult>> {
            queue.write().unwrap().push(ScriptCommand::CreateEntity {
                components: vec![
                    ComponentData::Transform(transform),
                    ComponentData::Material(material),
                ],
            });
            debug!("Queued entity creation with components");
            Ok(())
        },
    );

    // Destroy entity
    let queue = command_queue.clone();
    module.set_native_fn(
        "destroy_entity",
        move |entity: i64| -> Result<(), Box<EvalAltResult>> {
            let entity_id = entity as u64;
            queue
                .write()
                .unwrap()
                .push(ScriptCommand::DestroyEntity { entity: entity_id });
            debug!(entity = entity_id, "Queued entity destruction");
            Ok(())
        },
    );

    module
}

// Register Material type for scripts
pub fn register_material_type(engine: &mut Engine) {
    // Register Material type
    engine
        .register_type_with_name::<Material>("Material")
        .register_get("color", |m: &mut Material| {
            vec![
                Dynamic::from(m.color[0] as f64),
                Dynamic::from(m.color[1] as f64),
                Dynamic::from(m.color[2] as f64),
                Dynamic::from(m.color[3] as f64),
            ]
        })
        .register_fn("clone", |m: &mut Material| *m);

    // Create a material module with constructor functions
    let mut material_module = Module::new();
    material_module.set_native_fn("gray", |value: f64| Ok(Material::gray(value as f32)));
    material_module.set_native_fn("red", || Ok(Material::red()));
    material_module.set_native_fn("green", || Ok(Material::green()));
    material_module.set_native_fn("blue", || Ok(Material::blue()));
    material_module.set_native_fn("from_rgb", |r: f64, g: f64, b: f64| {
        Ok(Material::from_rgb(r as f32, g as f32, b as f32))
    });
    material_module.set_native_fn("from_rgba", |r: f64, g: f64, b: f64, a: f64| {
        Ok(Material::from_rgba(r as f32, g as f32, b as f32, a as f32))
    });

    engine.register_static_module("Material", material_module.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_api_registration() {
        let mut engine = Engine::new();
        register_world_api(&mut engine);

        // The functions should be available but return placeholder values
        let result: Dynamic = engine
            .eval(
                r#"
            world::get_component(0, "Transform")
        "#,
            )
            .unwrap();
        // Dynamic doesn't implement PartialEq, so just ensure it exists
        let _ = result;
    }

    #[test]
    fn test_material_registration() {
        let mut engine = Engine::new();
        register_material_type(&mut engine);

        // Material functions are in the Material module
        let result: Material = engine.eval("Material::red()").unwrap();
        assert_eq!(result.color, [1.0, 0.0, 0.0, 1.0]);

        let result: Material = engine.eval("Material::from_rgb(0.5, 0.5, 0.5)").unwrap();
        assert_eq!(result.color, [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn test_command_queue_module() {
        let command_queue = CommandQueue::default();
        let component_cache = SharedComponentCache::default();

        // Populate cache with test data
        {
            let mut cache = component_cache.write().unwrap();
            cache.transforms.insert(1, Transform::default());
            cache.materials.insert(1, Material::red());
            cache.names.insert(1, "TestEntity".to_string());
        }

        let module = create_world_module(command_queue.clone(), component_cache);
        let mut engine = Engine::new();
        
        // Register Transform type first
        use crate::scripting::modules::math::register_math_types;
        register_math_types(&mut engine);
        
        engine.register_static_module("world", module.into());

        // Test get_transform
        let result: Transform = engine.eval("world::get_transform(1)").unwrap();
        assert_eq!(result, Transform::default());

        // Test set_transform (should queue command)
        engine
            .eval::<()>("world::set_transform(1, Transform::create())")
            .unwrap();
        assert_eq!(command_queue.read().unwrap().len(), 1);
    }
}
