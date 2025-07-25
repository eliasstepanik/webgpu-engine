//! Physics system integration for the engine

use crate::core::entity::{Transform, World};
use crate::physics::{
    avbd_solver::{AVBDConfig, AVBDSolver, RigidbodyData},
    collision::{
        broad_phase::{sweep_and_prune, BroadPhaseEntry},
        narrow_phase::test_collision,
        Contact,
    },
    components::{Collider, CollisionShape, PhysicsMaterial, Rigidbody},
    constraints::ContactConstraint,
    PhysicsConfig,
};
use glam::{Quat, Vec3};
use hecs::Entity;
use std::collections::HashMap;
use tracing::{debug, info, trace};

/// Physics accumulator for fixed timestep
static mut PHYSICS_ACCUMULATOR: f32 = 0.0;

/// Main physics update system with fixed timestep
pub fn update_physics_system(
    world: &mut World,
    solver: &mut AVBDSolver,
    config: &PhysicsConfig,
    delta_time: f32,
) {
    trace!("Starting physics update with dt={}", delta_time);

    // Use fixed timestep with interpolation for stability
    unsafe {
        PHYSICS_ACCUMULATOR += delta_time;

        // Run physics in fixed timesteps
        while PHYSICS_ACCUMULATOR >= config.fixed_timestep {
            // Store previous transforms for interpolation
            store_previous_transforms(world);

            // Run the actual AVBD physics
            update_physics_system_avbd(world, solver, config.fixed_timestep);

            PHYSICS_ACCUMULATOR -= config.fixed_timestep;
        }

        // Interpolate positions for smooth rendering
        let alpha = PHYSICS_ACCUMULATOR / config.fixed_timestep;
        interpolate_transforms(world, alpha);
    }
}

/// AVBD physics update system (currently broken, needs fixes)
#[allow(dead_code)]
pub fn update_physics_system_avbd(world: &mut World, solver: &mut AVBDSolver, delta_time: f32) {
    trace!("Starting physics update, dt={}", delta_time);

    // Skip if delta time is too small
    if delta_time < 0.0001 {
        return;
    }

    // 0. Update hierarchy to ensure GlobalTransform components exist
    crate::core::entity::hierarchy::advance_frame();
    crate::core::entity::update_hierarchy_system(world);

    // 1. Gather physics entities
    let (mut bodies, body_entity_map) = gather_rigidbodies(world);
    if bodies.is_empty() {
        trace!("No rigidbodies found in world");
        return;
    }
    debug!("Found {} rigidbodies", bodies.len());

    // 2. Gather colliders
    let colliders = gather_colliders(world, &body_entity_map);
    debug!("Gathered {} colliders", colliders.len());

    // Debug: Print body and collider details
    for (i, body) in bodies.iter().enumerate() {
        trace!(
            "Body {}: entity={:?}, pos={:?}, vel={:?}",
            i,
            body.entity,
            body.position,
            body.linear_velocity
        );
    }
    for (i, collider) in colliders.iter().enumerate() {
        trace!(
            "Collider {}: entity={:?}, pos={:?}, body_idx={:?}",
            i,
            collider.entity,
            collider.position,
            collider.body_index
        );
    }

    // 3. Apply damping
    apply_damping(&mut bodies, delta_time);

    // 4. Detect collisions
    let contacts = detect_all_collisions(&colliders, &bodies);
    debug!("Detected {} contacts", contacts.len());

    // 5. Create contact constraints
    solver.constraints.clear();
    debug!("Creating constraints from {} contacts", contacts.len());
    for (i, contact) in contacts.iter().enumerate() {
        // Get material properties
        let material = get_contact_material(world, contact);

        // Find body indices - handle static bodies
        let body_a_idx = body_entity_map.get(&contact.entity_a).copied();
        let body_b_idx = body_entity_map.get(&contact.entity_b).copied();

        debug!(
            "Contact {}: entities {:?} <-> {:?}, body indices {:?} <-> {:?}",
            i, contact.entity_a, contact.entity_b, body_a_idx, body_b_idx
        );
        debug!(
            "  Position: {:?}, Normal: {:?}, Penetration: {}",
            contact.position, contact.normal, contact.penetration
        );

        // Skip if neither entity has a rigidbody
        if body_a_idx.is_none() && body_b_idx.is_none() {
            debug!("  Skipping - no rigidbodies found");
            continue;
        }

        // Create constraint - handle static bodies by using special index
        let constraint = ContactConstraint::new_with_optional_bodies(
            contact.clone(),
            body_a_idx,
            body_b_idx,
            &bodies,
            material.as_ref(),
            delta_time,
        );
        solver.constraints.push(Box::new(constraint));
        debug!(
            "  Created constraint, total constraints: {}",
            solver.constraints.len()
        );
    }

    // 6. Update vertex coloring for parallelization
    solver.update_coloring(&bodies);

    // 7. Run AVBD solver
    debug!(
        "Running AVBD solver with {} constraints and {} bodies",
        solver.constraints.len(),
        bodies.len()
    );
    solver.step(&mut bodies, delta_time);
    debug!("AVBD solver step completed");

    // 8. Write back to components
    update_transforms(world, &bodies);

    trace!("Physics update complete");
}

/// Gather all rigidbodies from the world
pub fn gather_rigidbodies(world: &World) -> (Vec<RigidbodyData>, HashMap<Entity, usize>) {
    let mut bodies = Vec::new();
    let mut entity_map = HashMap::new();

    // Query entities with Rigidbody and GlobalTransform
    for (entity, (rigidbody, _transform, global_transform)) in world
        .query::<(
            &Rigidbody,
            &Transform,
            &crate::core::entity::components::GlobalTransform,
        )>()
        .iter()
    {
        let idx = bodies.len();
        entity_map.insert(entity, idx);

        // Use global transform for physics
        let (_, rotation, position) = global_transform.matrix.to_scale_rotation_translation();

        // Create rigidbody data
        let mut data = RigidbodyData::new(
            entity,
            position,
            rotation,
            rigidbody.linear_velocity,
            rigidbody.angular_velocity,
            rigidbody.mass,
            rigidbody.inertia_tensor,
            rigidbody.use_gravity,
            rigidbody.is_kinematic,
            rigidbody.linear_damping,
            rigidbody.angular_damping,
        );

        // If there's a collider, update inertia based on shape
        if let Ok(collider) = world.get::<Collider>(entity) {
            let shape_inertia = collider.shape.calculate_inertia(rigidbody.mass);
            data.inertia_local = shape_inertia;
        }

        bodies.push(data);
    }

    info!("Gathered {} rigidbodies", bodies.len());
    (bodies, entity_map)
}

/// Gather all colliders with their transforms
pub fn gather_colliders(world: &World, body_map: &HashMap<Entity, usize>) -> Vec<ColliderEntry> {
    let mut colliders = Vec::new();

    // Use GlobalTransform to get the actual world position/rotation/scale
    for (entity, (collider, _transform, global_transform)) in world
        .query::<(
            &Collider,
            &Transform,
            &crate::core::entity::components::GlobalTransform,
        )>()
        .iter()
    {
        // Extract position, rotation, and scale from global transform
        let (scale, rotation, position) = global_transform.matrix.to_scale_rotation_translation();

        // Apply scale to the collider shape
        let scaled_collider = scale_collider(collider, scale);

        colliders.push(ColliderEntry {
            entity,
            collider: scaled_collider,
            position,
            rotation,
            // Use usize::MAX for static colliders (no rigidbody)
            body_index: body_map.get(&entity).copied().unwrap_or(usize::MAX),
        });
    }

    colliders
}

/// Apply scale to a collider shape
fn scale_collider(collider: &Collider, scale: Vec3) -> Collider {
    let scaled_shape = match &collider.shape {
        CollisionShape::Box { half_extents } => CollisionShape::Box {
            half_extents: *half_extents * scale,
        },
        CollisionShape::Sphere { radius } => CollisionShape::Sphere {
            // Use the maximum scale component for sphere to maintain shape
            radius: *radius * scale.max_element(),
        },
        CollisionShape::Capsule {
            radius,
            half_height,
        } => CollisionShape::Capsule {
            // Use XZ scale for radius, Y scale for height
            radius: *radius * scale.x.max(scale.z),
            half_height: *half_height * scale.y,
        },
    };

    Collider {
        shape: scaled_shape,
        is_trigger: collider.is_trigger,
        material_id: collider.material_id,
    }
}

/// Entry for collision detection
pub struct ColliderEntry {
    pub entity: Entity,
    pub collider: Collider,
    pub position: Vec3,
    pub rotation: Quat,
    pub body_index: usize,
}

/// Apply damping to all bodies
fn apply_damping(bodies: &mut [RigidbodyData], _dt: f32) {
    for body in bodies {
        if !body.is_kinematic {
            // Apply linear damping directly from the rigidbody component
            // This is already stored in the RigidbodyData from gather_rigidbodies
            // The damping is applied in the AVBD solver's update_inertial step
            // So we don't need to apply it here to avoid double damping
        }
    }
}

/// Detect all collisions
pub fn detect_all_collisions(
    colliders: &[ColliderEntry],
    bodies: &[RigidbodyData],
) -> Vec<Contact> {
    if colliders.is_empty() {
        return Vec::new();
    }

    // Build broad phase entries
    let broad_entries: Vec<BroadPhaseEntry> = colliders
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let aabb = entry
                .collider
                .shape
                .world_aabb(entry.position, entry.rotation);
            trace!(
                "AABB for collider {}: min={:?}, max={:?}",
                idx,
                aabb.min,
                aabb.max
            );
            BroadPhaseEntry {
                entity: entry.entity,
                aabb,
            }
        })
        .collect();

    // Broad phase
    let pairs = sweep_and_prune(&broad_entries);
    debug!(
        "Broad phase found {} potential collision pairs: {:?}",
        pairs.len(),
        pairs
    );

    // Narrow phase
    let mut contacts = Vec::new();
    for (i, j) in pairs {
        let entry_a = &colliders[i];
        let entry_b = &colliders[j];

        // Skip trigger colliders
        if entry_a.collider.is_trigger || entry_b.collider.is_trigger {
            continue;
        }

        // Check if either is static (no rigidbody)
        let is_static_a = entry_a.body_index == usize::MAX;
        let is_static_b = entry_b.body_index == usize::MAX;

        // Skip if both are static
        if is_static_a && is_static_b {
            continue;
        }

        // Skip if both are kinematic
        if !is_static_a && !is_static_b {
            let body_a = &bodies[entry_a.body_index];
            let body_b = &bodies[entry_b.body_index];
            if body_a.is_kinematic && body_b.is_kinematic {
                continue;
            }
        }

        // Test collision
        debug!(
            "Testing collision: {:?} vs {:?}",
            entry_a.entity, entry_b.entity
        );
        if let Some(contact) = test_collision(
            &entry_a.collider.shape,
            (entry_a.position, entry_a.rotation),
            entry_a.entity,
            &entry_b.collider.shape,
            (entry_b.position, entry_b.rotation),
            entry_b.entity,
        ) {
            debug!(
                "Contact detected: pos={:?}, normal={:?}, penetration={}",
                contact.position, contact.normal, contact.penetration
            );
            contacts.push(contact);
        } else {
            debug!(
                "No contact between {:?} and {:?}",
                entry_a.entity, entry_b.entity
            );
        }
    }

    contacts
}

/// Get the physics material for a contact
fn get_contact_material(world: &World, contact: &Contact) -> Option<PhysicsMaterial> {
    // Try to get material from either collider
    let mat_a = world
        .get::<Collider>(contact.entity_a)
        .ok()
        .and_then(|collider| collider.material_id)
        .and_then(Entity::from_bits)
        .and_then(|entity| world.get::<PhysicsMaterial>(entity).ok())
        .map(|mat| (*mat).clone());

    let mat_b = world
        .get::<Collider>(contact.entity_b)
        .ok()
        .and_then(|collider| collider.material_id)
        .and_then(Entity::from_bits)
        .and_then(|entity| world.get::<PhysicsMaterial>(entity).ok())
        .map(|mat| (*mat).clone());

    // Combine materials (average properties)
    match (mat_a, mat_b) {
        (Some(a), Some(b)) => Some(PhysicsMaterial {
            static_friction: (a.static_friction + b.static_friction) * 0.5,
            dynamic_friction: (a.dynamic_friction + b.dynamic_friction) * 0.5,
            restitution: (a.restitution + b.restitution) * 0.5,
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Update transform components from physics data
fn update_transforms(world: &mut World, bodies: &[RigidbodyData]) {
    for body in bodies {
        // Pre-calculate transform values to avoid borrow conflicts
        let (local_position, local_rotation) = {
            // Check if this entity has a parent
            if let Ok(parent_entity) =
                world.get::<crate::core::entity::components::Parent>(body.entity)
            {
                // Get parent's global transform to convert world position to local
                if let Ok(parent_global) =
                    world.get::<crate::core::entity::components::GlobalTransform>(parent_entity.0)
                {
                    // Convert world position/rotation to parent-relative local coordinates
                    let parent_world_matrix = parent_global.matrix;
                    let parent_inverse = parent_world_matrix.inverse();

                    // Transform world position to parent's local space
                    let local_position = parent_inverse.transform_point3(body.position);

                    // For rotation, we need to remove parent's rotation
                    let (_, parent_rotation, _) =
                        parent_world_matrix.to_scale_rotation_translation();
                    let local_rotation = parent_rotation.conjugate() * body.rotation;

                    (local_position, local_rotation)
                } else {
                    // Parent doesn't have GlobalTransform - fall back to world coordinates
                    (body.position, body.rotation)
                }
            } else {
                // No parent - world coordinates = local coordinates
                (body.position, body.rotation)
            }
        };

        // Update Transform with pre-calculated values
        if let Ok((transform,)) = world.query_one_mut::<(&mut Transform,)>(body.entity) {
            transform.position = local_position;
            transform.rotation = local_rotation;
        }

        // Update Rigidbody velocities
        if let Ok((rigidbody,)) = world.query_one_mut::<(&mut Rigidbody,)>(body.entity) {
            rigidbody.linear_velocity = body.linear_velocity;
            rigidbody.angular_velocity = body.angular_velocity;
        }
    }
}

/// Store previous transforms for interpolation
fn store_previous_transforms(world: &mut World) {
    // Store current positions as previous for interpolation
    // Only process entities with Rigidbody components
    for (entity, (transform, _rigidbody)) in world.query::<(&Transform, &Rigidbody)>().iter() {
        // Store in a component or temporary storage
        // For now, we'll skip interpolation implementation
        // This would require adding a PreviousTransform component
        let _ = (entity, transform);
    }
}

/// Interpolate transforms between physics steps for smooth rendering
fn interpolate_transforms(world: &mut World, alpha: f32) {
    // Interpolate between previous and current transforms
    // Only process entities with Rigidbody components
    // For now, we'll skip interpolation implementation
    // This would require the PreviousTransform component
    let _ = (world, alpha);
}

/// Create a physics solver with configuration
pub fn create_physics_solver(config: &PhysicsConfig) -> AVBDSolver {
    let avbd_config = AVBDConfig {
        iterations: config.velocity_iterations,
        beta: 10.0,      // Stiffness ramping speed
        alpha: 0.98,     // Error correction factor
        gamma: 0.99,     // Warmstart decay
        k_start: 5000.0, // Initial stiffness
        gravity: config.gravity,
    };
    AVBDSolver::with_physics_config(avbd_config, config)
}

/// Create a default physics solver
pub fn create_default_solver() -> AVBDSolver {
    create_physics_solver(&PhysicsConfig::default())
}

/// Add a ball joint between two entities
pub fn add_ball_joint(
    solver: &mut AVBDSolver,
    world: &World,
    entity_a: Entity,
    entity_b: Entity,
    world_anchor: Vec3,
) -> Result<(), String> {
    // Find body indices
    let mut body_a_idx = None;
    let mut body_b_idx = None;
    let mut bodies = Vec::new();

    for (entity, (rb, transform)) in world.query::<(&Rigidbody, &Transform)>().iter() {
        let idx = bodies.len();
        if entity == entity_a {
            body_a_idx = Some(idx);
        }
        if entity == entity_b {
            body_b_idx = Some(idx);
        }

        bodies.push(RigidbodyData::new(
            entity,
            transform.position,
            transform.rotation,
            rb.linear_velocity,
            rb.angular_velocity,
            rb.mass,
            rb.inertia_tensor,
            rb.use_gravity,
            rb.is_kinematic,
            rb.linear_damping,
            rb.angular_damping,
        ));
    }

    match (body_a_idx, body_b_idx) {
        (Some(a), Some(b)) => {
            use crate::physics::constraints::BallJoint;
            let joint = BallJoint::new(a, b, world_anchor, &bodies);
            solver.constraints.push(Box::new(joint));
            Ok(())
        }
        _ => Err("One or both entities don't have rigidbodies".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::World;

    #[test]
    fn test_physics_system_empty() {
        let mut world = World::new();
        let mut solver = create_default_solver();

        // Should not crash with empty world
        let config = PhysicsConfig::default();
        update_physics_system(&mut world, &mut solver, &config, 0.016);
    }

    #[test]
    fn test_rigidbody_gathering() {
        let mut world = World::new();

        // Add a rigidbody with GlobalTransform (required for physics queries)
        world.spawn((
            Transform::default(),
            crate::core::entity::components::GlobalTransform::default(),
            Rigidbody::default(),
        ));

        let (bodies, map) = gather_rigidbodies(&world);
        assert_eq!(bodies.len(), 1);
        assert_eq!(map.len(), 1);
    }
}
