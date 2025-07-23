//! Test hecs directly to isolate the issue
use engine::physics::components::{Collider, CollisionShape, Rigidbody};
use engine::prelude::Transform;
use glam::Vec3;

#[test]
fn test_hecs_directly() {
    // Create a hecs world directly
    let mut world = hecs::World::new();

    // Spawn floor without Rigidbody
    let floor = world.spawn((
        Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Default::default(),
            scale: Vec3::ONE,
        },
        Collider {
            shape: CollisionShape::Box {
                half_extents: Vec3::new(5.0, 0.1, 5.0),
            },
            material_id: Default::default(),
            is_trigger: false,
        },
    ));

    println!("Created floor entity: {floor:?}");

    // Test queries
    // query_one returns Ok if the entity exists, we need to check if we can get the component
    let has_transform = world
        .query_one::<&Transform>(floor)
        .map(|mut q| q.get().is_some())
        .unwrap_or(false);
    let has_collider = world
        .query_one::<&Collider>(floor)
        .map(|mut q| q.get().is_some())
        .unwrap_or(false);
    let has_rigidbody = world
        .query_one::<&Rigidbody>(floor)
        .map(|mut q| q.get().is_some())
        .unwrap_or(false);

    println!("Floor has Transform: {has_transform}");
    println!("Floor has Collider: {has_collider}");
    println!("Floor has Rigidbody: {has_rigidbody}");

    // Count rigidbodies
    let rb_count = world.query::<&Rigidbody>().iter().count();
    println!("Total rigidbodies: {rb_count}");

    // Also try with get
    let has_rb_with_get = world.get::<&Rigidbody>(floor).is_ok();
    println!("Floor has Rigidbody (using get): {has_rb_with_get}");

    assert!(has_transform, "Floor should have Transform");
    assert!(has_collider, "Floor should have Collider");
    assert!(!has_rigidbody, "Floor should NOT have Rigidbody");
    assert_eq!(rb_count, 0, "Should have 0 rigidbodies");
}
