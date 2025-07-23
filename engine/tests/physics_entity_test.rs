//! Test entity creation for physics

use engine::core::entity::World;
use engine::physics::components::{Collider, CollisionShape, Rigidbody};
use engine::prelude::Transform;
use glam::{Mat3, Quat, Vec3};
use tracing::info;

#[test]
fn test_entity_components() {
    // Initialize logging
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    info!("Starting entity component test");

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

    // Check components more carefully
    info!("Checking floor components...");

    // Count total rigidbodies
    let total_rbs = world.query::<&Rigidbody>().iter().count();
    info!("Total rigidbodies in world: {}", total_rbs);

    let has_transform = world.query_one::<&Transform>(floor).is_ok();
    let has_collider = world.query_one::<&Collider>(floor).is_ok();
    let has_rigidbody = world.query_one::<&Rigidbody>(floor).is_ok();

    info!("Floor has Transform: {}", has_transform);
    info!("Floor has Collider: {}", has_collider);
    info!("Floor has Rigidbody: {}", has_rigidbody);

    // Query all entities with rigidbodies
    for (entity, rb) in world.query::<&Rigidbody>().iter() {
        info!("Entity {:?} has rigidbody with mass {}", entity, rb.mass);
    }

    assert!(has_transform, "Floor should have Transform");
    assert!(has_collider, "Floor should have Collider");
    assert!(!has_rigidbody, "Floor should NOT have Rigidbody");

    // Create cube with rigidbody
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

    // Check cube components
    let cube_has_transform = world.query_one::<&Transform>(cube).is_ok();
    let cube_has_collider = world.query_one::<&Collider>(cube).is_ok();
    let cube_has_rigidbody = world.query_one::<&Rigidbody>(cube).is_ok();

    info!("Cube has Transform: {}", cube_has_transform);
    info!("Cube has Collider: {}", cube_has_collider);
    info!("Cube has Rigidbody: {}", cube_has_rigidbody);

    assert!(cube_has_transform, "Cube should have Transform");
    assert!(cube_has_collider, "Cube should have Collider");
    assert!(cube_has_rigidbody, "Cube should have Rigidbody");

    // Double check floor still doesn't have rigidbody
    let floor_still_no_rb = world.query_one::<&Rigidbody>(floor).is_err();
    assert!(floor_still_no_rb, "Floor should still not have Rigidbody");
}
