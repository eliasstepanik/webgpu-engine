//! Hierarchy system for updating global transforms based on parent relationships

use super::components::{GlobalTransform, GlobalWorldTransform, Parent, Transform, WorldTransform};
use super::world::World;
use crate::core::camera::{Camera, CameraWorldPosition};
// hecs::Entity is re-exported from world module
use glam::DVec3;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, trace};

// Frame tracking to prevent multiple hierarchy updates per frame
static LAST_HIERARCHY_UPDATE_FRAME: AtomicU64 = AtomicU64::new(0);
static CURRENT_FRAME: AtomicU64 = AtomicU64::new(0);

/// Advance to the next frame. Should be called once per frame by the main loop.
pub fn advance_frame() {
    CURRENT_FRAME.fetch_add(1, Ordering::SeqCst);
}

/// Reset frame counters - only for testing
#[cfg(test)]
pub fn reset_frame_counters() {
    CURRENT_FRAME.store(0, Ordering::SeqCst);
    LAST_HIERARCHY_UPDATE_FRAME.store(0, Ordering::SeqCst);
}

/// Update the hierarchy system, calculating global transforms from local transforms
/// and parent relationships using breadth-first traversal.
///
/// This system handles both regular Transform and high-precision WorldTransform components,
/// supporting mixed hierarchies where entities can use different transform types.
pub fn update_hierarchy_system(world: &mut World) {
    let current_frame = CURRENT_FRAME.load(Ordering::SeqCst);
    let last_update = LAST_HIERARCHY_UPDATE_FRAME.load(Ordering::SeqCst);

    if current_frame == last_update {
        trace!("Skipping hierarchy update - already updated this frame");
        return;
    }

    LAST_HIERARCHY_UPDATE_FRAME.store(current_frame, Ordering::SeqCst);
    trace!("Updating hierarchy for frame {}", current_frame);

    update_regular_hierarchy(world);
    update_world_hierarchy(world);
}

/// Update hierarchy for regular Transform components
fn update_regular_hierarchy(world: &mut World) {
    // Pre-allocate collections for performance
    let mut queue = Vec::with_capacity(1024);
    let mut visited = HashSet::with_capacity(1024);
    let mut next_level = Vec::new();

    let inner = world.inner_mut();

    // Find root entities (entities with Transform but no Parent)
    let mut root_updates = Vec::new();
    for (entity, (transform,)) in inner.query::<(&Transform,)>().without::<&Parent>().iter() {
        let world_matrix = transform.to_matrix();
        root_updates.push((entity, world_matrix));
        visited.insert(entity);
    }

    // Update root entities' global transforms
    for (entity, world_matrix) in &root_updates {
        match inner.query_one_mut::<&mut GlobalTransform>(*entity) {
            Ok(query) => {
                query.matrix = *world_matrix;
            }
            Err(_) => {
                let _ = inner.insert_one(*entity, GlobalTransform::from_matrix(*world_matrix));
            }
        }

        // Update CameraWorldPosition for cameras to maintain exact position
        if inner.get::<&Camera>(*entity).is_ok() {
            let world_pos = world_matrix.w_axis.truncate();
            let world_pos_f64 =
                DVec3::new(world_pos.x as f64, world_pos.y as f64, world_pos.z as f64);

            match inner.query_one_mut::<&mut CameraWorldPosition>(*entity) {
                Ok(query) => {
                    query.position = world_pos_f64;
                }
                Err(_) => {
                    let _ = inner.insert_one(*entity, CameraWorldPosition::new(world_pos_f64));
                }
            }
        }
    }

    queue.extend(root_updates);
    trace!(
        root_count = queue.len(),
        "Starting regular hierarchy update"
    );

    // BFS traversal for Transform entities
    while !queue.is_empty() {
        let mut child_updates = Vec::new();

        for (parent_entity, parent_world_matrix) in queue.drain(..) {
            // Find children of this entity that also use Transform
            for (child_entity, parent) in inner.query::<&Parent>().iter() {
                if parent.0 == parent_entity {
                    // Only process children that have Transform (not WorldTransform)
                    if inner.get::<&Transform>(child_entity).is_err() {
                        continue;
                    }

                    if visited.contains(&child_entity) {
                        error!(
                            parent = ?parent_entity,
                            child = ?child_entity,
                            "Cyclic parent-child relationship detected in regular hierarchy"
                        );
                        continue;
                    }

                    visited.insert(child_entity);

                    let child_world_matrix =
                        if let Ok(child_transform) = inner.get::<&Transform>(child_entity) {
                            let local_matrix = child_transform.to_matrix();
                            parent_world_matrix * local_matrix
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
            match inner.query_one_mut::<&mut GlobalTransform>(child_entity) {
                Ok(query) => {
                    query.matrix = child_world_matrix;
                }
                Err(_) => {
                    let _ = inner.insert_one(
                        child_entity,
                        GlobalTransform::from_matrix(child_world_matrix),
                    );
                }
            }

            // Update CameraWorldPosition for cameras to maintain exact position
            if inner.get::<&Camera>(child_entity).is_ok() {
                let world_pos = child_world_matrix.w_axis.truncate();
                let world_pos_f64 =
                    DVec3::new(world_pos.x as f64, world_pos.y as f64, world_pos.z as f64);

                // Debug logging for camera position updates
                if world_pos.length() > 10000.0 {
                    trace!(
                        entity = ?child_entity,
                        world_pos = ?world_pos,
                        world_pos_f64 = ?world_pos_f64,
                        "Updating camera position far from origin"
                    );
                }

                match inner.query_one_mut::<&mut CameraWorldPosition>(child_entity) {
                    Ok(query) => {
                        query.position = world_pos_f64;
                    }
                    Err(_) => {
                        let _ =
                            inner.insert_one(child_entity, CameraWorldPosition::new(world_pos_f64));
                    }
                }
            }
        }

        std::mem::swap(&mut queue, &mut next_level);
    }

    trace!(
        processed_count = visited.len(),
        "Regular hierarchy update completed"
    );
}

/// Update hierarchy for high-precision WorldTransform components
fn update_world_hierarchy(world: &mut World) {
    let mut queue = Vec::with_capacity(1024);
    let mut visited = HashSet::with_capacity(1024);
    let mut next_level = Vec::new();

    let inner = world.inner_mut();

    // Find root entities (entities with WorldTransform but no Parent)
    let mut root_updates = Vec::new();
    for (entity, (world_transform,)) in inner
        .query::<(&WorldTransform,)>()
        .without::<&Parent>()
        .iter()
    {
        let world_matrix = world_transform.to_matrix();
        root_updates.push((entity, world_matrix));
        visited.insert(entity);
    }

    // Update root entities' global world transforms
    for (entity, world_matrix) in &root_updates {
        match inner.query_one_mut::<&mut GlobalWorldTransform>(*entity) {
            Ok(query) => {
                query.matrix = *world_matrix;
            }
            Err(_) => {
                let _ = inner.insert_one(*entity, GlobalWorldTransform::from_matrix(*world_matrix));
            }
        }
    }

    queue.extend(root_updates);
    trace!(root_count = queue.len(), "Starting world hierarchy update");

    // BFS traversal for WorldTransform entities
    while !queue.is_empty() {
        let mut child_updates = Vec::new();

        for (parent_entity, parent_world_matrix) in queue.drain(..) {
            // Find children of this entity
            for (child_entity, parent) in inner.query::<&Parent>().iter() {
                if parent.0 == parent_entity {
                    // Process children with WorldTransform or mixed hierarchies
                    if visited.contains(&child_entity) {
                        error!(
                            parent = ?parent_entity,
                            child = ?child_entity,
                            "Cyclic parent-child relationship detected in world hierarchy"
                        );
                        continue;
                    }

                    visited.insert(child_entity);

                    let child_world_matrix = if let Ok(child_world_transform) =
                        inner.get::<&WorldTransform>(child_entity)
                    {
                        // Child has WorldTransform - use high precision math
                        let local_matrix = child_world_transform.to_matrix();
                        parent_world_matrix * local_matrix
                    } else if let Ok(child_transform) = inner.get::<&Transform>(child_entity) {
                        // Mixed hierarchy: parent has WorldTransform, child has Transform
                        let local_matrix = child_transform.to_matrix().as_dmat4();
                        parent_world_matrix * local_matrix
                    } else {
                        // Child has no transform - inherit parent
                        parent_world_matrix
                    };

                    child_updates.push((child_entity, child_world_matrix));
                    next_level.push((child_entity, child_world_matrix));
                }
            }
        }

        // Update children's global transforms based on their transform type
        for (child_entity, child_world_matrix) in child_updates {
            if inner.get::<&WorldTransform>(child_entity).is_ok() {
                // Child has WorldTransform - update GlobalWorldTransform
                match inner.query_one_mut::<&mut GlobalWorldTransform>(child_entity) {
                    Ok(query) => {
                        query.matrix = child_world_matrix;
                    }
                    Err(_) => {
                        let _ = inner.insert_one(
                            child_entity,
                            GlobalWorldTransform::from_matrix(child_world_matrix),
                        );
                    }
                }
            } else if inner.get::<&Transform>(child_entity).is_ok() {
                // Mixed hierarchy: update regular GlobalTransform with downgraded precision
                let regular_matrix = child_world_matrix.as_mat4();
                match inner.query_one_mut::<&mut GlobalTransform>(child_entity) {
                    Ok(query) => {
                        query.matrix = regular_matrix;
                    }
                    Err(_) => {
                        let _ = inner
                            .insert_one(child_entity, GlobalTransform::from_matrix(regular_matrix));
                    }
                }
            }
        }

        std::mem::swap(&mut queue, &mut next_level);
    }

    trace!(
        processed_count = visited.len(),
        "World hierarchy update completed"
    );
}

/// Helper function to validate hierarchy consistency
pub fn validate_hierarchy_system(world: &World) {
    let mut issues = 0;

    // Check for entities with Parent but no Transform of any kind
    for (entity, parent) in world.query::<&Parent>().iter() {
        if world.get::<Transform>(entity).is_err() && world.get::<WorldTransform>(entity).is_err() {
            error!(
                entity = ?entity,
                parent = ?parent.0,
                "Entity has Parent component but no Transform or WorldTransform component"
            );
            issues += 1;
        }
    }

    if issues > 0 {
        error!("Found {} hierarchy validation issues", issues);
    } else {
        trace!("Hierarchy validation passed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    #[test]
    fn test_basic_hierarchy() {
        reset_frame_counters();
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
        advance_frame();
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
        reset_frame_counters();
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
        advance_frame();
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
        reset_frame_counters();
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
        advance_frame();
        update_hierarchy_system(&mut world);

        // Child should be at world (2, 0, 0) due to parent's scale
        let child_global = world.get::<GlobalTransform>(child).unwrap();
        assert_eq!(child_global.position(), Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_cycle_detection() {
        reset_frame_counters();
        let mut world = World::new();

        // Create two entities
        let a = world.spawn((Transform::default(), GlobalTransform::default()));
        let b = world.spawn((Transform::default(), GlobalTransform::default(), Parent(a)));

        // Create cycle: a -> b -> a
        world.insert_one(a, Parent(b)).unwrap();

        // Should not panic, just log error
        advance_frame();
        update_hierarchy_system(&mut world);

        // Both entities should still exist
        assert!(world.contains(a));
        assert!(world.contains(b));
    }

    #[test]
    fn test_missing_transform_auto_add() {
        reset_frame_counters();
        let mut world = World::new();

        // Create parent
        let parent = world.spawn((Transform::default(), GlobalTransform::default()));

        // Create child without GlobalTransform
        let child = world.spawn((Transform::default(), Parent(parent)));

        // Update hierarchy
        advance_frame();
        update_hierarchy_system(&mut world);

        // Child should now have GlobalTransform
        assert!(world.get::<GlobalTransform>(child).is_ok());
    }

    #[test]
    fn test_camera_parenting_no_drift() {
        reset_frame_counters();
        let mut world = World::new();

        // Create camera with specific position and rotation
        let camera_pos = Vec3::new(10.5, 25.3, -15.7);
        let camera_rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4); // 45 degrees

        let camera = world.spawn((
            Transform::from_position_rotation(camera_pos, camera_rot),
            GlobalTransform::default(),
            crate::core::camera::Camera::default(),
        ));

        // Create parent at different position
        let parent = world.spawn((
            Transform::from_position(Vec3::new(-5.0, 10.0, 20.0)),
            GlobalTransform::default(),
        ));

        // Update hierarchy to establish initial state
        advance_frame();
        update_hierarchy_system(&mut world);

        // Store original world position
        let original_world_pos = world.get::<GlobalTransform>(camera).unwrap().position();

        // Verify CameraWorldPosition was created and matches
        {
            let cam_world_pos = world.get::<CameraWorldPosition>(camera).unwrap();
            assert!(
                (cam_world_pos.position.as_vec3() - original_world_pos).length() < 0.0001,
                "CameraWorldPosition doesn't match GlobalTransform position"
            );
        }

        // Calculate new local position to maintain world position when parenting
        let new_local_position = {
            let parent_world_matrix = world.get::<GlobalTransform>(parent).unwrap().matrix;
            let camera_world_pos = world.get::<GlobalTransform>(camera).unwrap().position();
            let parent_world_pos = parent_world_matrix.w_axis.truncate();

            // For simple case with no parent rotation/scale, local position = world position - parent world position
            // This is a simplified calculation that assumes parent has no rotation/scale
            camera_world_pos - parent_world_pos
        };

        // Update camera's local transform
        if let Ok(query) = world.query_one_mut::<&mut Transform>(camera) {
            query.position = new_local_position;
            // Keep existing rotation and scale
        }

        // Parent the camera
        world.insert_one(camera, Parent(parent)).unwrap();

        // Update hierarchy once
        advance_frame();
        update_hierarchy_system(&mut world);

        // Verify world position hasn't changed
        let new_world_pos = world.get::<GlobalTransform>(camera).unwrap().position();

        assert!(
            (new_world_pos - original_world_pos).length() < 0.0001,
            "Camera drifted by {} units",
            (new_world_pos - original_world_pos).length()
        );

        // Verify CameraWorldPosition is still accurate
        {
            let new_cam_world_pos = world.get::<CameraWorldPosition>(camera).unwrap();
            assert!(
                (new_cam_world_pos.position.as_vec3() - new_world_pos).length() < 0.0001,
                "CameraWorldPosition doesn't match GlobalTransform position after parenting"
            );
        }
    }

    #[test]
    fn test_multiple_hierarchy_updates_no_accumulation() {
        reset_frame_counters();
        let mut world = World::new();

        // Create camera
        let camera = world.spawn((
            Transform::from_position(Vec3::new(100.0, 50.0, 25.0)),
            GlobalTransform::default(),
            crate::core::camera::Camera::default(),
        ));

        // Create parent
        let parent = world.spawn((
            Transform::from_position(Vec3::new(10.0, 5.0, 2.0)),
            GlobalTransform::default(),
        ));

        // Initial update
        advance_frame();
        update_hierarchy_system(&mut world);
        let _initial_pos = world.get::<GlobalTransform>(camera).unwrap().position();

        // Parent the camera
        world.insert_one(camera, Parent(parent)).unwrap();

        // Update hierarchy multiple times to simulate multiple updates per frame
        advance_frame();
        update_hierarchy_system(&mut world);
        let pos_after_first = world.get::<GlobalTransform>(camera).unwrap().position();

        // With frame tracking, subsequent updates should be skipped
        update_hierarchy_system(&mut world);
        update_hierarchy_system(&mut world);
        update_hierarchy_system(&mut world);

        let pos_after_multiple = world.get::<GlobalTransform>(camera).unwrap().position();

        // Position should be the same after multiple updates
        assert!(
            (pos_after_first - pos_after_multiple).length() < f32::EPSILON,
            "Position changed after multiple updates: drift = {}",
            (pos_after_first - pos_after_multiple).length()
        );
    }

    #[test]
    fn test_camera_hierarchy_precision() {
        reset_frame_counters();
        let mut world = World::new();

        // Test with precise fractional positions
        let camera_pos = Vec3::new(1234.5678, 9876.543, -4567.89);

        let camera = world.spawn((
            Transform::from_position(camera_pos),
            GlobalTransform::default(),
            crate::core::camera::Camera::default(),
        ));

        let parent = world.spawn((
            Transform::from_position(Vec3::new(111.111, 222.222, 333.333)),
            GlobalTransform::default(),
        ));

        // Update to establish initial positions
        advance_frame();
        update_hierarchy_system(&mut world);
        let original_world_pos = world.get::<GlobalTransform>(camera).unwrap().position();

        // Parent and update
        world.insert_one(camera, Parent(parent)).unwrap();
        advance_frame();
        update_hierarchy_system(&mut world);

        // Unparent and update
        world.inner_mut().remove_one::<Parent>(camera).ok();
        advance_frame();
        update_hierarchy_system(&mut world);

        let final_world_pos = world.get::<GlobalTransform>(camera).unwrap().position();

        // After parenting and unparenting, camera should return to exact original position
        assert!(
            (final_world_pos - original_world_pos).length() < 0.0001,
            "Camera position not restored after parent/unparent cycle: drift = {}",
            (final_world_pos - original_world_pos).length()
        );
    }

    #[test]
    fn test_camera_parenting_drift_at_large_distances() {
        reset_frame_counters();
        let mut world = World::new();

        // Create a cube at origin
        let cube = world.spawn((Transform::default(), GlobalTransform::default()));

        // Create a camera offset from cube
        let camera = world.spawn((
            Transform::from_position(Vec3::new(5.0, 5.0, 5.0)),
            GlobalTransform::default(),
            Camera::default(),
        ));

        // Update hierarchy to establish initial positions
        advance_frame();
        update_hierarchy_system(&mut world);

        // Get initial positions
        let initial_cube_pos = world.get::<GlobalTransform>(cube).unwrap().position();
        let initial_camera_pos = world.get::<GlobalTransform>(camera).unwrap().position();
        let initial_relative_pos = initial_camera_pos - initial_cube_pos;

        // Parent camera to cube (properly calculate local transform)
        let child_world_matrix = world.get::<GlobalTransform>(camera).unwrap().matrix;
        let parent_world_matrix = world.get::<GlobalTransform>(cube).unwrap().matrix;

        // Calculate local transform
        let parent_inverse = parent_world_matrix.inverse();
        let new_local_matrix = parent_inverse * child_world_matrix;
        let (scale, rotation, translation) = new_local_matrix.to_scale_rotation_translation();

        // Update camera's local transform
        if let Ok(cam_transform) = world.query_one_mut::<&mut Transform>(camera) {
            cam_transform.position = translation;
            cam_transform.rotation = rotation;
            cam_transform.scale = scale;
        }

        // Parent the camera
        world.insert_one(camera, Parent(cube)).unwrap();

        // Update hierarchy
        advance_frame();
        update_hierarchy_system(&mut world);

        // Move cube far from origin
        if let Ok(cube_transform) = world.query_one_mut::<&mut Transform>(cube) {
            cube_transform.position = Vec3::new(100000.0, 0.0, 0.0);
        }

        // Update hierarchy
        advance_frame();
        update_hierarchy_system(&mut world);

        // Check positions
        let cube_pos = world.get::<GlobalTransform>(cube).unwrap().position();
        let camera_pos = world.get::<GlobalTransform>(camera).unwrap().position();
        let relative_pos = camera_pos - cube_pos;
        let drift = (relative_pos - initial_relative_pos).length();

        assert!(
            drift < 0.01,
            "Camera drifted {drift} units at large distance (relative pos changed from {initial_relative_pos:?} to {relative_pos:?})"
        );

        // Also check CameraWorldPosition
        let cam_world_pos = world.get::<CameraWorldPosition>(camera).unwrap();
        let expected_pos = DVec3::new(100005.0, 5.0, 5.0);
        let pos_drift = (cam_world_pos.position - expected_pos).length();
        assert!(
            pos_drift < 0.01,
            "CameraWorldPosition drifted {pos_drift} units"
        );
    }

    #[test]
    fn test_parented_transforms_maintain_relative_positions() {
        reset_frame_counters();
        let mut world = World::new();

        // Create parent and child entities
        let parent = world.spawn((Transform::default(), GlobalTransform::default()));

        let child = world.spawn((
            Transform::from_position(Vec3::new(10.0, 10.0, 10.0)),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Update hierarchy
        advance_frame();
        update_hierarchy_system(&mut world);

        // Get initial relative position
        let parent_pos = world.get::<GlobalTransform>(parent).unwrap().position();
        let child_pos = world.get::<GlobalTransform>(child).unwrap().position();
        let initial_relative = child_pos - parent_pos;

        // Move parent multiple times to different positions
        let test_positions = vec![
            Vec3::new(1000.0, 0.0, 0.0),
            Vec3::new(50000.0, 1000.0, -2000.0),
            Vec3::new(100000.0, -5000.0, 50000.0),
            Vec3::new(-75000.0, 25000.0, -80000.0),
        ];

        for test_pos in test_positions {
            // Move parent
            world
                .query_one_mut::<&mut Transform>(parent)
                .unwrap()
                .position = test_pos;

            // Update hierarchy
            advance_frame();
            update_hierarchy_system(&mut world);

            // Check relative position is maintained
            let parent_pos = world.get::<GlobalTransform>(parent).unwrap().position();
            let child_pos = world.get::<GlobalTransform>(child).unwrap().position();
            let current_relative = child_pos - parent_pos;

            let drift = (current_relative - initial_relative).length();
            assert!(
                drift < 0.001,
                "Relative position drifted by {drift} at parent position {test_pos:?} (expected {initial_relative:?}, got {current_relative:?})"
            );
        }
    }

    #[test]
    fn test_frame_tracking_prevents_multiple_updates() {
        reset_frame_counters();
        let mut world = World::new();

        // Create a simple entity
        let entity = world.spawn((
            Transform::from_position(Vec3::new(1.0, 2.0, 3.0)),
            GlobalTransform::default(),
        ));

        // Advance frame
        advance_frame();

        // First update should proceed
        update_hierarchy_system(&mut world);
        let pos1 = world.get::<GlobalTransform>(entity).unwrap().position();

        // Modify transform
        world
            .query_one_mut::<&mut Transform>(entity)
            .unwrap()
            .position = Vec3::new(4.0, 5.0, 6.0);

        // Second update in same frame should be skipped
        update_hierarchy_system(&mut world);
        let pos2 = world.get::<GlobalTransform>(entity).unwrap().position();

        // Position should not have changed because update was skipped
        assert_eq!(pos1, pos2, "Hierarchy updated multiple times in same frame");

        // Advance to next frame
        advance_frame();

        // Now update should proceed
        update_hierarchy_system(&mut world);
        let pos3 = world.get::<GlobalTransform>(entity).unwrap().position();

        // Position should now be updated
        assert_ne!(pos1, pos3, "Hierarchy not updated in new frame");
        assert_eq!(pos3, Vec3::new(4.0, 5.0, 6.0));
    }
}
