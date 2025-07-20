use crate::core::entity::components::Transform;
use crate::core::entity::{Entity, Name};
use crate::graphics::material::Material;
use crate::graphics::mesh::Mesh;
use crate::graphics::renderer::MeshId;
use crate::scripting::property_types::ScriptProperties;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error};

#[derive(Clone, Debug)]
pub enum ScriptCommand {
    SetTransform {
        entity: u64,
        transform: Transform,
    },
    SetMaterial {
        entity: u64,
        material: Material,
    },
    CreateEntity {
        components: Vec<ComponentData>,
    },
    DestroyEntity {
        entity: u64,
    },
    SetProperties {
        entity: u64,
        properties: ScriptProperties,
    },
    UploadMesh {
        name: String,
        mesh: Mesh,
        callback_id: u64,
    },
    SetMeshId {
        entity: u64,
        mesh_id: MeshId,
    },
}

#[derive(Clone, Debug)]
pub enum ComponentData {
    Transform(Transform),
    Material(Material),
    Name(String),
    MeshId(MeshId),
}

pub type CommandQueue = Arc<RwLock<Vec<ScriptCommand>>>;

#[derive(Clone, Default)]
pub struct ComponentCache {
    pub transforms: HashMap<u64, Transform>,
    pub materials: HashMap<u64, Material>,
    pub names: HashMap<u64, String>,
    pub mesh_ids: HashMap<u64, MeshId>,
}

pub type SharedComponentCache = Arc<RwLock<ComponentCache>>;

impl ComponentCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.transforms.clear();
        self.materials.clear();
        self.names.clear();
        self.mesh_ids.clear();
    }
}

impl ScriptCommand {
    pub fn apply(&self, world: &mut hecs::World) -> Result<(), String> {
        match self {
            ScriptCommand::SetTransform { entity, transform } => {
                if let Some(ent) = Entity::from_bits(*entity) {
                    if world.contains(ent) {
                        world
                            .insert_one(ent, *transform)
                            .map_err(|e| format!("Failed to insert transform: {e:?}"))?;
                        debug!(entity = *entity, "Applied transform update from script");
                        Ok(())
                    } else {
                        error!(entity = *entity, "Entity not found for transform update");
                        Err(format!("Entity {entity} not found"))
                    }
                } else {
                    error!(entity = *entity, "Invalid entity ID");
                    Err(format!("Invalid entity ID: {entity}"))
                }
            }
            ScriptCommand::SetMaterial { entity, material } => {
                if let Some(ent) = Entity::from_bits(*entity) {
                    if world.contains(ent) {
                        world
                            .insert_one(ent, *material)
                            .map_err(|e| format!("Failed to insert material: {e:?}"))?;
                        debug!(entity = *entity, "Applied material update from script");
                        Ok(())
                    } else {
                        error!(entity = *entity, "Entity not found for material update");
                        Err(format!("Entity {entity} not found"))
                    }
                } else {
                    error!(entity = *entity, "Invalid entity ID");
                    Err(format!("Invalid entity ID: {entity}"))
                }
            }
            ScriptCommand::CreateEntity { components } => {
                let mut entity_builder = hecs::EntityBuilder::new();

                for component in components {
                    match component {
                        ComponentData::Transform(t) => {
                            entity_builder.add(*t);
                        }
                        ComponentData::Material(m) => {
                            entity_builder.add(*m);
                        }
                        ComponentData::Name(n) => {
                            entity_builder.add(Name::new(n.clone()));
                        }
                        ComponentData::MeshId(id) => {
                            entity_builder.add(id.clone());
                        }
                    }
                }

                let entity_id = world.spawn(entity_builder.build());

                debug!(
                    entity = entity_id.to_bits().get(),
                    "Created entity from script"
                );
                Ok(())
            }
            ScriptCommand::DestroyEntity { entity } => {
                if let Some(ent) = Entity::from_bits(*entity) {
                    if world.contains(ent) {
                        world
                            .despawn(ent)
                            .map_err(|e| format!("Failed to destroy entity: {e:?}"))?;
                        debug!(entity = *entity, "Destroyed entity from script");
                        Ok(())
                    } else {
                        error!(entity = *entity, "Entity not found for destruction");
                        Err(format!("Entity {entity} not found"))
                    }
                } else {
                    error!(entity = *entity, "Invalid entity ID");
                    Err(format!("Invalid entity ID: {entity}"))
                }
            }
            ScriptCommand::SetProperties { entity, properties } => {
                if let Some(ent) = Entity::from_bits(*entity) {
                    if world.contains(ent) {
                        world
                            .insert_one(ent, properties.clone())
                            .map_err(|e| format!("Failed to update properties: {e:?}"))?;
                        debug!(entity = *entity, "Updated script properties from script");
                        Ok(())
                    } else {
                        error!(entity = *entity, "Entity not found for properties update");
                        Err(format!("Entity {entity} not found"))
                    }
                } else {
                    error!(entity = *entity, "Invalid entity ID");
                    Err(format!("Invalid entity ID: {entity}"))
                }
            }
            ScriptCommand::UploadMesh {
                name,
                mesh: _,
                callback_id,
            } => {
                // Mesh upload is handled separately by the mesh upload system
                debug!(
                    name = name,
                    callback_id = callback_id,
                    "Mesh upload command queued"
                );
                Ok(())
            }
            ScriptCommand::SetMeshId { entity, mesh_id } => {
                if let Some(ent) = Entity::from_bits(*entity) {
                    if world.contains(ent) {
                        world
                            .insert_one(ent, mesh_id.clone())
                            .map_err(|e| format!("Failed to insert mesh ID: {e:?}"))?;
                        debug!(entity = *entity, mesh_id = ?mesh_id, "Applied mesh ID update from script");
                        Ok(())
                    } else {
                        error!(entity = *entity, "Entity not found for mesh ID update");
                        Err(format!("Entity {entity} not found"))
                    }
                } else {
                    error!(entity = *entity, "Invalid entity ID");
                    Err(format!("Invalid entity ID: {entity}"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_queue_thread_safety() {
        let queue = CommandQueue::default();
        let q1 = queue.clone();

        std::thread::spawn(move || {
            q1.write().unwrap().push(ScriptCommand::SetTransform {
                entity: 1,
                transform: Transform::default(),
            });
        })
        .join()
        .unwrap();

        assert_eq!(queue.read().unwrap().len(), 1);
    }

    #[test]
    fn test_component_cache() {
        let mut cache = ComponentCache::new();
        cache.transforms.insert(1, Transform::default());
        cache.materials.insert(1, Material::default());
        cache.names.insert(1, "Test".to_string());

        assert_eq!(cache.transforms.len(), 1);
        assert_eq!(cache.materials.len(), 1);
        assert_eq!(cache.names.len(), 1);

        cache.clear();

        assert_eq!(cache.transforms.len(), 0);
        assert_eq!(cache.materials.len(), 0);
        assert_eq!(cache.names.len(), 0);
        assert_eq!(cache.mesh_ids.len(), 0);
    }

    #[test]
    fn test_set_properties_command() {
        use crate::scripting::property_types::{PropertyValue, ScriptProperties};

        let mut world = hecs::World::new();
        let entity = world.spawn((Transform::default(),));
        let entity_id = entity.to_bits().get();

        // Create properties to set
        let mut properties = ScriptProperties::new();
        properties
            .values
            .insert("test".to_string(), PropertyValue::Float(42.0));

        // Create and apply the command
        let command = ScriptCommand::SetProperties {
            entity: entity_id,
            properties: properties.clone(),
        };

        assert!(command.apply(&mut world).is_ok());

        // Verify the properties were applied
        let props = world.get::<&ScriptProperties>(entity).unwrap();
        assert_eq!(props.values.get("test"), Some(&PropertyValue::Float(42.0)));
    }

    #[test]
    fn test_set_properties_command_invalid_entity() {
        let mut world = hecs::World::new();
        let properties = ScriptProperties::new();

        let command = ScriptCommand::SetProperties {
            entity: 9999, // Non-existent entity
            properties,
        };

        assert!(command.apply(&mut world).is_err());
    }
}
