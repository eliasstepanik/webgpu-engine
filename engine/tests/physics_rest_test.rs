//! Test that physics objects come to rest properly

use engine::core::entity::World;
use engine::core::entity::{GlobalTransform, Transform};
use engine::physics::components::{Collider, CollisionShape, Rigidbody};
use engine::physics::simple_physics::simple_physics_update;
use glam::{Mat3, Vec3};

#[test]
fn test_object_comes_to_rest() {
    let mut world = World::new();

    // Create ground (static)
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

    // Create falling sphere
    let sphere = world.spawn((
        Transform::from_position(Vec3::new(0.0, 5.0, 0.0)),
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.01,
            angular_damping: 0.01,
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

    // Update hierarchy to sync GlobalTransform
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Simulate for 5 seconds
    let dt = 0.016; // 60 FPS
    let mut last_y = 5.0;
    let mut at_rest_count = 0;

    for i in 0..300 {
        simple_physics_update(&mut world, &engine::physics::PhysicsConfig::default(), dt);

        let transform = world.get::<Transform>(sphere).unwrap();
        let velocity = world.get::<Rigidbody>(sphere).unwrap();

        println!(
            "Frame {}: y={:.4}, vel_y={:.4}",
            i, transform.position.y, velocity.linear_velocity.y
        );

        // Check if object is at rest (not moving much)
        let movement = (transform.position.y - last_y).abs();
        if movement < 0.001 && velocity.linear_velocity.length() < 0.01 {
            at_rest_count += 1;
        } else {
            at_rest_count = 0;
        }

        last_y = transform.position.y;

        // If object has been at rest for 10 frames, we're good
        if at_rest_count > 10 {
            break;
        }
    }

    // Verify object came to rest
    let final_transform = world.get::<Transform>(sphere).unwrap();
    let final_velocity = world.get::<Rigidbody>(sphere).unwrap();

    // Sphere should be resting on top of ground
    // Ground is at y=0 with half_extent 0.5, so top is at y=0.5
    // Sphere has radius 0.5, so its center should be at y=1.0
    assert!(
        (final_transform.position.y - 1.0).abs() < 0.05,
        "Sphere should rest at y=1.0, but is at y={}",
        final_transform.position.y
    );

    // Velocity should be near zero
    assert!(
        final_velocity.linear_velocity.length() < 0.02,
        "Sphere should have near-zero velocity, but has velocity={:?}",
        final_velocity.linear_velocity
    );
}

#[test]
fn test_stacking_stability() {
    let mut world = World::new();

    // Create ground
    world.spawn((
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

    // Create stack of boxes
    let mut boxes = Vec::new();
    for i in 0..3 {
        let y = 2.0 + i as f32 * 1.5; // Larger gap to avoid initial penetration
        let box_entity = world.spawn((
            Transform::from_position(Vec3::new(0.0, y, 0.0)),
            GlobalTransform::default(),
            Rigidbody {
                mass: 1.0,
                linear_velocity: Vec3::ZERO,
                angular_velocity: Vec3::ZERO,
                linear_damping: 0.05,
                angular_damping: 0.05,
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
        boxes.push(box_entity);
    }

    // Update hierarchy
    engine::core::entity::hierarchy::advance_frame();
    engine::core::entity::update_hierarchy_system(&mut world);

    // Simulate for 3 seconds
    let dt = 0.016;
    for frame in 0..180 {
        simple_physics_update(&mut world, &engine::physics::PhysicsConfig::default(), dt);

        // Print positions every 30 frames
        if frame % 30 == 29 {
            println!("Frame {frame}:");
            for (i, &box_entity) in boxes.iter().enumerate() {
                let transform = world.get::<Transform>(box_entity).unwrap();
                let velocity = world.get::<Rigidbody>(box_entity).unwrap();
                println!(
                    "  Box {}: y={:.3}, vel_y={:.3}",
                    i, transform.position.y, velocity.linear_velocity.y
                );
            }
        }
    }

    // Check that boxes are stacked properly (allowing for some compression)
    let mut prev_y = 0.5; // Top of ground
    for (i, &box_entity) in boxes.iter().enumerate() {
        let transform = world.get::<Transform>(box_entity).unwrap();

        // Each box should be at least 0.8 units above the previous (allowing 20% compression)
        let min_y = prev_y + 0.8;
        assert!(
            transform.position.y >= min_y,
            "Box {} is too compressed: y={}, expected at least y={}",
            i,
            transform.position.y,
            min_y
        );

        // Boxes shouldn't be floating too high
        let max_y = prev_y + 1.2;
        assert!(
            transform.position.y <= max_y,
            "Box {} is floating too high: y={}, expected at most y={}",
            i,
            transform.position.y,
            max_y
        );

        prev_y = transform.position.y + 0.5; // Add half box height for next comparison
    }
}
