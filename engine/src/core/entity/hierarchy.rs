//! Hierarchy system for updating global transforms based on parent relationships

use super::components::{GlobalTransform, Parent, Transform};
use super::world::World;
// hecs::Entity is re-exported from world module
use std::collections::HashSet;
use std::ops::Mul;
use tracing::{debug, error};

/// Update the hierarchy system, calculating global transforms from local transforms
/// and parent relationships using breadth-first traversal
pub fn update_hierarchy_system(world: &mut World) {
    // Pre-allocate collections for performance
    let mut queue = Vec::with_capacity(1024);
    let mut visited = HashSet::with_capacity(1024);
    let mut next_level = Vec::new();

    // Find root entities (entities with Transform but no Parent)
    let inner = world.inner_mut();

    // Collect root entities first
    let mut root_updates = Vec::new();
    for (entity, (transform,)) in inner.query::<(&Transform,)>().without::<&Parent>().iter() {
        let world_matrix = transform.to_matrix();
        root_updates.push((entity, world_matrix));
        visited.insert(entity);
    }

    // Update root entities' global transforms
    for (entity, world_matrix) in &root_updates {
        if let Ok(query) = inner.query_one_mut::<&mut GlobalTransform>(*entity) {
            query.matrix = *world_matrix;
        } else {
            let _ = inner.insert_one(*entity, GlobalTransform::from_matrix(*world_matrix));
        }
    }

    // Start BFS from roots - we'll process their children
    queue.extend(root_updates);

    debug!(root_count = queue.len(), "Starting hierarchy update");

    // BFS traversal - process children only, roots are already done
    while !queue.is_empty() {
        // Process current level - collect all children first
        let mut child_updates = Vec::new();

        for (parent_entity, parent_world_matrix) in queue.drain(..) {
            // Find children of this entity
            for (child_entity, parent) in inner.query::<&Parent>().iter() {
                if parent.0 == parent_entity {
                    // Check for cycles
                    if visited.contains(&child_entity) {
                        error!(
                            parent = ?parent_entity,
                            child = ?child_entity,
                            "Cyclic parent-child relationship detected"
                        );
                        continue;
                    }

                    visited.insert(child_entity);

                    // Calculate child's world transform
                    let child_world_matrix =
                        if let Ok(child_transform) = inner.get::<&Transform>(child_entity) {
                            let local_matrix = child_transform.to_matrix();
                            // Column-major matrix multiplication: parent * local
                            parent_world_matrix.mul(local_matrix)
                        } else {
                            parent_world_matrix
                        };

                    child_updates.push((child_entity, child_world_matrix));
                    next_level.push((child_entity, child_world_matrix));
                }
            }
        }

        // Update all children's global transforms
        for (child_entity, child_world_matrix) in child_updates {
            if let Ok(query) = inner.query_one_mut::<&mut GlobalTransform>(child_entity) {
                query.matrix = child_world_matrix;
            } else {
                let _ = inner.insert_one(
                    child_entity,
                    GlobalTransform::from_matrix(child_world_matrix),
                );
            }
        }

        // Swap buffers for next level
        std::mem::swap(&mut queue, &mut next_level);
    }

    debug!(
        processed_count = visited.len(),
        "Hierarchy update completed"
    );

    // Handle entities with Parent but no Transform (edge case)
    for (entity, parent) in world.query::<&Parent>().iter() {
        if world.get::<Transform>(entity).is_err() {
            error!(
                entity = ?entity,
                parent = ?parent.0,
                "Entity has Parent component but no Transform component"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    #[test]
    fn test_basic_hierarchy() {
        let mut world = World::new();

        // Create parent at position (1, 0, 0)
        let parent = world.spawn((
            Transform::from_position(Vec3::X),
            GlobalTransform::default(),
        ));

        // Create child at local position (0, 1, 0)
        let child = world.spawn((
            Transform::from_position(Vec3::Y),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Update hierarchy
        update_hierarchy_system(&mut world);

        // Parent should be at world (1, 0, 0)
        let parent_global = world.get::<GlobalTransform>(parent).unwrap();
        assert_eq!(parent_global.position(), Vec3::X);

        // Child should be at world (1, 1, 0)
        let child_global = world.get::<GlobalTransform>(child).unwrap();
        assert_eq!(child_global.position(), Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn test_multi_level_hierarchy() {
        let mut world = World::new();

        // Create grandparent at (1, 0, 0)
        let grandparent = world.spawn((
            Transform::from_position(Vec3::X),
            GlobalTransform::default(),
        ));

        // Create parent at local (0, 1, 0), world should be (1, 1, 0)
        let parent = world.spawn((
            Transform::from_position(Vec3::Y),
            GlobalTransform::default(),
            Parent(grandparent),
        ));

        // Create child at local (0, 0, 1), world should be (1, 1, 1)
        let child = world.spawn((
            Transform::from_position(Vec3::Z),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Update hierarchy
        update_hierarchy_system(&mut world);

        // Check positions
        let grandparent_global = world.get::<GlobalTransform>(grandparent).unwrap();
        assert_eq!(grandparent_global.position(), Vec3::X);

        let parent_global = world.get::<GlobalTransform>(parent).unwrap();
        assert_eq!(parent_global.position(), Vec3::new(1.0, 1.0, 0.0));

        let child_global = world.get::<GlobalTransform>(child).unwrap();
        assert_eq!(child_global.position(), Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_scale_propagation() {
        let mut world = World::new();

        // Create parent with 2x scale
        let parent = world.spawn((
            Transform {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(2.0),
            },
            GlobalTransform::default(),
        ));

        // Create child at local (1, 0, 0)
        let child = world.spawn((
            Transform::from_position(Vec3::X),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Update hierarchy
        update_hierarchy_system(&mut world);

        // Child should be at world (2, 0, 0) due to parent's scale
        let child_global = world.get::<GlobalTransform>(child).unwrap();
        assert_eq!(child_global.position(), Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_cycle_detection() {
        let mut world = World::new();

        // Create two entities
        let a = world.spawn((Transform::default(), GlobalTransform::default()));
        let b = world.spawn((Transform::default(), GlobalTransform::default(), Parent(a)));

        // Create cycle: a -> b -> a
        world.insert_one(a, Parent(b)).unwrap();

        // Should not panic, just log error
        update_hierarchy_system(&mut world);

        // Both entities should still exist
        assert!(world.contains(a));
        assert!(world.contains(b));
    }

    #[test]
    fn test_missing_transform_auto_add() {
        let mut world = World::new();

        // Create parent
        let parent = world.spawn((Transform::default(), GlobalTransform::default()));

        // Create child without GlobalTransform
        let child = world.spawn((Transform::default(), Parent(parent)));

        // Update hierarchy
        update_hierarchy_system(&mut world);

        // Child should now have GlobalTransform
        assert!(world.get::<GlobalTransform>(child).is_ok());
    }
}
