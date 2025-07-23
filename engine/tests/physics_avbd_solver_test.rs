//! Tests for AVBD solver integration

use engine::core::entity::{Transform, World};
use engine::physics::{
    components::{Collider, CollisionShape, Rigidbody},
    systems::{create_physics_solver, update_physics_system},
    PhysicsConfig,
};
use glam::Vec3;
use tracing::info;

#[test]
fn test_avbd_solver_gravity() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let mut world = World::new();
    let mut config = PhysicsConfig::default();
    config.gravity = Vec3::new(0.0, -10.0, 0.0);
    config.fixed_timestep = 1.0 / 60.0;

    let mut solver = create_physics_solver(&config);

    // Create falling sphere
    let sphere = world.spawn((
        Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            ..Default::default()
        },
        Collider {
            shape: CollisionShape::Sphere { radius: 0.5 },
            is_trigger: false,
            material_id: None,
        },
    ));

    // Update hierarchy to compute GlobalTransform
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Simulate for 1 second
    let num_steps = 60;
    for i in 0..num_steps {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);

        // Update hierarchy after physics
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);

        if i % 10 == 0 {
            let transform = world.get::<Transform>(sphere).unwrap();
            info!("Step {}: position = {:?}", i, transform.position);
        }
    }

    // After 1 second of falling, check position
    let transform = world.get::<Transform>(sphere).unwrap();

    // y = y0 + v0*t + 0.5*a*t^2
    // y = 10 + 0*1 + 0.5*(-10)*1^2 = 10 - 5 = 5
    assert!(
        (transform.position.y - 5.0).abs() < 0.1,
        "Expected y position around 5.0, got {}",
        transform.position.y
    );
}

#[test]
fn test_fixed_timestep_consistency() {
    let config = PhysicsConfig {
        fixed_timestep: 1.0 / 120.0,
        ..Default::default()
    };

    // Update with variable delta times that should give same result
    let test_cases = vec![
        vec![0.016, 0.016, 0.016, 0.016], // ~60 fps
        vec![0.008, 0.024, 0.032],        // Variable
        vec![0.064],                      // Single large step
    ];

    let mut final_positions = Vec::new();

    for deltas in test_cases {
        // Create fresh world for each test
        let mut test_world = World::new();
        let mut test_solver = create_physics_solver(&config);

        // Create a falling object
        let entity = test_world.spawn((
            Transform::from_position(Vec3::new(0.0, 100.0, 0.0)),
            engine::core::entity::components::GlobalTransform::default(),
            Rigidbody {
                mass: 1.0,
                use_gravity: true,
                ..Default::default()
            },
        ));

        // Run simulation
        for dt in deltas {
            update_physics_system(&mut test_world, &mut test_solver, &config, dt);
        }

        // Get final position
        let transform = test_world.get::<Transform>(entity).unwrap();
        final_positions.push(transform.position.y);
    }

    // All simulations should give similar results (within numerical precision)
    for i in 1..final_positions.len() {
        assert!(
            (final_positions[i] - final_positions[0]).abs() < 0.01,
            "Fixed timestep should give consistent results regardless of frame timing"
        );
    }
}

#[test]
fn test_warmstarting_improves_convergence() {
    let mut world = World::new();
    let config = PhysicsConfig::default();
    let mut solver = create_physics_solver(&config);

    // Create stacked boxes scenario
    let _ground = world.spawn((
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

    let box1 = world.spawn((
        Transform::from_position(Vec3::new(0.0, 1.5, 0.0)),
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
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

    // Run several frames to build up warmstart data
    for _ in 0..10 {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);
    }

    // Warmstarting should maintain stable contact
    let transform = world.get::<Transform>(box1).unwrap();
    assert!(
        (transform.position.y - 1.0).abs() < 0.01,
        "Box should rest at y=1.0 with warmstarting"
    );
}
