//! Tests for correct contact point calculation in physics

use engine::physics::collision::narrow_phase::test_collision;
use engine::physics::components::CollisionShape;
use glam::{Quat, Vec3};

#[test]
fn test_sphere_box_contact_point_on_sphere_surface() {
    let mut world = hecs::World::new();
    let sphere_entity = world.spawn(());
    let box_entity = world.spawn(());

    // Sphere at origin with radius 1
    let sphere_shape = CollisionShape::Sphere { radius: 1.0 };
    let sphere_pos = Vec3::ZERO;
    let sphere_rot = Quat::IDENTITY;

    // Box to the right
    let box_shape = CollisionShape::Box {
        half_extents: Vec3::ONE,
    };
    let box_pos = Vec3::new(2.5, 0.0, 0.0);
    let box_rot = Quat::IDENTITY;

    // Test collision
    let contact = test_collision(
        &sphere_shape,
        (sphere_pos, sphere_rot),
        sphere_entity,
        &box_shape,
        (box_pos, box_rot),
        box_entity,
    );

    assert!(contact.is_some());
    let contact = contact.unwrap();

    // Contact point should be on sphere surface
    let distance_from_sphere_center = (contact.position - sphere_pos).length();
    assert!(
        (distance_from_sphere_center - 1.0).abs() < 0.001,
        "Contact point should be on sphere surface, but distance is {distance_from_sphere_center}"
    );

    // Contact point should be between sphere and box
    assert!(contact.position.x > sphere_pos.x);
    assert!(contact.position.x < box_pos.x - 1.0);
}

#[test]
fn test_box_box_contact_point_in_overlap() {
    let mut world = hecs::World::new();
    let box_a_entity = world.spawn(());
    let box_b_entity = world.spawn(());

    // Box A at origin
    let box_a_shape = CollisionShape::Box {
        half_extents: Vec3::ONE,
    };
    let box_a_pos = Vec3::ZERO;
    let box_a_rot = Quat::IDENTITY;

    // Box B slightly overlapping from above
    let box_b_shape = CollisionShape::Box {
        half_extents: Vec3::ONE,
    };
    let box_b_pos = Vec3::new(0.0, 1.9, 0.0); // 0.1 penetration
    let box_b_rot = Quat::IDENTITY;

    // Test collision
    let contact = test_collision(
        &box_a_shape,
        (box_a_pos, box_a_rot),
        box_a_entity,
        &box_b_shape,
        (box_b_pos, box_b_rot),
        box_b_entity,
    );

    assert!(contact.is_some());
    let contact = contact.unwrap();

    // Contact point should be in the overlap region
    assert!(contact.position.y > 0.9); // Above box A top
    assert!(contact.position.y < 1.0); // Below box B bottom

    // Contact normal should point upward (from A to B)
    assert!(contact.normal.y > 0.9);

    // Penetration should be approximately 0.1
    assert!((contact.penetration - 0.1).abs() < 0.01);
}

#[test]
fn test_sphere_sphere_contact_point_midway() {
    let mut world = hecs::World::new();
    let sphere_a_entity = world.spawn(());
    let sphere_b_entity = world.spawn(());

    // Sphere A at origin
    let sphere_a_shape = CollisionShape::Sphere { radius: 1.0 };
    let sphere_a_pos = Vec3::ZERO;
    let sphere_a_rot = Quat::IDENTITY;

    // Sphere B to the right, overlapping
    let sphere_b_shape = CollisionShape::Sphere { radius: 1.0 };
    let sphere_b_pos = Vec3::new(1.5, 0.0, 0.0); // 0.5 penetration
    let sphere_b_rot = Quat::IDENTITY;

    // Test collision
    let contact = test_collision(
        &sphere_a_shape,
        (sphere_a_pos, sphere_a_rot),
        sphere_a_entity,
        &sphere_b_shape,
        (sphere_b_pos, sphere_b_rot),
        sphere_b_entity,
    );

    assert!(contact.is_some());
    let contact = contact.unwrap();

    // Contact point should be between sphere centers
    assert!(contact.position.x > 0.0);
    assert!(contact.position.x < 1.5);

    // For spheres, contact point is on first sphere's surface
    let expected_contact = sphere_a_pos + contact.normal * (1.0 - 0.5 * 0.5);
    assert!((contact.position - expected_contact).length() < 0.01);
}
