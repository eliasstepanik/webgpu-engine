//! Test simple physics integration

use engine::core::entity::World;
use engine::physics::{components::Rigidbody, simple_physics::simple_physics_update};
use engine::prelude::Transform;
use glam::{Mat3, Vec3};
use tracing::info;

#[test]
fn test_simple_physics_integration() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // Create world
    let mut world = World::new();

    // Create falling cube
    let cube = world.spawn((
        Transform {
            position: Vec3::new(0.0, 10.0, 0.0),
            rotation: glam::Quat::IDENTITY,
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
    ));

    info!("Created cube at y=10.0");

    // Run physics for 1 second (60 steps at 60Hz)
    let dt = 1.0 / 60.0;
    for i in 0..60 {
        // Get position before update
        let _pos_before = world.get::<Transform>(cube).unwrap().position;
        let _vel_before = world.get::<Rigidbody>(cube).unwrap().linear_velocity;

        // Run physics
        simple_physics_update(&mut world, &engine::physics::PhysicsConfig::default(), dt);

        // Get position after update
        let pos_after = world.get::<Transform>(cube).unwrap().position;
        let vel_after = world.get::<Rigidbody>(cube).unwrap().linear_velocity;

        if i % 10 == 0 || i < 5 {
            info!(
                "Step {}: pos y={:.4}, vel y={:.4}",
                i, pos_after.y, vel_after.y
            );
        }
    }

    // After 1 second of falling:
    // Expected position: y = 10 + 0*1 + 0.5*(-9.81)*1² = 10 - 4.905 = 5.095
    // Expected velocity: v = 0 + (-9.81)*1 = -9.81

    let final_pos = world.get::<Transform>(cube).unwrap().position;
    let final_vel = world.get::<Rigidbody>(cube).unwrap().linear_velocity;

    info!("Final position: y={:.4}", final_pos.y);
    info!("Final velocity: y={:.4}", final_vel.y);

    assert!(
        (final_pos.y - 5.095).abs() < 0.1,
        "Body should have fallen to around y=5.095, but is at y={}",
        final_pos.y
    );
    assert!(
        (final_vel.y + 9.81).abs() < 0.1,
        "Body should have velocity around -9.81, but has {}",
        final_vel.y
    );
}
