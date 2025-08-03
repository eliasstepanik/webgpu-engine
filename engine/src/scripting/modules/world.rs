//! World API for accessing entities and components from scripts

use crate::core::entity::{Transform, World};
use crate::graphics::renderer::MeshId;
use crate::graphics::Material;
use crate::scripting::commands::{
    CommandQueue, ComponentData, ScriptCommand, SharedComponentCache,
};
use rhai::{Dynamic, Engine, EvalAltResult, Module};
use std::sync::{Arc, RwLock};
use tracing::{debug, trace, warn};

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

    // Get position from Transform component
    let cache = component_cache.clone();
    module.set_native_fn(
        "get_position",
        move |entity: i64| -> Result<Dynamic, Box<EvalAltResult>> {
            let entity_id = entity as u64;
            let cache_guard = cache.read().unwrap();

            if let Some(transform) = cache_guard.transforms.get(&entity_id) {
                trace!(
                    entity = entity_id,
                    "Retrieved position from Transform cache"
                );
                let mut map = rhai::Map::new();
                map.insert("x".into(), Dynamic::from(transform.position.x as f64));
                map.insert("y".into(), Dynamic::from(transform.position.y as f64));
                map.insert("z".into(), Dynamic::from(transform.position.z as f64));
                Ok(Dynamic::from(map))
            } else {
                Err(format!("Entity {entity_id} not found or missing Transform component").into())
            }
        },
    );

    // Set position in Transform component
    let queue = command_queue.clone();
    let cache = component_cache.clone();
    module.set_native_fn(
        "set_position",
        move |entity: i64, position: rhai::Map| -> Result<(), Box<EvalAltResult>> {
            let entity_id = entity as u64;
            // Extract x, y, z from the map
            let x = position.get("x")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(0.0) as f32;
            let y = position.get("y")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(0.0) as f32;
            let z = position.get("z")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(0.0) as f32;

            // Get current transform to preserve rotation and scale
            let current_transform = {
                let cache_guard = cache.read().unwrap();
                cache_guard.transforms.get(&entity_id).copied()
                    .unwrap_or_else(|| Transform {
                        position: glam::Vec3::new(x, y, z),
                        rotation: glam::Quat::IDENTITY,
                        scale: glam::Vec3::ONE,
                    })
            };

            let new_transform = Transform {
                position: glam::Vec3::new(x, y, z),
                rotation: current_transform.rotation,
                scale: current_transform.scale,
            };

            queue.write().unwrap().push(ScriptCommand::SetTransform {
                entity: entity_id,
                transform: new_transform,
            });
            trace!(entity = entity_id, position = ?glam::Vec3::new(x, y, z), "Queued position update");
            Ok(())
        },
    );

    // Set scale in Transform component
    let queue = command_queue.clone();
    let cache = component_cache.clone();
    module.set_native_fn(
        "set_scale",
        move |entity: i64, scale: rhai::Map| -> Result<(), Box<EvalAltResult>> {
            let entity_id = entity as u64;

            // Extract x, y, z from the map
            let x = scale
                .get("x")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(1.0) as f32;
            let y = scale
                .get("y")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(1.0) as f32;
            let z = scale
                .get("z")
                .and_then(|v| v.as_float().ok())
                .unwrap_or(1.0) as f32;

            // Get current transform to preserve position and rotation
            let current_transform = {
                let cache_guard = cache.read().unwrap();
                cache_guard
                    .transforms
                    .get(&entity_id)
                    .copied()
                    .unwrap_or_else(|| Transform {
                        position: glam::Vec3::ZERO,
                        rotation: glam::Quat::IDENTITY,
                        scale: glam::Vec3::new(x, y, z),
                    })
            };

            let new_transform = Transform {
                position: current_transform.position,
                rotation: current_transform.rotation,
                scale: glam::Vec3::new(x, y, z),
            };

            queue.write().unwrap().push(ScriptCommand::SetTransform {
                entity: entity_id,
                transform: new_transform,
            });
            trace!(entity = entity_id, scale = ?glam::Vec3::new(x, y, z), "Queued scale update");
            Ok(())
        },
    );

    // Spawn a new entity - Note: This is a placeholder as we can't easily spawn entities from scripts
    // In a real implementation, this would need to communicate with the main world
    module.set_native_fn(
        "spawn_entity",
        move || -> Result<i64, Box<EvalAltResult>> {
            // For now, return a dummy entity ID
            // TODO: Implement proper entity spawning through command queue
            warn!("spawn_entity called from script - not fully implemented");
            Ok(0)
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

    // Set MeshId component
    let queue = command_queue.clone();
    module.set_native_fn(
        "set_mesh_id",
        move |entity: i64, mesh_id: &str| -> Result<(), Box<EvalAltResult>> {
            let entity_id = entity as u64;
            queue.write().unwrap().push(ScriptCommand::SetMeshId {
                entity: entity_id,
                mesh_id: MeshId(mesh_id.to_string()),
            });
            trace!(
                entity = entity_id,
                mesh_id = mesh_id,
                "Queued MeshId update"
            );
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

    // Create entity with mesh
    let queue = command_queue.clone();
    module.set_native_fn(
        "create_entity_with_mesh",
        move |transform: Transform,
              material: Material,
              mesh_id: &str|
              -> Result<(), Box<EvalAltResult>> {
            queue.write().unwrap().push(ScriptCommand::CreateEntity {
                components: vec![
                    ComponentData::Transform(transform),
                    ComponentData::Material(material),
                    ComponentData::MeshId(MeshId(mesh_id.to_string())),
                ],
            });
            debug!("Queued entity creation with mesh");
            Ok(())
        },
    );

    // Create entity with mesh and name
    let queue = command_queue.clone();
    module.set_native_fn(
        "create_entity_with_mesh_and_name",
        move |transform: Transform,
              material: Material,
              mesh_id: &str,
              name: &str|
              -> Result<(), Box<EvalAltResult>> {
            queue.write().unwrap().push(ScriptCommand::CreateEntity {
                components: vec![
                    ComponentData::Transform(transform),
                    ComponentData::Material(material),
                    ComponentData::MeshId(MeshId(mesh_id.to_string())),
                    ComponentData::Name(name.to_string()),
                ],
            });
            debug!("Queued entity creation with mesh and name");
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

    // Get entity count
    let cache = component_cache.clone();
    module.set_native_fn(
        "get_entity_count",
        move || -> Result<i64, Box<EvalAltResult>> {
            let cache_guard = cache.read().unwrap();
            // Count unique entities across all component types
            let mut entities = std::collections::HashSet::<u64>::new();
            entities.extend(cache_guard.transforms.keys());
            entities.extend(cache_guard.materials.keys());
            entities.extend(cache_guard.names.keys());
            let count = entities.len() as i64;
            trace!(count = count, "Counted entities");
            Ok(count)
        },
    );

    // Get all entities
    let cache = component_cache.clone();
    module.set_native_fn(
        "get_all_entities",
        move || -> Result<Vec<i64>, Box<EvalAltResult>> {
            let cache_guard = cache.read().unwrap();
            // Collect unique entities across all component types
            let mut entities = std::collections::HashSet::<u64>::new();
            entities.extend(cache_guard.transforms.keys());
            entities.extend(cache_guard.materials.keys());
            entities.extend(cache_guard.names.keys());
            let mut entity_vec: Vec<i64> = entities.iter().map(|&id| id as i64).collect();
            entity_vec.sort(); // Sort for consistent ordering
            trace!(count = entity_vec.len(), "Retrieved all entities");
            Ok(entity_vec)
        },
    );

    // Check if entity exists
    let cache = component_cache.clone();
    module.set_native_fn(
        "entity_exists",
        move |entity: i64| -> Result<bool, Box<EvalAltResult>> {
            let entity_id = entity as u64;
            let cache_guard = cache.read().unwrap();
            // Check if entity has any components
            let exists = cache_guard.transforms.contains_key(&entity_id)
                || cache_guard.materials.contains_key(&entity_id)
                || cache_guard.names.contains_key(&entity_id);
            trace!(
                entity = entity_id,
                exists = exists,
                "Checked entity existence"
            );
            Ok(exists)
        },
    );

    // Has component check
    let cache = component_cache.clone();
    module.set_native_fn(
        "has_component",
        move |entity: i64, component_type: &str| -> Result<bool, Box<EvalAltResult>> {
            let entity_id = entity as u64;
            let cache_guard = cache.read().unwrap();

            let has = match component_type {
                "Transform" => cache_guard.transforms.contains_key(&entity_id),
                "Material" => cache_guard.materials.contains_key(&entity_id),
                "Name" => cache_guard.names.contains_key(&entity_id),
                _ => false,
            };

            trace!(
                entity = entity_id,
                component_type = component_type,
                has = has,
                "Checked component presence"
            );
            Ok(has)
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
        .register_set("color", |m: &mut Material, color: Dynamic| {
            // Try to handle as a direct array first
            if color.is_array() {
                let array = color.cast::<rhai::Array>();
                if array.len() >= 3 {
                    // Extract values from Dynamic array elements
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        array[0].as_float(),
                        array[1].as_float(),
                        array[2].as_float(),
                    ) {
                        m.color[0] = (r as f32).clamp(0.0, 1.0);
                        m.color[1] = (g as f32).clamp(0.0, 1.0);
                        m.color[2] = (b as f32).clamp(0.0, 1.0);

                        // Set alpha if provided
                        if array.len() >= 4 {
                            if let Ok(a) = array[3].as_float() {
                                m.color[3] = (a as f32).clamp(0.0, 1.0);
                            }
                        }
                    }
                }
            }
            // Note: Vec<Dynamic> from the getter will also be handled by the is_array() check
        })
        .register_fn(
            "set_color",
            |m: &mut Material, r: f64, g: f64, b: f64, a: f64| {
                m.color[0] = (r as f32).clamp(0.0, 1.0);
                m.color[1] = (g as f32).clamp(0.0, 1.0);
                m.color[2] = (b as f32).clamp(0.0, 1.0);
                m.color[3] = (a as f32).clamp(0.0, 1.0);
            },
        )
        .register_fn("set_rgb", |m: &mut Material, r: f64, g: f64, b: f64| {
            m.color[0] = (r as f32).clamp(0.0, 1.0);
            m.color[1] = (g as f32).clamp(0.0, 1.0);
            m.color[2] = (b as f32).clamp(0.0, 1.0);
            // Keep existing alpha
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
