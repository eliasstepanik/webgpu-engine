//! Input/Output module for asset loading and scene serialization

pub mod component_registry;
mod entity_mapper;
pub mod hot_reload;
mod scene;

pub use component_registry::ComponentRegistry;
pub use entity_mapper::EntityMapper;
pub use hot_reload::{reload_scene_with_validation, ReloadCallback, SceneWatcher, WatcherConfig};
pub use scene::{Scene, SceneError, SerializedEntity};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::{
        components::{GlobalTransform, Parent, Transform},
        World,
    };
    use glam::Vec3;
    use std::fs;

    #[test]
    fn test_scene_round_trip() {
        let mut world = World::new();

        // Create test hierarchy
        let parent = world.spawn((
            Transform::from_position(Vec3::new(1.0, 2.0, 3.0)),
            GlobalTransform::default(),
        ));

        let _child = world.spawn((
            Transform::from_position(Vec3::X),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Save to scene
        let scene = Scene::from_world(&world);

        // Create new world and load
        let mut new_world = World::new();
        let mapper = scene.instantiate(&mut new_world).unwrap();

        // Verify structure preserved
        assert_eq!(new_world.query::<()>().iter().count(), 2);

        // Check parent relationship remapped correctly
        let remapped_child = mapper.remap(1).unwrap(); // child was second entity
        let child_parent = new_world.get::<Parent>(remapped_child).unwrap();
        let remapped_parent = mapper.remap(0).unwrap();
        assert_eq!(child_parent.0, remapped_parent);
    }

    #[test]
    fn test_missing_component() {
        let json = r#"{
            "entities": [{
                "components": {
                    "Transform": {"position":[0,0,0],"rotation":[0,0,0,1],"scale":[1,1,1]},
                    "UnknownComponent": {"data": "ignored"}
                }
            }]
        }"#;

        let scene: Scene = serde_json::from_str(json).unwrap();
        let mut world = World::new();

        // Should not panic, just warn
        let result = scene.instantiate(&mut world);
        assert!(result.is_ok());
        assert_eq!(world.query::<&Transform>().iter().count(), 1);
    }

    #[test]
    fn test_empty_scene() {
        let json = r#"{"entities": []}"#;
        let scene: Scene = serde_json::from_str(json).unwrap();
        let mut world = World::new();

        let result = scene.instantiate(&mut world);
        assert!(result.is_ok());
        assert_eq!(world.query::<()>().iter().count(), 0);
    }

    #[test]
    fn test_parent_remapping() {
        let mut world = World::new();

        // Create a chain: grandparent -> parent -> child
        let grandparent = world.spawn((Transform::default(), GlobalTransform::default()));
        let parent = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(grandparent),
        ));
        let _child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(parent),
        ));

        let scene = Scene::from_world(&world);
        let mut new_world = World::new();
        let mapper = scene.instantiate(&mut new_world).unwrap();

        // Verify all entities exist
        assert_eq!(new_world.query::<()>().iter().count(), 3);

        // Verify parent relationships
        let new_child = mapper.remap(2).unwrap();
        let new_parent = mapper.remap(1).unwrap();
        let new_grandparent = mapper.remap(0).unwrap();

        let child_parent = new_world.get::<Parent>(new_child).unwrap();
        assert_eq!(child_parent.0, new_parent);

        let parent_parent = new_world.get::<Parent>(new_parent).unwrap();
        assert_eq!(parent_parent.0, new_grandparent);
    }

    #[test]
    fn test_additive_loading() {
        let mut world = World::new();

        // Add initial entities
        let existing = world.spawn((Transform::from_position(Vec3::Y),));

        // Create scene with new entities
        let mut scene_world = World::new();
        scene_world.spawn((Transform::from_position(Vec3::X),));
        scene_world.spawn((Transform::from_position(Vec3::Z),));
        let scene = Scene::from_world(&scene_world);

        // Load additively
        let mapper = scene.instantiate(&mut world).unwrap();

        // Should have 3 entities total
        assert_eq!(world.query::<()>().iter().count(), 3);

        // Existing entity should still exist
        assert!(world.contains(existing));

        // New entities should exist
        assert!(world.contains(mapper.remap(0).unwrap()));
        assert!(world.contains(mapper.remap(1).unwrap()));
    }

    #[test]
    fn test_scene_file_io() {
        let mut world = World::new();
        world.spawn((
            Transform::from_position(Vec3::new(1.0, 2.0, 3.0)),
            GlobalTransform::default(),
        ));

        let scene = Scene::from_world(&world);

        // Create temp file path
        let path = "test_scene_temp.json";

        // Save
        scene.save_to_file(path).unwrap();

        // Load
        let loaded_scene = Scene::load_from_file(path).unwrap();

        // Clean up
        let _ = fs::remove_file(path);

        // Verify
        assert_eq!(loaded_scene.entities.len(), 1);
    }
}
