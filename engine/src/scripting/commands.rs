use crate::core::entity::components::Transform;
use crate::core::entity::{Entity, Name};
use crate::graphics::material::Material;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error};

#[derive(Clone, Debug)]
pub enum ScriptCommand {
    SetTransform { entity: u64, transform: Transform },
    SetMaterial { entity: u64, material: Material },
    CreateEntity { components: Vec<ComponentData> },
    DestroyEntity { entity: u64 },
}

#[derive(Clone, Debug)]
pub enum ComponentData {
    Transform(Transform),
    Material(Material),
    Name(String),
}

pub type CommandQueue = Arc<RwLock<Vec<ScriptCommand>>>;

#[derive(Clone, Default)]
pub struct ComponentCache {
    pub transforms: HashMap<u64, Transform>,
    pub materials: HashMap<u64, Material>,
    pub names: HashMap<u64, String>,
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
    }
}
