//! Debug test to investigate collision detection issues

use engine::core::entity::{components::GlobalTransform, Transform, World};
use engine::physics::{
    components::{Collider, CollisionShape, Rigidbody},
    systems::{create_default_solver, update_physics_system_avbd},
};
use glam::Vec3;
use tracing::{debug, info};

#[test]
fn test_collision_detection_pipeline() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_env_filter("engine::physics=trace")
        .try_init();

    let mut world = World::new();
    let mut solver = create_default_solver();

    // Create floor - static collider
    let floor = world.spawn((
        Transform {
            position: Vec3::new(0.0, -1.0, 0.0),
            scale: Vec3::new(20.0, 0.2, 20.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(0.5, 0.5, 0.5),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Create falling box - dynamic rigidbody
    let falling_box = world.spawn((
        Transform {
            position: Vec3::new(0.0, 5.0, 0.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            ..Default::default()
        },
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(0.5, 0.5, 0.5),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    info!("Created floor entity: {:?}", floor);
    info!("Created falling box entity: {:?}", falling_box);

    // Update hierarchy to compute GlobalTransforms
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Verify GlobalTransforms were computed
    {
        let floor_global = world.get::<GlobalTransform>(floor).unwrap();
        let floor_pos = floor_global.matrix.to_scale_rotation_translation().2;
        info!("Floor global position: {:?}", floor_pos);

        let box_global = world.get::<GlobalTransform>(falling_box).unwrap();
        let box_pos = box_global.matrix.to_scale_rotation_translation().2;
        info!("Falling box global position: {:?}", box_pos);
    }

    // Run physics for several frames
    for frame in 0..100 {
        // Update physics
        update_physics_system_avbd(&mut world, &mut solver, 0.016);

        // Update hierarchy again to propagate transform changes
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);

        // Check positions
        let box_transform = world.get::<Transform>(falling_box).unwrap();
        let box_global = world.get::<GlobalTransform>(falling_box).unwrap();
        let box_world_pos = box_global.matrix.to_scale_rotation_translation().2;

        debug!(
            "Frame {}: box local pos = {:?}, world pos = {:?}",
            frame, box_transform.position, box_world_pos
        );

        // The box should stop falling when it hits the floor at around y = -0.3
        // (floor is at y = -1.0 with scale.y = 0.2, so top surface is at y = -0.8)
        // (box half extents are 0.5, so bottom of box should be at y = -0.3)
        if frame > 50 && box_world_pos.y < -0.5 {
            panic!("Box fell through floor! Frame {frame}, position: {box_world_pos:?}");
        }

        // Check if collision was detected
        if frame == 50 {
            let expected_y = -0.3; // Where the box should rest
            let tolerance = 0.1;
            assert!(
                (box_world_pos.y - expected_y).abs() < tolerance,
                "Box should have collided with floor by frame 50. Expected y ~= {}, got {}",
                expected_y,
                box_world_pos.y
            );
        }
    }

    info!("Test completed successfully - collision detection working");
}

#[test]
fn test_scaled_collider_aabb() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("engine::physics=trace")
        .try_init();

    let mut world = World::new();

    // Create a box with non-uniform scale
    let entity = world.spawn((
        Transform {
            position: Vec3::ZERO,
            scale: Vec3::new(20.0, 0.2, 20.0),
            ..Default::default()
        },
        GlobalTransform::default(),
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(0.5, 0.5, 0.5),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Update hierarchy
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Get the computed global transform
    let global_transform = world.get::<GlobalTransform>(entity).unwrap();
    let (scale, rotation, position) = global_transform.matrix.to_scale_rotation_translation();

    info!("Scale: {:?}", scale);
    info!("Position: {:?}", position);

    // Compute scaled AABB
    let collider = world.get::<Collider>(entity).unwrap();
    let scaled_shape = match &collider.shape {
        CollisionShape::Box { half_extents } => CollisionShape::Box {
            half_extents: *half_extents * scale,
        },
        _ => panic!("Expected box shape"),
    };

    let aabb = scaled_shape.world_aabb(position, rotation);
    info!("AABB: min={:?}, max={:?}", aabb.min, aabb.max);

    // Verify the AABB is correct
    // With scale (20, 0.2, 20) and half_extents (0.5, 0.5, 0.5)
    // The scaled half_extents should be (10, 0.1, 10)
    assert!((aabb.min - Vec3::new(-10.0, -0.1, -10.0)).length() < 0.01);
    assert!((aabb.max - Vec3::new(10.0, 0.1, 10.0)).length() < 0.01);
}
