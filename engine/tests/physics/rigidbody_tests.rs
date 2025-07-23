//! Tests for rigidbody physics

use engine::core::entity::{Transform, World};
use engine::physics::{Collider, Rigidbody, avbd_solver::AVBDConfig, systems::{create_default_solver, update_physics_system}};
use glam::Vec3;

#[test]
fn test_gravity_integration() {
    let mut world = World::new();
    let mut solver = create_default_solver();
    
    // Create a falling body
    let entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            ..Default::default()
        },
        Collider::sphere(1.0),
    ));
    
    // Step physics
    let dt = 0.016; // 60 FPS
    update_physics_system(&mut world, &mut solver, dt);
    
    // Check that velocity has increased downward
    let rb = world.get::<Rigidbody>(entity).unwrap();
    assert!(rb.linear_velocity.y < 0.0);
    assert!((rb.linear_velocity.y - (-9.81 * dt)).abs() < 0.01);
}

#[test]
fn test_kinematic_bodies() {
    let mut world = World::new();
    let mut solver = create_default_solver();
    
    // Create a kinematic body
    let entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
        Rigidbody::kinematic(),
        Collider::box_collider(Vec3::ONE),
    ));
    
    // Store initial position
    let initial_pos = world.get::<Transform>(entity).unwrap().position;
    
    // Step physics
    update_physics_system(&mut world, &mut solver, 0.016);
    
    // Check that position hasn't changed
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.position, initial_pos);
}

#[test]
fn test_velocity_damping() {
    let mut world = World::new();
    let mut solver = create_default_solver();
    
    // Create a body with initial velocity and damping
    let entity = world.spawn((
        Transform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::new(10.0, 0.0, 0.0),
            linear_damping: 0.5,
            use_gravity: false,
            ..Default::default()
        },
        Collider::sphere(1.0),
    ));
    
    // Store initial velocity
    let initial_vel = world.get::<Rigidbody>(entity).unwrap().linear_velocity;
    
    // Step physics
    update_physics_system(&mut world, &mut solver, 0.1);
    
    // Check that velocity has decreased
    let rb = world.get::<Rigidbody>(entity).unwrap();
    assert!(rb.linear_velocity.length() < initial_vel.length());
}

#[test]
fn test_multiple_bodies() {
    let mut world = World::new();
    let mut solver = create_default_solver();
    
    // Create multiple falling bodies
    let entities: Vec<_> = (0..10)
        .map(|i| {
            world.spawn((
                Transform::from_position(Vec3::new(i as f32 * 2.0, 10.0, 0.0)),
                Rigidbody {
                    mass: 1.0,
                    use_gravity: true,
                    ..Default::default()
                },
                Collider::sphere(0.5),
            ))
        })
        .collect();
    
    // Step physics
    update_physics_system(&mut world, &mut solver, 0.016);
    
    // Check that all bodies have downward velocity
    for entity in entities {
        let rb = world.get::<Rigidbody>(entity).unwrap();
        assert!(rb.linear_velocity.y < 0.0);
    }
}

#[test]
fn test_force_application() {
    let mut world = World::new();
    
    // Create a body
    let entity = world.spawn((
        Transform::default(),
        Rigidbody {
            mass: 2.0,
            use_gravity: false,
            ..Default::default()
        },
    ));
    
    // Apply force
    {
        let mut rb = world.get_mut::<Rigidbody>(entity).unwrap();
        rb.apply_force(Vec3::new(10.0, 0.0, 0.0), 0.1);
    }
    
    // Check velocity
    let rb = world.get::<Rigidbody>(entity).unwrap();
    // F = ma, a = F/m = 10/2 = 5, v = at = 5 * 0.1 = 0.5
    assert!((rb.linear_velocity.x - 0.5).abs() < 0.01);
}

#[test]
fn test_inertia_calculation() {
    let sphere = Collider::sphere(2.0);
    let mass = 10.0;
    let inertia = sphere.shape.calculate_inertia(mass);
    
    // Sphere inertia: I = 0.4 * m * r²
    let expected = 0.4 * mass * 4.0;
    assert!((inertia.x_axis.x - expected).abs() < 0.01);
    assert!((inertia.y_axis.y - expected).abs() < 0.01);
    assert!((inertia.z_axis.z - expected).abs() < 0.01);
}

#[test]
fn test_transform_update() {
    let mut world = World::new();
    let mut solver = create_default_solver();
    
    // Create a body with initial velocity
    let entity = world.spawn((
        Transform::from_position(Vec3::ZERO),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::new(1.0, 0.0, 0.0),
            use_gravity: false,
            ..Default::default()
        },
        Collider::box_collider(Vec3::splat(0.5)),
    ));
    
    // Step physics
    let dt = 0.1;
    update_physics_system(&mut world, &mut solver, dt);
    
    // Check that transform was updated
    let transform = world.get::<Transform>(entity).unwrap();
    assert!((transform.position.x - 0.1).abs() < 0.01); // pos = vel * dt
}

#[test]
fn test_no_gravity_bodies() {
    let mut world = World::new();
    let mut solver = create_default_solver();
    
    // Create a body without gravity
    let entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
        Rigidbody {
            mass: 1.0,
            use_gravity: false,
            ..Default::default()
        },
        Collider::sphere(1.0),
    ));
    
    // Step physics
    update_physics_system(&mut world, &mut solver, 0.016);
    
    // Check that velocity is still zero
    let rb = world.get::<Rigidbody>(entity).unwrap();
    assert_eq!(rb.linear_velocity, Vec3::ZERO);
}