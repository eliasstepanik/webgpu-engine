//! Test that physics objects rotate realistically

use engine::core::entity::World;
use engine::core::entity::{GlobalTransform, Transform};
use engine::physics::components::{Collider, CollisionShape, Rigidbody};
use engine::physics::simple_physics::simple_physics_update;
use glam::{EulerRot, Quat, Vec3};

#[test]
fn test_cube_tips_over() {
    let mut world = World::new();

    // Create ground
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

    // Create cube tilted on corner - rotate 45 degrees around X and Z
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        std::f32::consts::PI / 4.0, // 45 degrees around X
        0.0,
        std::f32::consts::PI / 4.0, // 45 degrees around Z
    );

    let cube = world.spawn((
        Transform {
            position: Vec3::new(-0.2, 1.2, 0.0), // Offset horizontally so it lands on edge
            rotation,
            scale: Vec3::ONE,
        },
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::new(0.0, -2.0, 0.0), // Initial downward velocity
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.05,
            angular_damping: 0.05,
            use_gravity: true,
            is_kinematic: false,
            inertia_tensor: CollisionShape::Box {
                half_extents: Vec3::splat(0.5),
            }
            .calculate_inertia(1.0),
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

    // Simulate for 3 seconds
    let dt = 0.016;
    let mut last_printed = 0;

    // Enable debug logging for first few frames
    std::env::set_var("RUST_LOG", "engine::physics=debug");
    let _ = tracing_subscriber::fmt::try_init();

    for i in 0..180 {
        if i < 5 {
            println!("\n=== Frame {i} ===");
        }
        simple_physics_update(&mut world, &engine::physics::PhysicsConfig::default(), dt);

        let transform = world.get::<Transform>(cube).unwrap();
        let velocity = world.get::<Rigidbody>(cube).unwrap();

        // Print every 30 frames
        if i - last_printed >= 30 {
            let (roll, pitch, yaw) = transform.rotation.to_euler(EulerRot::XYZ);
            println!(
                "Frame {}: y={:.3}, angular_vel={:.3}, rotation=(r:{:.1}°, p:{:.1}°, y:{:.1}°)",
                i,
                transform.position.y,
                velocity.angular_velocity.length(),
                roll.to_degrees(),
                pitch.to_degrees(),
                yaw.to_degrees()
            );
            last_printed = i;
        }

        // Check if cube has settled
        if velocity.linear_velocity.length() < 0.01 && velocity.angular_velocity.length() < 0.01 {
            println!("Cube settled at frame {i}");
            break;
        }
    }

    // Final check - cube should have rotated to rest on a face
    let final_transform = world.get::<Transform>(cube).unwrap();

    // Check that cube is resting on a face by checking if one of the major axes is mostly vertical
    let up = final_transform.rotation * Vec3::Y;
    let right = final_transform.rotation * Vec3::X;
    let forward = final_transform.rotation * Vec3::Z;

    // At least one axis should be nearly vertical (aligned with world Y)
    let max_alignment = up.y.abs().max(right.y.abs()).max(forward.y.abs());

    assert!(
        max_alignment > 0.9,
        "Cube should have rotated to rest on a face, but max axis alignment with Y is only {max_alignment:.3}"
    );

    println!("Final orientation - Up: {up:?}, Right: {right:?}, Forward: {forward:?}");
}

#[test]
fn test_angular_momentum_conservation() {
    let mut world = World::new();

    // Create a box spinning in free space (no gravity)
    let box_entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 5.0, 0.0)),
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::new(0.0, 2.0, 0.0), // Spinning around Y axis
            linear_damping: 0.0,                        // No damping
            angular_damping: 0.0,                       // No damping
            use_gravity: false,                         // No gravity
            is_kinematic: false,
            inertia_tensor: CollisionShape::Box {
                half_extents: Vec3::splat(0.5),
            }
            .calculate_inertia(1.0),
        },
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::splat(0.5),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Initial angular velocity
    let initial_angular_vel = 2.0;

    // Simulate for 1 second
    let dt = 0.016;
    for _ in 0..60 {
        simple_physics_update(&mut world, &engine::physics::PhysicsConfig::default(), dt);
    }

    // Check angular momentum is conserved
    let final_rb = world.get::<Rigidbody>(box_entity).unwrap();
    let final_angular_speed = final_rb.angular_velocity.length();

    assert!(
        (final_angular_speed - initial_angular_vel).abs() < 0.01,
        "Angular momentum should be conserved, but speed changed from {initial_angular_vel} to {final_angular_speed}"
    );
}
