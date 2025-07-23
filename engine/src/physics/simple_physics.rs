//! Simple explicit physics integration for comparison

use crate::core::entity::World;
use crate::physics::{
    collision::{
        broad_phase::{sweep_and_prune, BroadPhaseEntry},
        narrow_phase::test_collision,
    },
    components::{Collider, CollisionShape, Rigidbody},
};
use crate::prelude::Transform;
use glam::{Mat3, Vec3};
use tracing::debug;

/// Simple explicit Euler physics update
pub fn simple_physics_update(world: &mut World, config: &crate::physics::PhysicsConfig, dt: f32) {
    // Sub-step for stability with large time steps
    let substeps = (dt / config.fixed_timestep).ceil() as u32;
    let substep_dt = dt / substeps as f32;

    debug!(
        "Simple physics update: dt={}, substeps={}, substep_dt={}",
        dt, substeps, substep_dt
    );

    for _ in 0..substeps {
        simple_physics_substep(world, config, substep_dt);
    }
}

/// Perform a single physics substep
fn simple_physics_substep(world: &mut World, config: &crate::physics::PhysicsConfig, dt: f32) {
    let gravity = config.gravity;

    // First pass: update velocities and positions
    let mut updates = Vec::new();

    for (entity, (rigidbody, transform)) in world.query::<(&Rigidbody, &Transform)>().iter() {
        if rigidbody.is_kinematic {
            continue;
        }

        // Apply gravity
        let acceleration = if rigidbody.use_gravity {
            gravity
        } else {
            Vec3::ZERO
        };

        // Update velocity (v = v + a * dt)
        let new_velocity = rigidbody.linear_velocity + acceleration * dt;

        // Apply damping
        let damped_velocity = new_velocity * (1.0 - rigidbody.linear_damping).powf(dt);

        // Clamp velocity to prevent tunneling
        let clamped_velocity = if damped_velocity.length() > config.max_linear_velocity {
            damped_velocity.normalize() * config.max_linear_velocity
        } else {
            damped_velocity
        };

        // Update position (x = x + v * dt)
        // Only update position if velocity is non-zero to prevent drift
        let new_position = if clamped_velocity.length_squared() > 1e-6 {
            transform.position + clamped_velocity * dt
        } else {
            transform.position
        };

        // Update angular velocity with damping
        let damped_angular_velocity =
            rigidbody.angular_velocity * (1.0 - rigidbody.angular_damping).powf(dt);

        // Update rotation (integrate angular velocity)
        let rotation_change = if damped_angular_velocity.length_squared() > 1e-6 {
            let angle = damped_angular_velocity.length() * dt;
            let axis = damped_angular_velocity.normalize();
            glam::Quat::from_axis_angle(axis, angle)
        } else {
            glam::Quat::IDENTITY
        };
        let new_rotation = (rotation_change * transform.rotation).normalize();

        updates.push((
            entity,
            new_position,
            clamped_velocity,
            new_rotation,
            damped_angular_velocity,
        ));
    }

    // Apply position and rotation updates
    for (entity, new_position, new_velocity, new_rotation, new_angular_velocity) in updates {
        if let Ok(transform) = world.query_one_mut::<&mut Transform>(entity) {
            transform.position = new_position;
            transform.rotation = new_rotation;
        }
        if let Ok(rigidbody) = world.query_one_mut::<&mut Rigidbody>(entity) {
            rigidbody.linear_velocity = new_velocity;
            rigidbody.angular_velocity = new_angular_velocity;
        }
    }

    // Update GlobalTransform after position changes - MUST be before collision detection
    // Need to advance frame for GlobalTransform to update
    crate::core::entity::hierarchy::advance_frame();
    crate::core::entity::update_hierarchy_system(world);

    // Second pass: detect and resolve collisions
    detect_and_resolve_collisions(world, config, dt);

    // Update GlobalTransform again after collision resolution
    crate::core::entity::hierarchy::advance_frame();
    crate::core::entity::update_hierarchy_system(world);
}

/// Detect and resolve collisions
fn detect_and_resolve_collisions(
    world: &mut World,
    config: &crate::physics::PhysicsConfig,
    _dt: f32,
) {
    // Gather all colliders with their transforms
    let mut collider_data = Vec::new();
    debug!("Starting collision detection");

    for (entity, (collider, transform)) in world.query::<(&Collider, &Transform)>().iter() {
        // Use GlobalTransform if available, otherwise use Transform
        let (scale, rotation, position) = if let Ok(global_transform) =
            world.get::<crate::core::entity::GlobalTransform>(entity)
        {
            global_transform.matrix.to_scale_rotation_translation()
        } else {
            (transform.scale, transform.rotation, transform.position)
        };

        debug!(
            "Entity {:?} transform: local_pos={:?}, global_pos={:?}",
            entity, transform.position, position
        );

        // Apply scale to collider
        let scaled_shape = match &collider.shape {
            CollisionShape::Box { half_extents } => CollisionShape::Box {
                half_extents: *half_extents * scale,
            },
            CollisionShape::Sphere { radius } => CollisionShape::Sphere {
                radius: *radius * scale.max_element(),
            },
            CollisionShape::Capsule {
                radius,
                half_height,
            } => CollisionShape::Capsule {
                radius: *radius * scale.x.max(scale.z),
                half_height: *half_height * scale.y,
            },
        };

        collider_data.push((
            entity,
            position,
            rotation,
            scaled_shape,
            collider.is_trigger,
        ));
    }

    debug!("Found {} colliders", collider_data.len());

    // Broad phase
    let broad_entries: Vec<_> = collider_data
        .iter()
        .map(|(entity, pos, rot, shape, _)| {
            let aabb = shape.world_aabb(*pos, *rot);
            debug!(
                "Entity {:?}: pos={:?}, aabb_min={:?}, aabb_max={:?}",
                entity, pos, aabb.min, aabb.max
            );
            BroadPhaseEntry {
                entity: *entity,
                aabb,
            }
        })
        .collect();

    let pairs = sweep_and_prune(&broad_entries);
    debug!(
        "Broad phase found {} potential collision pairs",
        pairs.len()
    );

    // Narrow phase and resolution
    for (i, j) in pairs {
        let (entity_a, pos_a, rot_a, shape_a, is_trigger_a) = &collider_data[i];
        let (entity_b, pos_b, rot_b, shape_b, is_trigger_b) = &collider_data[j];

        // Skip if both are triggers
        if *is_trigger_a && *is_trigger_b {
            continue;
        }

        // Skip if both are static (no rigidbody)
        // FIXED: Use proper query_one syntax
        let has_rb_a = world
            .query_one::<&Rigidbody>(*entity_a)
            .map(|mut q| q.get().is_some())
            .unwrap_or(false);
        let has_rb_b = world
            .query_one::<&Rigidbody>(*entity_b)
            .map(|mut q| q.get().is_some())
            .unwrap_or(false);
        debug!(
            "Testing collision: entity_a={:?} has_rb={}, entity_b={:?} has_rb={}",
            entity_a, has_rb_a, entity_b, has_rb_b
        );

        if !has_rb_a && !has_rb_b {
            continue;
        }

        // Test collision
        if let Some(contact) = test_collision(
            shape_a,
            (*pos_a, *rot_a),
            *entity_a,
            shape_b,
            (*pos_b, *rot_b),
            *entity_b,
        ) {
            debug!(
                "Collision detected between {:?} and {:?}, penetration: {}",
                entity_a, entity_b, contact.penetration
            );

            // The contact normal points from A to B, which means:
            // - For A to move away from B, A should move in the -normal direction
            // - For B to move away from A, B should move in the +normal direction

            // Get masses first
            let mass_a = if has_rb_a {
                world
                    .query_one::<&Rigidbody>(*entity_a)
                    .map(|mut q| q.get().map(|rb| rb.mass).unwrap_or(f32::INFINITY))
                    .unwrap_or(f32::INFINITY)
            } else {
                f32::INFINITY // Static object has infinite mass
            };

            let mass_b = if has_rb_b {
                world
                    .query_one::<&Rigidbody>(*entity_b)
                    .map(|mut q| q.get().map(|rb| rb.mass).unwrap_or(f32::INFINITY))
                    .unwrap_or(f32::INFINITY)
            } else {
                f32::INFINITY // Static object has infinite mass
            };

            // Penetration recovery with bias to prevent sinking
            let bias = 0.8; // Baumgarte stabilization factor
            let _slop = 0.0005; // Penetration slop to prevent jitter (very small for accuracy)
            let correction_amount = contact.penetration * bias; // Don't subtract slop for better accuracy

            debug!(
                "Collision: A={:?} B={:?}, normal={:?}, pen={:.4}",
                entity_a, entity_b, contact.normal, contact.penetration
            );
            debug!("  A: pos={:?}, has_rb={}", pos_a, has_rb_a);
            debug!("  B: pos={:?}, has_rb={}", pos_b, has_rb_b);
            debug!("  Contact point: {:?}", contact.position);

            // Position correction based on masses
            let inv_mass_a = if has_rb_a { 1.0 / mass_a } else { 0.0 };
            let inv_mass_b = if has_rb_b { 1.0 / mass_b } else { 0.0 };

            let total_inv_mass = inv_mass_a + inv_mass_b;
            if total_inv_mass > 0.0 && contact.penetration > 0.0 {
                // For ground collisions, ensure objects are pushed up
                let correction_normal = if !has_rb_a && has_rb_b && contact.normal.y < -0.5 {
                    // A is static ground, B is dynamic object, flip normal to push B up
                    -contact.normal
                } else if has_rb_a && !has_rb_b && contact.normal.y > 0.5 {
                    // B is static ground, A is dynamic object, normal already pushes A up
                    contact.normal
                } else {
                    contact.normal
                };

                // Always apply full correction for resting objects
                let is_resting_a = has_rb_a
                    && world
                        .query_one::<&Rigidbody>(*entity_a)
                        .map(|mut q| {
                            q.get()
                                .map(|rb| rb.linear_velocity.length() < 0.1)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                let is_resting_b = has_rb_b
                    && world
                        .query_one::<&Rigidbody>(*entity_b)
                        .map(|mut q| {
                            q.get()
                                .map(|rb| rb.linear_velocity.length() < 0.1)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);

                let final_correction = if is_resting_a || is_resting_b {
                    contact.penetration // Full correction for resting objects
                } else {
                    correction_amount // Partial correction for moving objects
                };

                // Move objects apart based on their inverse mass ratio
                if has_rb_a {
                    if let Ok(transform_a) = world.query_one_mut::<&mut Transform>(*entity_a) {
                        let move_ratio = inv_mass_a / total_inv_mass;
                        transform_a.position -= correction_normal * final_correction * move_ratio;
                    }
                }
                if has_rb_b {
                    if let Ok(transform_b) = world.query_one_mut::<&mut Transform>(*entity_b) {
                        let move_ratio = inv_mass_b / total_inv_mass;
                        transform_b.position += correction_normal * final_correction * move_ratio;
                    }
                }
            }

            // Improved velocity response
            let restitution = 0.3; // Coefficient of restitution

            // Get velocities (linear + angular at contact point) and transforms
            let (vel_a, transform_a) = if has_rb_a {
                world
                    .query_one::<(&Rigidbody, &Transform)>(*entity_a)
                    .map(|mut q| {
                        if let Some((rb, t)) = q.get() {
                            let r = contact.position - t.position;
                            let vel_at_contact = rb.linear_velocity + rb.angular_velocity.cross(r);
                            (vel_at_contact, *t)
                        } else {
                            (Vec3::ZERO, Transform::default())
                        }
                    })
                    .unwrap_or((Vec3::ZERO, Transform::default()))
            } else {
                (Vec3::ZERO, Transform::default())
            };

            let (vel_b, transform_b) = if has_rb_b {
                world
                    .query_one::<(&Rigidbody, &Transform)>(*entity_b)
                    .map(|mut q| {
                        if let Some((rb, t)) = q.get() {
                            let r = contact.position - t.position;
                            let vel_at_contact = rb.linear_velocity + rb.angular_velocity.cross(r);
                            (vel_at_contact, *t)
                        } else {
                            (Vec3::ZERO, Transform::default())
                        }
                    })
                    .unwrap_or((Vec3::ZERO, Transform::default()))
            } else {
                (Vec3::ZERO, Transform::default())
            };

            // Calculate relative velocity
            let relative_velocity = vel_a - vel_b;
            let velocity_along_normal = relative_velocity.dot(contact.normal);

            // Don't resolve if velocities are separating
            if velocity_along_normal > 0.0 {
                debug!("Velocities are already separating, skipping collision response");
                continue;
            }

            // Calculate impulse scalar with angular effects
            let r_a = contact.position - transform_a.position;
            let r_b = contact.position - transform_b.position;

            // Get inverse inertia tensors
            let inv_inertia_a = if has_rb_a {
                world
                    .query_one::<&Rigidbody>(*entity_a)
                    .map(|mut q| {
                        q.get()
                            .map(|rb| rb.inertia_tensor.inverse())
                            .unwrap_or(Mat3::ZERO)
                    })
                    .unwrap_or(Mat3::ZERO)
            } else {
                Mat3::ZERO
            };

            let inv_inertia_b = if has_rb_b {
                world
                    .query_one::<&Rigidbody>(*entity_b)
                    .map(|mut q| {
                        q.get()
                            .map(|rb| rb.inertia_tensor.inverse())
                            .unwrap_or(Mat3::ZERO)
                    })
                    .unwrap_or(Mat3::ZERO)
            } else {
                Mat3::ZERO
            };

            // Calculate effective mass including rotational effects
            let angular_factor_a = (inv_inertia_a * r_a.cross(contact.normal))
                .cross(r_a)
                .dot(contact.normal);
            let angular_factor_b = (inv_inertia_b * r_b.cross(contact.normal))
                .cross(r_b)
                .dot(contact.normal);
            let effective_mass = total_inv_mass + angular_factor_a + angular_factor_b;

            let impulse_scalar = -(1.0 + restitution) * velocity_along_normal / effective_mass;
            let impulse = contact.normal * impulse_scalar;

            // Apply impulse to bodies
            if has_rb_a && inv_mass_a > 0.0 {
                if let Ok((rb_a, transform_a)) =
                    world.query_one_mut::<(&mut Rigidbody, &Transform)>(*entity_a)
                {
                    rb_a.linear_velocity += impulse * inv_mass_a;

                    // Calculate torque from off-center collision
                    let r_a = contact.position - transform_a.position; // Vector from center to contact point
                    let torque_impulse = r_a.cross(impulse);
                    let inv_inertia_a = rb_a.inertia_tensor.inverse();
                    rb_a.angular_velocity += inv_inertia_a * torque_impulse;

                    // Apply velocity threshold to prevent micro-bouncing
                    let vertical_vel = rb_a.linear_velocity.y.abs();

                    // If vertical velocity is low and we're near the ground, zero it out
                    if vertical_vel < config.rest_velocity_threshold && contact.normal.y.abs() > 0.8
                    {
                        rb_a.linear_velocity.y = 0.0;
                    }

                    // If overall velocity is very low, zero it completely
                    if rb_a.linear_velocity.length() < config.rest_velocity_threshold * 0.5 {
                        rb_a.linear_velocity = Vec3::ZERO;
                    }

                    // Also dampen angular velocity when resting
                    if rb_a.linear_velocity.length() < config.rest_velocity_threshold * 0.5
                        && rb_a.angular_velocity.length() < 0.1
                    {
                        rb_a.angular_velocity = Vec3::ZERO;
                    }
                }
            }

            if has_rb_b && inv_mass_b > 0.0 {
                if let Ok((rb_b, transform_b)) =
                    world.query_one_mut::<(&mut Rigidbody, &Transform)>(*entity_b)
                {
                    rb_b.linear_velocity -= impulse * inv_mass_b;

                    // Calculate torque from off-center collision
                    let r_b = contact.position - transform_b.position; // Vector from center to contact point
                    let torque_impulse = r_b.cross(-impulse); // Negative because impulse is in opposite direction for B
                    let inv_inertia_b = rb_b.inertia_tensor.inverse();
                    rb_b.angular_velocity += inv_inertia_b * torque_impulse;

                    // Apply velocity threshold to prevent micro-bouncing
                    let vertical_vel = rb_b.linear_velocity.y.abs();

                    // If vertical velocity is low and we're near the ground, zero it out
                    if vertical_vel < config.rest_velocity_threshold && contact.normal.y.abs() > 0.8
                    {
                        rb_b.linear_velocity.y = 0.0;
                    }

                    // If overall velocity is very low, zero it completely
                    if rb_b.linear_velocity.length() < config.rest_velocity_threshold * 0.5 {
                        rb_b.linear_velocity = Vec3::ZERO;
                    }

                    // Also dampen angular velocity when resting
                    if rb_b.linear_velocity.length() < config.rest_velocity_threshold * 0.5
                        && rb_b.angular_velocity.length() < 0.1
                    {
                        rb_b.angular_velocity = Vec3::ZERO;
                    }
                }
            }
        }
    }
}
