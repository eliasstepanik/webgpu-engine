//! Direct test of torque generation

use engine::core::entity::World;
use engine::core::entity::{GlobalTransform, Transform};
use engine::physics::components::{Collider, CollisionShape, Rigidbody};
use engine::physics::simple_physics::simple_physics_update;
use glam::{EulerRot, Quat, Vec3};

#[test]
fn test_torque_from_offset_impact() {
    let mut world = World::new();

    // Enable debug logging
    std::env::set_var("RUST_LOG", "engine::physics=debug");
    let _ = tracing_subscriber::fmt::try_init();

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

    // Create box slightly above ground, will fall and hit on edge
    let box_entity = world.spawn((
        Transform {
            position: Vec3::new(0.0, 0.8, 0.0), // Very close to ground
            rotation: Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.2), // More tilt
            scale: Vec3::ONE,
        },
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::new(0.0, -3.0, 0.0), // Falling
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.0, // No damping for this test
            angular_damping: 0.0,
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

    println!("Initial state:");
    {
        let rb = world.get::<Rigidbody>(box_entity).unwrap();
        println!("  Linear velocity: {:?}", rb.linear_velocity);
        println!("  Angular velocity: {:?}", rb.angular_velocity);
        println!("  Inertia tensor: {:?}", rb.inertia_tensor);
    }

    // Run one physics step
    println!("\nRunning physics step...");
    simple_physics_update(&mut world, 0.016);

    // Check if we generated any angular velocity
    let rb = world.get::<Rigidbody>(box_entity).unwrap();
    let transform = world.get::<Transform>(box_entity).unwrap();

    println!("\nAfter collision:");
    println!("  Position: {:?}", transform.position);
    println!("  Linear velocity: {:?}", rb.linear_velocity);
    println!("  Angular velocity: {:?}", rb.angular_velocity);

    // The box should have some angular velocity after hitting the ground off-center
    assert!(
        rb.angular_velocity.length() > 0.01,
        "Box should have angular velocity after off-center collision, but has {:?}",
        rb.angular_velocity
    );
}
