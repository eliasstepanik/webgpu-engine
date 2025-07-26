//! Test that physics objects settle at the correct positions

use engine::core::entity::World;
use engine::core::entity::{GlobalTransform, Transform};
use engine::physics::components::{Collider, CollisionShape, Rigidbody};
use engine::physics::simple_physics::simple_physics_update;
use glam::{Mat3, Vec3};

#[test]
fn test_sphere_on_ground_position() {
    let mut world = World::new();

    // Create ground at y=0, with half extent 0.5, so top surface is at y=0.5
    let _ground = world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(10.0, 0.5, 10.0),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Create sphere with radius 0.5, starting at y=2.0
    let sphere = world.spawn((
        Transform::from_position(Vec3::new(0.0, 2.0, 0.0)),
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.1, // Higher damping to settle faster
            angular_damping: 0.1,
            use_gravity: true,
            is_kinematic: false,
            inertia_tensor: Mat3::IDENTITY,
        },
        Collider {
            shape: CollisionShape::Sphere { radius: 0.5 },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Update hierarchy
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Simulate for 2 seconds with detailed output
    let dt = 0.016;
    let config = engine::physics::PhysicsConfig::default();
    for i in 0..125 {
        simple_physics_update(&mut world, &config, dt);

        let transform = world.get::<Transform>(sphere).unwrap();
        let velocity = world.get::<Rigidbody>(sphere).unwrap();

        // Print every 10 frames
        if i % 10 == 0 {
            println!(
                "Frame {}: y={:.6}, vel_y={:.6}",
                i, transform.position.y, velocity.linear_velocity.y
            );
        }

        // Check if settled (very low velocity and near expected position)
        if velocity.linear_velocity.length() < 0.001 && (transform.position.y - 1.0).abs() < 0.01 {
            println!("Settled at frame {} with y={:.6}", i, transform.position.y);
            break;
        }
    }

    // Final position check
    let final_transform = world.get::<Transform>(sphere).unwrap();
    let final_velocity = world.get::<Rigidbody>(sphere).unwrap();

    println!(
        "Final position: y={:.6}, velocity: {:?}",
        final_transform.position.y, final_velocity.linear_velocity
    );

    // Sphere center should be at y=1.0 (ground top 0.5 + sphere radius 0.5)
    let expected_y = 1.0;
    let position_error = (final_transform.position.y - expected_y).abs();

    assert!(
        position_error < 0.005,
        "Sphere should be at y={}, but is at y={} (error: {})",
        expected_y,
        final_transform.position.y,
        position_error
    );

    assert!(
        final_velocity.linear_velocity.length() < 0.001,
        "Sphere should have near-zero velocity, but has velocity={:?}",
        final_velocity.linear_velocity
    );
}

#[test]
fn test_box_on_ground_position() {
    let mut world = World::new();

    // Create ground at y=0
    let _ground = world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(10.0, 0.5, 10.0),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Create box with half extents 0.5 (1x1x1 box), starting at y=3.0
    let box_entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 3.0, 0.0)),
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.1,
            angular_damping: 0.1,
            use_gravity: true,
            is_kinematic: false,
            inertia_tensor: Mat3::IDENTITY,
        },
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::splat(0.5),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Update hierarchy
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Simulate for 2 seconds
    let dt = 0.016;
    let config = engine::physics::PhysicsConfig::default();
    for i in 0..125 {
        simple_physics_update(&mut world, &config, dt);

        if i % 25 == 0 {
            let transform = world.get::<Transform>(box_entity).unwrap();
            let velocity = world.get::<Rigidbody>(box_entity).unwrap();
            println!(
                "Frame {}: y={:.6}, vel_y={:.6}",
                i, transform.position.y, velocity.linear_velocity.y
            );
        }
    }

    // Final position check
    let final_transform = world.get::<Transform>(box_entity).unwrap();

    // Box center should be at y=1.0 (ground top 0.5 + box half extent 0.5)
    let expected_y = 1.0;
    let position_error = (final_transform.position.y - expected_y).abs();

    assert!(
        position_error < 0.005,
        "Box should be at y={}, but is at y={} (error: {})",
        expected_y,
        final_transform.position.y,
        position_error
    );
}
