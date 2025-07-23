//! Tests for PhysicsConfig integration

use engine::core::entity::{Transform, World};
use engine::physics::{
    components::{Collider, CollisionShape, Rigidbody},
    systems::{create_physics_solver, update_physics_system},
    PhysicsConfig,
};
use glam::Vec3;

#[test]
fn test_custom_gravity_config() {
    let mut world = World::new();
    let config = PhysicsConfig {
        gravity: Vec3::new(0.0, -1.62, 0.0), // Moon-like gravity
        ..Default::default()
    };

    let mut solver = create_physics_solver(&config);

    // Create falling object
    let object = world.spawn((
        Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            ..Default::default()
        },
    ));

    // Simulate for 1 second
    let steps = (1.0 / config.fixed_timestep) as i32;
    for _ in 0..steps {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);
    }

    // Check position with custom gravity
    let transform = world.get::<Transform>(object).unwrap();
    let expected_y = 10.0 + 0.5 * config.gravity.y * 1.0; // y = y0 + 0.5 * a * t^2

    assert!(
        (transform.position.y - expected_y).abs() < 0.1,
        "Object should fall according to custom gravity. Expected {}, got {}",
        expected_y,
        transform.position.y
    );
}

#[test]
fn test_velocity_clamping_config() {
    let mut world = World::new();
    let config = PhysicsConfig {
        max_linear_velocity: 5.0, // Low max velocity
        max_angular_velocity: 2.0,
        ..Default::default()
    };

    let mut solver = create_physics_solver(&config);

    // Create object with high initial velocity
    let object = world.spawn((
        Transform::from_position(Vec3::ZERO),
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: false,
            linear_velocity: Vec3::new(100.0, 0.0, 0.0), // Way over max
            angular_velocity: Vec3::new(0.0, 10.0, 0.0), // Also over max
            ..Default::default()
        },
    ));

    // Run one physics step
    update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);

    // Check velocities are clamped
    let rb = world.get::<Rigidbody>(object).unwrap();

    assert!(
        rb.linear_velocity.length() <= config.max_linear_velocity + 0.01,
        "Linear velocity should be clamped to max"
    );
    assert!(
        rb.angular_velocity.length() <= config.max_angular_velocity + 0.01,
        "Angular velocity should be clamped to max"
    );
}

#[test]
fn test_rest_velocity_threshold() {
    let mut world = World::new();
    let config = PhysicsConfig {
        rest_velocity_threshold: 0.1,
        ..Default::default()
    };

    let mut solver = create_physics_solver(&config);

    // Create ground
    world.spawn((
        Transform::from_position(Vec3::ZERO),
        engine::core::entity::components::GlobalTransform::default(),
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(10.0, 0.5, 10.0),
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Create falling box
    let box_entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 1.2, 0.0)),
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            linear_velocity: Vec3::new(0.05, -0.05, 0.05), // Below threshold
            ..Default::default()
        },
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::ONE * 0.5,
            },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Run physics to let it collide
    for _ in 0..10 {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);
    }

    // Velocity should be zeroed due to rest threshold
    let rb = world.get::<Rigidbody>(box_entity).unwrap();
    assert_eq!(
        rb.linear_velocity,
        Vec3::ZERO,
        "Velocity below rest threshold should be zeroed"
    );
}

#[test]
fn test_damping_config() {
    let mut world = World::new();
    let config = PhysicsConfig {
        linear_damping: 0.5, // High damping
        angular_damping: 0.5,
        ..Default::default()
    };

    let mut solver = create_physics_solver(&config);

    // Create moving object
    let object = world.spawn((
        Transform::from_position(Vec3::ZERO),
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: false,
            linear_velocity: Vec3::new(10.0, 0.0, 0.0),
            angular_velocity: Vec3::new(0.0, 5.0, 0.0),
            linear_damping: config.linear_damping,
            angular_damping: config.angular_damping,
            ..Default::default()
        },
    ));

    // Run for 1 second
    let steps = (1.0 / config.fixed_timestep) as i32;
    for _ in 0..steps {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);
    }

    // Velocities should be significantly reduced
    let rb = world.get::<Rigidbody>(object).unwrap();

    assert!(
        rb.linear_velocity.length() < 5.0,
        "Linear velocity should be damped significantly"
    );
    assert!(
        rb.angular_velocity.length() < 2.5,
        "Angular velocity should be damped significantly"
    );
}
