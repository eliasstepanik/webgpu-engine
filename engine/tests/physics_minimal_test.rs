//! Minimal physics test to debug issues

use engine::core::entity::{update_hierarchy_system, GlobalTransform, World};
use engine::physics::{
    components::{Collider, CollisionShape, Rigidbody},
    systems::{create_default_solver, update_physics_system},
    PhysicsConfig,
};
use engine::prelude::Transform;
use glam::{Mat3, Quat, Vec3};
use tracing::{debug, info};

#[test]
fn test_minimal_physics() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    info!("Starting minimal physics test");

    // Create world
    let mut world = World::new();

    // Create floor entity (static collider, no rigidbody)
    let floor = world.spawn((
        Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(10.0, 1.0, 10.0),
        },
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(0.5, 0.5, 0.5),
            },
            is_trigger: false,
            material_id: Some(0),
        },
    ));

    info!("Created floor entity: {:?}", floor);

    // Check immediately after creation
    if world
        .query_one::<&Rigidbody>(floor)
        .map(|mut q| q.get().is_some())
        .unwrap_or(false)
    {
        panic!("Floor has rigidbody immediately after creation!");
    }

    // Create falling cube (dynamic rigidbody)
    let cube = world.spawn((
        Transform {
            position: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Rigidbody {
            mass: 1.0,
            linear_damping: 0.01,
            angular_damping: 0.01,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            inertia_tensor: Mat3::from_diagonal(Vec3::splat(0.16666667)),
            use_gravity: true,
            is_kinematic: false,
        },
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(0.5, 0.5, 0.5),
            },
            is_trigger: false,
            material_id: Some(0),
        },
    ));

    info!("Created cube entity: {:?}", cube);

    // Run hierarchy system to create GlobalTransform
    engine::core::entity::hierarchy::advance_frame();
    update_hierarchy_system(&mut world);

    // Count components
    let rigidbody_count = world.query::<&Rigidbody>().iter().count();
    let collider_count = world.query::<&Collider>().iter().count();
    let transform_count = world.query::<&Transform>().iter().count();

    info!("Component counts:");
    info!("  Rigidbodies: {}", rigidbody_count);
    info!("  Colliders: {}", collider_count);
    info!("  Transforms: {}", transform_count);

    // Verify GlobalTransform was created
    let global_transform_count = world.query::<&GlobalTransform>().iter().count();
    info!("  GlobalTransforms: {}", global_transform_count);

    assert_eq!(rigidbody_count, 1, "Should have 1 rigidbody");
    assert_eq!(collider_count, 2, "Should have 2 colliders");
    assert_eq!(transform_count, 2, "Should have 2 transforms");
    assert_eq!(global_transform_count, 2, "Should have 2 global transforms");

    // Debug: Check which entity has the rigidbody
    if world
        .query_one::<&Rigidbody>(floor)
        .map(|mut q| q.get().is_some())
        .unwrap_or(false)
    {
        panic!("Floor should not have a rigidbody!");
    }
    if !world
        .query_one::<&Rigidbody>(cube)
        .map(|mut q| q.get().is_some())
        .unwrap_or(false)
    {
        panic!("Cube should have a rigidbody!");
    }

    // Create physics solver and config
    let config = PhysicsConfig::default();
    let mut solver = create_default_solver();

    // Get floor collider info
    {
        let floor_transform = world.get::<Transform>(floor).unwrap();
        let floor_global = world.get::<GlobalTransform>(floor).unwrap();
        info!(
            "Floor transform: pos={:?}, scale={:?}",
            floor_transform.position, floor_transform.scale
        );
        info!("Floor global transform: {:?}", floor_global.matrix);
    }

    // Run physics for several steps
    for i in 0..100 {
        debug!("Physics step {}", i);

        // Get cube position before update
        let cube_pos_before = world.get::<Transform>(cube).unwrap().position;

        // Run physics
        update_physics_system(&mut world, &mut solver, &config, 0.016);

        // Get cube position after update
        let cube_pos_after = world.get::<Transform>(cube).unwrap().position;

        // Log every 10th step or when collision should happen
        if i % 10 == 0 || cube_pos_after.y < 1.5 {
            info!("Step {}: Cube at y={:.4}", i, cube_pos_after.y);
        }

        // Check if cube moved
        if (cube_pos_before.y - cube_pos_after.y).abs() > 0.001 {
            debug!(
                "  Cube moved from y={:.4} to y={:.4}",
                cube_pos_before.y, cube_pos_after.y
            );
        }
    }

    // Final check - cube should have fallen
    let final_pos = world.get::<Transform>(cube).unwrap().position;
    info!("Final cube position: {:?}", final_pos);

    // Cube should have fallen (y < 5.0) but not through floor (y > 0.5)
    assert!(final_pos.y < 5.0, "Cube should have fallen");
    assert!(final_pos.y > 0.4, "Cube should not fall through floor");
}
