//! World wrapper providing helper methods for entity management

use super::components::{GlobalTransform, Transform};
use crate::graphics::{Material, MeshId};
use crate::io::{ReloadCallback, SceneWatcher, WatcherConfig};
use hecs::Entity;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Wrapper around hecs::World providing additional helper methods
pub struct World {
    inner: hecs::World,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
        }
    }

    /// Spawn a new entity with the given components
    pub fn spawn(&mut self, components: impl hecs::DynamicBundle) -> Entity {
        self.inner.spawn(components)
    }

    /// Get a reference to a component on an entity
    pub fn get<T: hecs::Component>(
        &self,
        entity: Entity,
    ) -> Result<hecs::Ref<T>, hecs::ComponentError> {
        self.inner.get::<&T>(entity)
    }

    /// Query a single entity for a mutable component reference
    pub fn query_one_mut<Q: hecs::Query>(
        &mut self,
        entity: Entity,
    ) -> Result<Q::Item<'_>, hecs::QueryOneError> {
        self.inner.query_one_mut::<Q>(entity)
    }

    /// Insert a component into an entity
    pub fn insert_one(
        &mut self,
        entity: Entity,
        component: impl hecs::Component,
    ) -> Result<(), hecs::NoSuchEntity> {
        self.inner.insert_one(entity, component)
    }

    /// Query entities with specific components
    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<Q> {
        self.inner.query()
    }

    /// Query entities with specific components (mutable)
    pub fn query_mut<Q: hecs::Query>(&mut self) -> hecs::QueryMut<Q> {
        self.inner.query_mut()
    }

    /// Despawn an entity and all its components
    pub fn despawn(&mut self, entity: Entity) -> Result<(), hecs::NoSuchEntity> {
        self.inner.despawn(entity)
    }

    /// Check if an entity exists
    pub fn contains(&self, entity: Entity) -> bool {
        self.inner.contains(entity)
    }

    /// Helper method to spawn an entity with required transform components
    /// This ensures that entities have both Transform and GlobalTransform
    pub fn spawn_with_transform(&mut self, components: impl hecs::DynamicBundle) -> Entity {
        let entity = self.spawn(components);

        // Auto-add Transform if missing
        if self.get::<Transform>(entity).is_err() {
            let _ = self.insert_one(entity, Transform::default());
            debug!(entity = ?entity, "Auto-added Transform component");
        }

        // Auto-add GlobalTransform if missing
        if self.get::<GlobalTransform>(entity).is_err() {
            let _ = self.insert_one(entity, GlobalTransform::default());
            debug!(entity = ?entity, "Auto-added GlobalTransform component");
        }

        entity
    }

    /// Generic helper to add an entity with automatic component requirements
    /// This is a template for game-specific helper methods
    pub fn add_with_requirements<T: hecs::Component + ComponentRequirements>(
        &mut self,
        component: T,
    ) -> Entity {
        let mut builder = hecs::EntityBuilder::new();
        builder.add(component);

        // Add required components
        T::add_requirements(&mut builder);

        let entity = self.inner.spawn(builder.build());
        debug!(entity = ?entity, component_type = std::any::type_name::<T>(), "Spawned entity with requirements");

        entity
    }

    /// Get access to the inner hecs::World for advanced operations
    pub fn inner(&self) -> &hecs::World {
        &self.inner
    }

    /// Get mutable access to the inner hecs::World for advanced operations
    pub fn inner_mut(&mut self) -> &mut hecs::World {
        &mut self.inner
    }

    /// Save the current world state to a scene file
    ///
    /// This is a convenience method that creates a scene from the world
    /// and saves it to the specified path.
    pub fn save_scene<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::io::Scene;

        let scene = Scene::from_world(self);
        scene.save_to_file(path)?;
        Ok(())
    }

    /// Load a scene from a file, replacing the current world content
    ///
    /// This clears the world and loads entities from the scene file.
    /// For additive loading, use `load_scene_additive` instead.
    pub fn load_scene<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::io::Scene;

        // Clear the world first
        self.inner.clear();

        let scene = Scene::load_from_file(path)?;
        scene.instantiate(self)?;
        Ok(())
    }

    /// Load a scene from a file additively, keeping existing entities
    ///
    /// This loads entities from the scene file and adds them to the world
    /// without clearing existing entities. Returns an EntityMapper for
    /// referencing the newly loaded entities.
    pub fn load_scene_additive<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<crate::io::EntityMapper, Box<dyn std::error::Error>> {
        use crate::io::Scene;

        let scene = Scene::load_from_file(path)?;
        let mapper = scene.instantiate(self)?;
        Ok(mapper)
    }

    /// Assign default meshes to entities that have transforms but no MeshId
    ///
    /// This method automatically assigns default meshes and materials to entities
    /// that have transform components but are missing graphics components.
    /// It's useful for quickly making a scene renderable after loading.
    pub fn assign_default_meshes(&mut self) {
        let mut entities_to_update = Vec::new();

        // Find entities with transform but no MeshId
        for (entity, (_transform, _global_transform)) in self.query::<(&Transform, &GlobalTransform)>().iter() {
            if self.get::<MeshId>(entity).is_err() {
                entities_to_update.push(entity);
            }
        }

        info!(count = entities_to_update.len(), "Assigning default meshes to entities");

        // Assign default meshes and materials
        for entity in entities_to_update {
            // Assign default cube mesh
            let mesh_id = MeshId("cube".to_string());
            if let Err(e) = self.insert_one(entity, mesh_id) {
                warn!(entity = ?entity, error = ?e, "Failed to assign default MeshId");
            } else {
                debug!(entity = ?entity, "Assigned default cube mesh");
            }

            // Assign default material if missing
            if self.get::<Material>(entity).is_err() {
                let material = Material::default();
                if let Err(e) = self.insert_one(entity, material) {
                    warn!(entity = ?entity, error = ?e, "Failed to assign default Material");
                } else {
                    debug!(entity = ?entity, "Assigned default material");
                }
            }
        }
    }

    /// Get statistics about the current scene
    ///
    /// Returns information about entity counts, component distributions,
    /// and other useful debugging information.
    pub fn get_scene_stats(&self) -> SceneStats {
        let mut stats = SceneStats {
            entity_count: 0,
            renderable_count: 0,
            camera_count: 0,
            mesh_types: HashMap::new(),
            material_count: 0,
        };

        // Count total entities
        stats.entity_count = self.query::<()>().iter().count();

        // Count renderable entities (have MeshId + Material + Transform)
        stats.renderable_count = self.query::<(&MeshId, &Material, &GlobalTransform)>().iter().count();

        // Count cameras
        stats.camera_count = self.query::<&crate::core::camera::Camera>().iter().count();

        // Count mesh types
        for (_, mesh_id) in self.query::<&MeshId>().iter() {
            *stats.mesh_types.entry(mesh_id.0.clone()).or_insert(0) += 1;
        }

        // Count materials
        stats.material_count = self.query::<&Material>().iter().count();

        stats
    }

    /// Clear all graphics components from entities
    ///
    /// This removes MeshId and Material components from all entities,
    /// useful for resetting the visual state while keeping transform hierarchy.
    pub fn clear_graphics_components(&mut self) {
        let mut entities_to_clear = Vec::new();

        // Find entities with graphics components
        for (entity, _) in self.query::<()>().iter() {
            if self.get::<MeshId>(entity).is_ok() || self.get::<Material>(entity).is_ok() {
                entities_to_clear.push(entity);
            }
        }

        info!(count = entities_to_clear.len(), "Clearing graphics components from entities");

        // Remove graphics components
        for entity in entities_to_clear {
            // Remove MeshId if present
            if self.get::<MeshId>(entity).is_ok() {
                if let Err(e) = self.inner.remove_one::<MeshId>(entity) {
                    warn!(entity = ?entity, error = ?e, "Failed to remove MeshId");
                }
            }

            // Remove Material if present
            if self.get::<Material>(entity).is_ok() {
                if let Err(e) = self.inner.remove_one::<Material>(entity) {
                    warn!(entity = ?entity, error = ?e, "Failed to remove Material");
                }
            }
        }
    }

    /// Query a single entity for components
    pub fn query_one<Q: hecs::Query>(
        &self,
        entity: Entity,
    ) -> Result<hecs::QueryOne<'_, Q>, hecs::NoSuchEntity> {
        self.inner.query_one::<Q>(entity)
    }

    /// Set up hot-reload watching for a scene file
    ///
    /// This creates a SceneWatcher that will automatically reload the scene
    /// when the file changes. The callback receives the current world state
    /// and can be used to handle the reload process.
    ///
    /// Note: This is a simplified interface. In practice, you would need
    /// to manage the watcher lifecycle and resource access more carefully.
    pub fn watch_scene<P: AsRef<Path>>(
        scene_path: P,
        config: WatcherConfig,
    ) -> Result<SceneWatcher, Box<dyn std::error::Error + Send + Sync>> {
        let scene_path = scene_path.as_ref();
        info!(path = ?scene_path, "Setting up scene watching");

        // Create a callback that demonstrates the hot-reload workflow
        let callback: ReloadCallback = Box::new(|world, _renderer, _asset_manager| {
            info!("Hot-reload callback triggered");
            
            // In practice, this would reload the scene:
            // 1. Clear the world
            // 2. Load the scene with validation
            // 3. Instantiate entities
            // 4. Assign default meshes if needed
            
            // For demonstration, we just log the current state
            let stats = world.get_scene_stats();
            info!(
                entities = stats.entity_count,
                renderables = stats.renderable_count,
                "Scene state during hot-reload"
            );

            Ok(())
        });

        SceneWatcher::new(scene_path, config, callback)
    }

    /// Development helper: log scene debugging information
    pub fn log_debug_info(&self) {
        if crate::dev::DevTools::is_enabled() {
            let stats = self.get_scene_stats();
            crate::dev::DevTools::log_dev_info(&format!(
                "Scene: {} entities, {} renderable, {} cameras",
                stats.entity_count, stats.renderable_count, stats.camera_count
            ));

            // Log mesh usage
            for (mesh_name, count) in &stats.mesh_types {
                crate::dev::DevTools::log_dev_info(&format!(
                    "Mesh '{}': {} instances",
                    mesh_name, count
                ));
            }
        }
    }

    /// Development helper: validate scene health and log warnings
    pub fn check_scene_health(&self) {
        if crate::dev::DevTools::is_enabled() {
            let stats = self.get_scene_stats();
            
            // Check for common issues
            if stats.camera_count == 0 {
                crate::dev::DevTools::log_dev_warning("No camera entities found in scene");
            }
            
            if stats.camera_count > 1 {
                crate::dev::DevTools::log_dev_warning(&format!(
                    "Multiple cameras found: {}",
                    stats.camera_count
                ));
            }
            
            if stats.renderable_count == 0 && stats.entity_count > 0 {
                crate::dev::DevTools::log_dev_warning("Entities present but none are renderable");
            }
            
            // Check for entities with transforms but no mesh/material
            let transform_count = self.query::<&Transform>().iter().count();
            if transform_count > stats.renderable_count {
                crate::dev::DevTools::log_dev_info(&format!(
                    "{} entities with transforms could have default meshes assigned",
                    transform_count - stats.renderable_count
                ));
            }
        }
    }
}

/// Statistics about a scene's content
#[derive(Debug, Clone)]
pub struct SceneStats {
    /// Total number of entities
    pub entity_count: usize,
    /// Number of entities that can be rendered (have MeshId, Material, Transform)
    pub renderable_count: usize,
    /// Number of camera entities
    pub camera_count: usize,
    /// Count of each mesh type in use
    pub mesh_types: HashMap<String, usize>,
    /// Total number of entities with materials
    pub material_count: usize,
}

/// Trait for components that have requirements
pub trait ComponentRequirements: hecs::Component {
    /// Add required components to the entity builder
    fn add_requirements(builder: &mut hecs::EntityBuilder) {
        // By default, add Transform and GlobalTransform
        builder.add(Transform::default());
        builder.add(GlobalTransform::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::components::Parent;
    use glam::Vec3;

    #[test]
    fn test_world_spawn() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));
        assert!(world.contains(entity));
    }

    #[test]
    fn test_spawn_with_transform() {
        let mut world = World::new();

        // Spawn without transform components
        let entity = world.spawn_with_transform(());

        // Should have both components auto-added
        assert!(world.get::<Transform>(entity).is_ok());
        assert!(world.get::<GlobalTransform>(entity).is_ok());
    }

    #[test]
    fn test_spawn_with_transform_existing() {
        let mut world = World::new();

        // Spawn with existing Transform
        let custom_transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
        let entity = world.spawn_with_transform((custom_transform,));

        // Should keep the custom transform
        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.position, Vec3::new(1.0, 2.0, 3.0));

        // Should still add GlobalTransform
        assert!(world.get::<GlobalTransform>(entity).is_ok());
    }

    #[test]
    fn test_entity_hierarchy() {
        let mut world = World::new();

        let parent = world.spawn((Transform::default(), GlobalTransform::default()));
        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(parent),
        ));

        let parent_ref = world.get::<Parent>(child).unwrap();
        assert_eq!(parent_ref.0, parent);
    }

    #[test]
    fn test_save_load_scene() {
        let mut world = World::new();

        // Create some entities
        let entity1 = world.spawn((
            Transform::from_position(Vec3::new(1.0, 2.0, 3.0)),
            GlobalTransform::default(),
        ));
        let _entity2 = world.spawn((
            Transform::from_position(Vec3::X),
            GlobalTransform::default(),
            Parent(entity1),
        ));

        // Save to temp file
        let temp_path = "test_world_scene.json";
        world.save_scene(temp_path).unwrap();

        // Load into new world
        let mut new_world = World::new();
        new_world.load_scene(temp_path).unwrap();

        // Verify entities exist
        assert_eq!(new_world.query::<()>().iter().count(), 2);

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_load_scene_additive() {
        let mut world = World::new();

        // Add initial entity
        let existing = world.spawn((Transform::from_position(Vec3::Y),));

        // Create scene file
        let mut temp_world = World::new();
        temp_world.spawn((Transform::from_position(Vec3::X),));
        temp_world.spawn((Transform::from_position(Vec3::Z),));

        let temp_path = "test_additive_scene.json";
        temp_world.save_scene(temp_path).unwrap();

        // Load additively
        let mapper = world.load_scene_additive(temp_path).unwrap();

        // Should have 3 entities total (1 existing + 2 from scene)
        assert_eq!(world.query::<()>().iter().count(), 3);

        // Existing entity should still exist
        assert!(world.contains(existing));

        // New entities should exist
        assert_eq!(mapper.len(), 2);

        // Cleanup
        let _ = std::fs::remove_file(temp_path);
    }
}
