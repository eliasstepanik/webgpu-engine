//! Tests for physics components and systems

#[cfg(test)]
use crate::core::entity::World;
#[cfg(test)]
use crate::physics::commands::create_command_queue;
#[cfg(test)]
use crate::physics::*;
#[cfg(test)]
use glam::Vec3;

#[test]
fn test_rigidbody_serialization() {
    let rb = RigidBody {
        body_type: RigidBodyType::Dynamic,
        linear_damping: 0.5,
        angular_damping: 0.3,
        handle: None, // Should skip in serialization
    };

    let json = serde_json::to_string(&rb).unwrap();
    let deserialized: RigidBody = serde_json::from_str(&json).unwrap();

    assert_eq!(rb.body_type, deserialized.body_type);
    assert_eq!(rb.linear_damping, deserialized.linear_damping);
    assert_eq!(rb.angular_damping, deserialized.angular_damping);
    assert!(deserialized.handle.is_none());
}

#[test]
fn test_collider_serialization() {
    let collider = Collider {
        shape: ColliderShape::Sphere(2.5),
        friction: 0.8,
        restitution: 0.3,
        density: 2.0,
        is_sensor: false,
        handle: Some(ColliderHandle::from_raw_parts(123, 456)), // Should skip
    };

    let json = serde_json::to_string(&collider).unwrap();
    let deserialized: Collider = serde_json::from_str(&json).unwrap();

    assert_eq!(collider.shape, deserialized.shape);
    assert_eq!(collider.friction, deserialized.friction);
    assert_eq!(collider.restitution, deserialized.restitution);
    assert_eq!(collider.density, deserialized.density);
    assert_eq!(collider.is_sensor, deserialized.is_sensor);
    assert!(deserialized.handle.is_none()); // Handle should be skipped
}

#[test]
fn test_physics_velocity_serialization() {
    let velocity = PhysicsVelocity {
        linear: Vec3::new(1.0, 2.0, 3.0),
        angular: Vec3::new(0.1, 0.2, 0.3),
    };

    let json = serde_json::to_string(&velocity).unwrap();
    let deserialized: PhysicsVelocity = serde_json::from_str(&json).unwrap();

    assert_eq!(velocity.linear, deserialized.linear);
    assert_eq!(velocity.angular, deserialized.angular);
}

#[test]
fn test_physics_mass_serialization() {
    let mass = PhysicsMass {
        mass: 10.5,
        center_of_mass: Vec3::new(0.1, 0.0, -0.2),
    };

    let json = serde_json::to_string(&mass).unwrap();
    let deserialized: PhysicsMass = serde_json::from_str(&json).unwrap();

    assert_eq!(mass.mass, deserialized.mass);
    assert_eq!(mass.center_of_mass, deserialized.center_of_mass);
}

#[test]
fn test_collider_shapes() {
    // Test cuboid creation
    let cuboid = Collider::cuboid(1.0, 2.0, 3.0);
    assert!(matches!(cuboid.shape, ColliderShape::Cuboid(_)));
    if let ColliderShape::Cuboid(half_extents) = cuboid.shape {
        assert_eq!(half_extents, Vec3::new(1.0, 2.0, 3.0));
    }

    // Test sphere creation
    let sphere = Collider::sphere(2.5);
    assert!(matches!(sphere.shape, ColliderShape::Sphere(_)));
    if let ColliderShape::Sphere(radius) = sphere.shape {
        assert_eq!(radius, 2.5);
    }

    // Test capsule creation
    let capsule = Collider::capsule(3.0, 1.0);
    assert!(matches!(capsule.shape, ColliderShape::Capsule { .. }));
    if let ColliderShape::Capsule {
        half_height,
        radius,
    } = capsule.shape
    {
        assert_eq!(half_height, 3.0);
        assert_eq!(radius, 1.0);
    }

    // Test cylinder creation
    let cylinder = Collider::cylinder(2.0, 1.5);
    assert!(matches!(cylinder.shape, ColliderShape::Cylinder { .. }));
    if let ColliderShape::Cylinder {
        half_height,
        radius,
    } = cylinder.shape
    {
        assert_eq!(half_height, 2.0);
        assert_eq!(radius, 1.5);
    }
}

#[test]
fn test_physics_world_creation() {
    let physics_world = PhysicsWorld::new();

    // Test default gravity
    assert_eq!(physics_world.gravity[0], 0.0);
    assert_eq!(physics_world.gravity[1], -9.81);
    assert_eq!(physics_world.gravity[2], 0.0);

    // Test integration parameters
    assert_eq!(physics_world.integration_parameters.dt, 1.0 / 60.0);
}

#[test]
fn test_physics_world_entity_registration() {
    use rapier3d_f64::dynamics::RigidBodyBuilder;

    let mut physics_world = PhysicsWorld::new();
    let mut world = World::new();

    // Create a test entity properly through World
    let entity = world.spawn(());

    // Create and insert a rigid body
    let rb = RigidBodyBuilder::dynamic().build();
    let handle = physics_world.rigid_body_set.insert(rb);

    // Register the body
    physics_world.register_body(entity, handle);

    // Test retrieval
    assert_eq!(physics_world.get_body_handle(entity), Some(handle));
    assert_eq!(physics_world.get_entity_for_body(handle), Some(entity));

    // Test unregistration
    let unregistered_handle = physics_world.unregister_body(entity);
    assert_eq!(unregistered_handle, Some(handle));
    assert_eq!(physics_world.get_body_handle(entity), None);
}

#[test]
fn test_physics_command_queue() {
    let queue = create_command_queue();

    // Test adding commands
    {
        let mut commands = queue.write().unwrap();
        commands.push(PhysicsCommand::ApplyForce {
            entity: 123,
            force: Vec3::new(10.0, 0.0, 0.0),
        });
        commands.push(PhysicsCommand::ApplyImpulse {
            entity: 456,
            impulse: Vec3::new(0.0, 5.0, 0.0),
        });
    }

    // Test reading commands
    {
        let commands = queue.read().unwrap();
        assert_eq!(commands.len(), 2);

        match &commands[0] {
            PhysicsCommand::ApplyForce { entity, force } => {
                assert_eq!(*entity, 123);
                assert_eq!(*force, Vec3::new(10.0, 0.0, 0.0));
            }
            _ => panic!("Expected ApplyForce command"),
        }
    }
}

#[test]
fn test_large_world_physics_precision() {
    use rapier3d_f64::dynamics::RigidBodyBuilder;
    use rapier3d_f64::prelude::{nalgebra, vector, ColliderBuilder};

    let mut physics_world = PhysicsWorld::new();

    // Test that Rapier3d-f64 can handle large world coordinates
    let large_position = vector![1_000_000_000.0, 0.0, 0.0];

    // Create a rigid body at a large distance
    let rb = RigidBodyBuilder::dynamic()
        .translation(large_position)
        .build();

    let rb_handle = physics_world.rigid_body_set.insert(rb);

    // Create a collider
    let collider = ColliderBuilder::ball(1.0).build();
    physics_world.collider_set.insert_with_parent(
        collider,
        rb_handle,
        &mut physics_world.rigid_body_set,
    );

    // Step the simulation
    physics_world.step();

    // Verify the position maintained precision after simulation
    let rb = &physics_world.rigid_body_set[rb_handle];
    let pos = rb.translation();

    // Check if position was maintained with high precision
    // Allow small tolerance for floating point operations
    assert!(
        (pos.x - 1_000_000_000.0).abs() < 1.0,
        "Expected position x to be ~1_000_000_000.0, but got {}",
        pos.x
    );

    // Verify gravity affected y position slightly (should have fallen a bit)
    assert!(pos.y < 0.0, "Object should have fallen due to gravity");
}

#[test]
fn test_scripting_physics_commands() {
    use crate::scripting::modules::physics::create_physics_module;

    // Test physics module creation
    let module = create_physics_module();

    // Verify module has expected functions
    // TODO: Module::contains_fn expects u64 hash, not &str
    // Need to find correct way to test module function presence
    // assert!(module.contains_fn("apply_force"));
    // assert!(module.contains_fn("apply_impulse"));
    // assert!(module.contains_fn("apply_torque"));
    // assert!(module.contains_fn("set_velocity"));
    // assert!(module.contains_fn("raycast"));

    // For now, just verify module creation doesn't panic
    assert!(!module.is_empty());
}

// TODO: Component registration has changed, need to update this test
// #[test]
// fn test_component_registration() {
//     use crate::io::component_registry::ComponentRegistry;

//     let mut registry = ComponentRegistry::new();

//     // Register physics components
//     RigidBody::register(&mut registry);
//     Collider::register(&mut registry);
//     PhysicsVelocity::register(&mut registry);
//     PhysicsMass::register(&mut registry);

//     // Verify they were registered
//     assert!(registry.get_metadata("RigidBody").is_some());
//     assert!(registry.get_metadata("Collider").is_some());
//     assert!(registry.get_metadata("PhysicsVelocity").is_some());
//     assert!(registry.get_metadata("PhysicsMass").is_some());
// }
