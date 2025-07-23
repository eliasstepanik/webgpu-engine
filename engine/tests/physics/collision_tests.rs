//! Tests for collision detection

use engine::physics::collision::{
    broad_phase::{sweep_and_prune, BroadPhaseEntry, SpatialHash},
    narrow_phase::{test_collision, compute_mtv},
    shapes::CollisionShape,
    AABB, Contact,
};
use glam::{Quat, Vec3};
use hecs::Entity;

#[test]
fn test_aabb_creation() {
    let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    assert_eq!(aabb.center(), Vec3::splat(0.5));
    assert_eq!(aabb.half_extents(), Vec3::splat(0.5));
}

#[test]
fn test_aabb_overlap() {
    let aabb1 = AABB::new(Vec3::ZERO, Vec3::ONE);
    let aabb2 = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
    let aabb3 = AABB::new(Vec3::splat(2.0), Vec3::splat(3.0));
    
    assert!(aabb1.overlaps(&aabb2));
    assert!(aabb2.overlaps(&aabb1));
    assert!(!aabb1.overlaps(&aabb3));
    assert!(!aabb3.overlaps(&aabb1));
}

#[test]
fn test_sphere_world_aabb() {
    let sphere = CollisionShape::Sphere { radius: 1.0 };
    let aabb = sphere.world_aabb(Vec3::new(5.0, 0.0, 0.0), Quat::IDENTITY);
    
    assert_eq!(aabb.min, Vec3::new(4.0, -1.0, -1.0));
    assert_eq!(aabb.max, Vec3::new(6.0, 1.0, 1.0));
}

#[test]
fn test_box_world_aabb() {
    let box_shape = CollisionShape::Box {
        half_extents: Vec3::new(1.0, 2.0, 3.0),
    };
    
    // No rotation
    let aabb = box_shape.world_aabb(Vec3::ZERO, Quat::IDENTITY);
    assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, -3.0));
    assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    
    // 90 degree rotation around Y axis
    let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let aabb_rotated = box_shape.world_aabb(Vec3::ZERO, rotation);
    assert!((aabb_rotated.min - Vec3::new(-3.0, -2.0, -1.0)).length() < 0.01);
    assert!((aabb_rotated.max - Vec3::new(3.0, 2.0, 1.0)).length() < 0.01);
}

#[test]
fn test_broad_phase_sweep_and_prune() {
    let entries = vec![
        BroadPhaseEntry {
            entity: Entity::from_bits(0),
            aabb: AABB::new(Vec3::ZERO, Vec3::ONE),
        },
        BroadPhaseEntry {
            entity: Entity::from_bits(1),
            aabb: AABB::new(Vec3::splat(0.5), Vec3::splat(1.5)),
        },
        BroadPhaseEntry {
            entity: Entity::from_bits(2),
            aabb: AABB::new(Vec3::splat(10.0), Vec3::splat(11.0)),
        },
    ];
    
    let pairs = sweep_and_prune(&entries);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0], (0, 1));
}

#[test]
fn test_spatial_hash() {
    let mut hash = SpatialHash::new(2.0);
    
    let aabb1 = AABB::new(Vec3::ZERO, Vec3::ONE);
    let aabb2 = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
    let aabb3 = AABB::new(Vec3::splat(10.0), Vec3::splat(11.0));
    
    hash.insert(0, &aabb1);
    hash.insert(1, &aabb2);
    hash.insert(2, &aabb3);
    
    let query = hash.query(&aabb1);
    assert!(query.contains(&0));
    assert!(query.contains(&1));
    assert!(!query.contains(&2));
}

#[test]
fn test_sphere_sphere_collision() {
    let sphere = CollisionShape::Sphere { radius: 1.0 };
    
    let contact = test_collision(
        &sphere,
        (Vec3::ZERO, Quat::IDENTITY),
        Entity::from_bits(0),
        &sphere,
        (Vec3::new(1.5, 0.0, 0.0), Quat::IDENTITY),
        Entity::from_bits(1),
    );
    
    assert!(contact.is_some());
    let contact = contact.unwrap();
    assert!((contact.penetration - 0.5).abs() < 0.01);
    assert!((contact.normal - Vec3::X).length() < 0.01);
}

#[test]
fn test_sphere_box_collision() {
    let sphere = CollisionShape::Sphere { radius: 1.0 };
    let box_shape = CollisionShape::Box {
        half_extents: Vec3::ONE,
    };
    
    let contact = test_collision(
        &sphere,
        (Vec3::new(2.5, 0.0, 0.0), Quat::IDENTITY),
        Entity::from_bits(0),
        &box_shape,
        (Vec3::ZERO, Quat::IDENTITY),
        Entity::from_bits(1),
    );
    
    assert!(contact.is_some());
    let contact = contact.unwrap();
    assert!(contact.penetration > 0.0);
    assert!(contact.penetration < 0.6);
}

#[test]
fn test_box_box_collision() {
    let box_shape = CollisionShape::Box {
        half_extents: Vec3::ONE,
    };
    
    // Overlapping boxes
    let contact = test_collision(
        &box_shape,
        (Vec3::ZERO, Quat::IDENTITY),
        Entity::from_bits(0),
        &box_shape,
        (Vec3::new(1.5, 0.0, 0.0), Quat::IDENTITY),
        Entity::from_bits(1),
    );
    
    assert!(contact.is_some());
    let contact = contact.unwrap();
    assert!((contact.penetration - 0.5).abs() < 0.01);
    
    // Separated boxes
    let no_contact = test_collision(
        &box_shape,
        (Vec3::ZERO, Quat::IDENTITY),
        Entity::from_bits(0),
        &box_shape,
        (Vec3::new(3.0, 0.0, 0.0), Quat::IDENTITY),
        Entity::from_bits(1),
    );
    
    assert!(no_contact.is_none());
}

#[test]
fn test_collision_shapes_support() {
    let sphere = CollisionShape::Sphere { radius: 2.0 };
    let support = sphere.support(Vec3::new(1.0, 1.0, 0.0).normalize());
    assert!((support.length() - 2.0).abs() < 0.01);
    
    let box_shape = CollisionShape::Box {
        half_extents: Vec3::new(1.0, 2.0, 3.0),
    };
    let support = box_shape.support(Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(support, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_shape_volume() {
    let sphere = CollisionShape::Sphere { radius: 1.0 };
    let expected = (4.0 / 3.0) * std::f32::consts::PI;
    assert!((sphere.volume() - expected).abs() < 0.01);
    
    let box_shape = CollisionShape::Box {
        half_extents: Vec3::new(1.0, 2.0, 3.0),
    };
    assert_eq!(box_shape.volume(), 48.0); // 2*1 * 2*2 * 2*3
}

#[test]
fn test_shape_contains_point() {
    let sphere = CollisionShape::Sphere { radius: 2.0 };
    assert!(sphere.contains_point(Vec3::ZERO));
    assert!(sphere.contains_point(Vec3::new(1.0, 0.0, 0.0)));
    assert!(!sphere.contains_point(Vec3::new(3.0, 0.0, 0.0)));
    
    let box_shape = CollisionShape::Box {
        half_extents: Vec3::ONE,
    };
    assert!(box_shape.contains_point(Vec3::ZERO));
    assert!(box_shape.contains_point(Vec3::new(0.5, 0.5, 0.5)));
    assert!(!box_shape.contains_point(Vec3::new(1.5, 0.0, 0.0)));
}

#[test]
fn test_mtv_computation() {
    let contact = Contact::new(
        Entity::from_bits(0),
        Entity::from_bits(1),
        Vec3::ZERO,
        Vec3::X,
        1.0,
    );
    
    let mtv = compute_mtv(&contact);
    assert_eq!(mtv, Vec3::X);
}