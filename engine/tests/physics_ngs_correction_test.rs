//! Tests for NGS position correction

use engine::core::entity::{Transform, World};
use engine::physics::{
    components::{Collider, CollisionShape, Rigidbody},
    systems::{create_physics_solver, update_physics_system},
    PhysicsConfig,
};
use glam::Vec3;

#[test]
fn test_ngs_position_correction_prevents_penetration() {
    let mut world = World::new();
    let config = PhysicsConfig {
        contact_slop: 0.004, // 4mm allowed penetration
        position_correction_rate: 0.8,
        ..Default::default()
    };

    let mut solver = create_physics_solver(&config);

    // Create ground
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

    // Create box that starts penetrating the ground
    let box_entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 0.9, 0.0)), // 0.1m penetration
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            linear_velocity: Vec3::ZERO,
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

    // Update hierarchy
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Run physics for a few steps
    for _ in 0..5 {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);
    }

    // Check that penetration is resolved
    let transform = world.get::<Transform>(box_entity).unwrap();
    assert!(
        transform.position.y >= 1.0 - config.contact_slop,
        "NGS should resolve penetration to within contact slop. Position: {}",
        transform.position.y
    );
}

#[test]
fn test_ngs_maintains_contact_slop() {
    let mut world = World::new();
    let config = PhysicsConfig {
        contact_slop: 0.004, // 4mm
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

    // Create resting box
    let box_entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 1.0, 0.0)),
        engine::core::entity::components::GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            linear_velocity: Vec3::ZERO,
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

    // Let it settle
    for _ in 0..20 {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);
    }

    // Check that small penetration is maintained for stability
    let transform = world.get::<Transform>(box_entity).unwrap();
    let penetration = 1.0 - transform.position.y;

    assert!(
        penetration >= 0.0,
        "Should have slight penetration for stability"
    );
    assert!(
        penetration <= config.contact_slop,
        "Penetration {} should be within contact slop {}",
        penetration,
        config.contact_slop
    );
}

#[test]
fn test_ngs_handles_stacking() {
    let mut world = World::new();
    let config = PhysicsConfig::default();
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

    // Create stack of boxes
    let mut boxes = Vec::new();
    for i in 0..3 {
        let box_entity = world.spawn((
            Transform::from_position(Vec3::new(0.0, 1.0 + i as f32, 0.0)),
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
        boxes.push(box_entity);
    }

    // Let stack settle
    for _ in 0..100 {
        update_physics_system(&mut world, &mut solver, &config, config.fixed_timestep);
        engine::core::entity::hierarchy::advance_frame();
        engine::core::entity::update_hierarchy_system(&mut world);
    }

    // Check stack positions
    for (i, &box_entity) in boxes.iter().enumerate() {
        let transform = world.get::<Transform>(box_entity).unwrap();
        let expected_y = 1.0 + i as f32;

        assert!(
            (transform.position.y - expected_y).abs() < 0.02,
            "Box {} should be at y={}, but is at y={}",
            i,
            expected_y,
            transform.position.y
        );
    }
}
