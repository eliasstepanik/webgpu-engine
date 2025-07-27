//! Physics update system
//!
//! This system synchronizes between the ECS transform components and the Rapier
//! physics simulation, supporting both Transform (f32) and WorldTransform (f64).

use crate::core::entity::{Entity, Transform, World, WorldTransform};
use crate::physics::{
    Collider, ColliderShape, PhysicsMass, PhysicsVelocity, PhysicsWorld, RigidBody, RigidBodyType,
};
use glam::{DVec3, Quat, Vec3};
use rapier3d_f64::prelude::*;
use tracing::{debug, trace, warn};

thread_local! {
    /// Thread-local physics command queue for deferred operations
    static PHYSICS_COMMAND_QUEUE: std::cell::RefCell<Vec<crate::physics::PhysicsCommand>> = std::cell::RefCell::new(Vec::new());
}

/// Update the physics simulation
pub fn physics_update_system(
    world: &mut World,
    physics_world: &mut PhysicsWorld,
    delta_time: f32,
) {
    trace!("Physics update system starting");
    
    // Step 1: Create new physics bodies and colliders for entities that need them
    create_physics_bodies(world, physics_world);
    
    // Step 2: Sync ECS transforms to physics positions
    sync_transforms_to_physics(world, physics_world);
    
    // Step 3: Sync velocities from ECS to physics
    sync_velocities_to_physics(world, physics_world);
    
    // Step 4: Process physics commands from scripts
    process_physics_commands(physics_world);
    
    // Step 5: Step the physics simulation
    physics_world.step();
    
    // Step 6: Write physics results back to ECS
    sync_physics_to_transforms(world, physics_world);
    sync_physics_to_velocities(world, physics_world);
    
    // Step 7: Clean up removed entities
    cleanup_removed_entities(world, physics_world);
    
    trace!("Physics update system completed");
}

/// Create physics bodies and colliders for entities that don't have them yet
fn create_physics_bodies(world: &mut World, physics_world: &mut PhysicsWorld) {
    // Query for entities with RigidBody component but no handle
    let mut bodies_to_create = Vec::new();
    
    for (entity, rb_component) in world.query::<&RigidBody>().iter() {
        if rb_component.handle.is_none() {
            bodies_to_create.push((entity, rb_component.clone()));
        }
    }
    
    // Create rigid bodies
    for (entity, mut rb_component) in bodies_to_create {
        let rb_type = match rb_component.body_type {
            RigidBodyType::Dynamic => rapier3d_f64::dynamics::RigidBodyType::Dynamic,
            RigidBodyType::Fixed => rapier3d_f64::dynamics::RigidBodyType::Fixed,
            RigidBodyType::KinematicPositionBased => {
                rapier3d_f64::dynamics::RigidBodyType::KinematicPositionBased
            }
            RigidBodyType::KinematicVelocityBased => {
                rapier3d_f64::dynamics::RigidBodyType::KinematicVelocityBased
            }
        };
        
        let mut rb_builder = RigidBodyBuilder::new(rb_type)
            .linear_damping(rb_component.linear_damping)
            .angular_damping(rb_component.angular_damping);
        
        // Set initial position from transform
        if let Ok(world_transform) = world.get::<&WorldTransform>(entity) {
            rb_builder = rb_builder.translation(vector![
                world_transform.position.x,
                world_transform.position.y,
                world_transform.position.z
            ]);
            rb_builder = rb_builder.rotation(world_transform.rotation.into());
        } else if let Ok(transform) = world.get::<&Transform>(entity) {
            rb_builder = rb_builder.translation(vector![
                transform.position.x as f64,
                transform.position.y as f64,
                transform.position.z as f64
            ]);
            rb_builder = rb_builder.rotation(transform.rotation.into());
        }
        
        let rb_handle = physics_world.rigid_body_set.insert(rb_builder);
        physics_world.register_body(entity, rb_handle);
        
        // Update the component with the handle
        rb_component.handle = Some(rb_handle);
        world.insert_one(entity, rb_component).ok();
        
        debug!("Created rigid body for entity {:?}", entity);
    }
    
    // Query for entities with Collider component but no handle
    let mut colliders_to_create = Vec::new();
    
    for (entity, (collider_component, _)) in world.query::<(&Collider, &RigidBody)>().iter() {
        if collider_component.handle.is_none() {
            colliders_to_create.push((entity, collider_component.clone()));
        }
    }
    
    // Create colliders
    for (entity, mut collider_component) in colliders_to_create {
        if let Some(rb_handle) = physics_world.get_body_handle(entity) {
            let shape = match &collider_component.shape {
                ColliderShape::Cuboid(half_extents) => SharedShape::cuboid(
                    half_extents.x as f64,
                    half_extents.y as f64,
                    half_extents.z as f64,
                ),
                ColliderShape::Sphere(radius) => SharedShape::ball(*radius as f64),
                ColliderShape::Capsule { half_height, radius } => {
                    SharedShape::capsule_y(*half_height as f64, *radius as f64)
                }
                ColliderShape::Cylinder { half_height, radius } => {
                    SharedShape::cylinder(*half_height as f64, *radius as f64)
                }
            };
            
            let mut collider_builder = ColliderBuilder::new(shape)
                .friction(collider_component.friction)
                .restitution(collider_component.restitution)
                .density(collider_component.density)
                .sensor(collider_component.is_sensor);
            
            // Apply mass properties if present
            if let Ok(mass) = world.get::<&PhysicsMass>(entity) {
                collider_builder = collider_builder
                    .mass(mass.mass)
                    .center_of_mass(vector![
                        mass.center_of_mass.x as f64,
                        mass.center_of_mass.y as f64,
                        mass.center_of_mass.z as f64
                    ]);
            }
            
            let collider_handle = physics_world.collider_set.insert_with_parent(
                collider_builder,
                rb_handle,
                &mut physics_world.rigid_body_set,
            );
            
            physics_world.register_collider(entity, collider_handle);
            
            // Update the component with the handle
            collider_component.handle = Some(collider_handle);
            world.insert_one(entity, collider_component).ok();
            
            debug!("Created collider for entity {:?}", entity);
        } else {
            warn!("Entity {:?} has Collider but no RigidBody", entity);
        }
    }
}

/// Sync transforms from ECS to physics
fn sync_transforms_to_physics(world: &World, physics_world: &mut PhysicsWorld) {
    // Sync entities with WorldTransform (high precision)
    for (entity, (world_transform, rb)) in world.query::<(&WorldTransform, &RigidBody)>().iter() {
        if let Some(handle) = rb.handle {
            if let Some(rigid_body) = physics_world.rigid_body_set.get_mut(handle) {
                // Only sync kinematic and fixed bodies (dynamic bodies are controlled by physics)
                if matches!(
                    rb.body_type,
                    RigidBodyType::KinematicPositionBased | RigidBodyType::Fixed
                ) {
                    rigid_body.set_translation(
                        vector![
                            world_transform.position.x,
                            world_transform.position.y,
                            world_transform.position.z
                        ],
                        true,
                    );
                    rigid_body.set_rotation(world_transform.rotation.into(), true);
                }
            }
        }
    }
    
    // Sync entities with regular Transform (standard precision)
    for (entity, (transform, rb)) in world.query::<(&Transform, &RigidBody)>().iter() {
        // Skip if entity has WorldTransform (already handled above)
        if world.get::<&WorldTransform>(entity).is_ok() {
            continue;
        }
        
        if let Some(handle) = rb.handle {
            if let Some(rigid_body) = physics_world.rigid_body_set.get_mut(handle) {
                // Only sync kinematic and fixed bodies
                if matches!(
                    rb.body_type,
                    RigidBodyType::KinematicPositionBased | RigidBodyType::Fixed
                ) {
                    rigid_body.set_translation(
                        vector![
                            transform.position.x as f64,
                            transform.position.y as f64,
                            transform.position.z as f64
                        ],
                        true,
                    );
                    rigid_body.set_rotation(transform.rotation.into(), true);
                }
            }
        }
    }
}

/// Sync velocities from ECS to physics
fn sync_velocities_to_physics(world: &World, physics_world: &mut PhysicsWorld) {
    for (entity, (velocity, rb)) in world.query::<(&PhysicsVelocity, &RigidBody)>().iter() {
        if let Some(handle) = rb.handle {
            if let Some(rigid_body) = physics_world.rigid_body_set.get_mut(handle) {
                // Only set velocities for dynamic bodies
                if rb.body_type == RigidBodyType::Dynamic {
                    rigid_body.set_linvel(
                        vector![
                            velocity.linear.x as f64,
                            velocity.linear.y as f64,
                            velocity.linear.z as f64
                        ],
                        true,
                    );
                    rigid_body.set_angvel(
                        vector![
                            velocity.angular.x as f64,
                            velocity.angular.y as f64,
                            velocity.angular.z as f64
                        ],
                        true,
                    );
                }
            }
        }
    }
}

/// Process physics commands from the command queue
fn process_physics_commands(physics_world: &mut PhysicsWorld) {
    use crate::physics::PhysicsCommand;
    
    PHYSICS_COMMAND_QUEUE.with(|queue| {
        let commands = queue.borrow_mut().drain(..).collect::<Vec<_>>();
        
        for command in commands {
            match command {
                PhysicsCommand::ApplyForce { entity, force } => {
                    if let Some(handle) = physics_world.get_body_handle(Entity::from_bits(entity).unwrap()) {
                        if let Some(rb) = physics_world.rigid_body_set.get_mut(handle) {
                            rb.add_force(
                                vector![force.x as f64, force.y as f64, force.z as f64],
                                true,
                            );
                        }
                    }
                }
                PhysicsCommand::ApplyImpulse { entity, impulse } => {
                    if let Some(handle) = physics_world.get_body_handle(Entity::from_bits(entity).unwrap()) {
                        if let Some(rb) = physics_world.rigid_body_set.get_mut(handle) {
                            rb.apply_impulse(
                                vector![impulse.x as f64, impulse.y as f64, impulse.z as f64],
                                true,
                            );
                        }
                    }
                }
                PhysicsCommand::ApplyTorque { entity, torque } => {
                    if let Some(handle) = physics_world.get_body_handle(Entity::from_bits(entity).unwrap()) {
                        if let Some(rb) = physics_world.rigid_body_set.get_mut(handle) {
                            rb.add_torque(
                                vector![torque.x as f64, torque.y as f64, torque.z as f64],
                                true,
                            );
                        }
                    }
                }
                PhysicsCommand::SetVelocity { entity, linear, angular } => {
                    if let Some(handle) = physics_world.get_body_handle(Entity::from_bits(entity).unwrap()) {
                        if let Some(rb) = physics_world.rigid_body_set.get_mut(handle) {
                            rb.set_linvel(
                                vector![linear.x as f64, linear.y as f64, linear.z as f64],
                                true,
                            );
                            rb.set_angvel(
                                vector![angular.x as f64, angular.y as f64, angular.z as f64],
                                true,
                            );
                        }
                    }
                }
            }
        }
    });
}

/// Sync physics results back to transforms
fn sync_physics_to_transforms(world: &mut World, physics_world: &PhysicsWorld) {
    // Collect updates to avoid borrow conflicts
    let mut transform_updates = Vec::new();
    let mut world_transform_updates = Vec::new();
    
    // Query entities with physics
    for (entity, rb) in world.query::<&RigidBody>().iter() {
        if let Some(handle) = rb.handle {
            if let Some(rigid_body) = physics_world.rigid_body_set.get(handle) {
                // Only sync dynamic bodies (kinematic/fixed are controlled by transforms)
                if rb.body_type == RigidBodyType::Dynamic {
                    let pos = rigid_body.translation();
                    let rot = rigid_body.rotation();
                    
                    // Check if entity has WorldTransform or regular Transform
                    if world.get::<&WorldTransform>(entity).is_ok() {
                        world_transform_updates.push((
                            entity,
                            DVec3::new(pos.x, pos.y, pos.z),
                            Quat::from_xyzw(rot.i as f32, rot.j as f32, rot.k as f32, rot.w as f32),
                        ));
                    } else if world.get::<&Transform>(entity).is_ok() {
                        transform_updates.push((
                            entity,
                            Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                            Quat::from_xyzw(rot.i as f32, rot.j as f32, rot.k as f32, rot.w as f32),
                        ));
                    }
                }
            }
        }
    }
    
    // Apply transform updates
    for (entity, position, rotation) in transform_updates {
        if let Ok(mut transform) = world.get::<&mut Transform>(entity) {
            transform.position = position;
            transform.rotation = rotation;
        }
    }
    
    // Apply world transform updates
    for (entity, position, rotation) in world_transform_updates {
        if let Ok(mut world_transform) = world.get::<&mut WorldTransform>(entity) {
            world_transform.position = position;
            world_transform.rotation = rotation;
        }
    }
}

/// Sync physics velocities back to ECS
fn sync_physics_to_velocities(world: &mut World, physics_world: &PhysicsWorld) {
    let mut velocity_updates = Vec::new();
    
    for (entity, (_, rb)) in world.query::<(&mut PhysicsVelocity, &RigidBody)>().iter() {
        if let Some(handle) = rb.handle {
            if let Some(rigid_body) = physics_world.rigid_body_set.get(handle) {
                let linvel = rigid_body.linvel();
                let angvel = rigid_body.angvel();
                
                velocity_updates.push((
                    entity,
                    Vec3::new(linvel.x as f32, linvel.y as f32, linvel.z as f32),
                    Vec3::new(angvel.x as f32, angvel.y as f32, angvel.z as f32),
                ));
            }
        }
    }
    
    // Apply velocity updates
    for (entity, linear, angular) in velocity_updates {
        if let Ok(mut velocity) = world.get::<&mut PhysicsVelocity>(entity) {
            velocity.linear = linear;
            velocity.angular = angular;
        }
    }
}

/// Clean up physics resources for removed entities
fn cleanup_removed_entities(world: &World, physics_world: &mut PhysicsWorld) {
    // Check all registered bodies and clean up those whose entities no longer exist
    let mut entities_to_cleanup = Vec::new();
    
    for (entity, _) in &physics_world.entity_to_body.clone() {
        if !world.contains(*entity) {
            entities_to_cleanup.push(*entity);
        }
    }
    
    for entity in entities_to_cleanup {
        physics_world.cleanup_entity(entity);
        debug!("Cleaned up physics resources for removed entity {:?}", entity);
    }
}

/// Queue a physics command for execution in the next physics update
pub fn queue_physics_command(command: crate::physics::PhysicsCommand) {
    PHYSICS_COMMAND_QUEUE.with(|queue| {
        queue.borrow_mut().push(command);
    });
}