//! Narrow phase collision detection for generating contact points

use super::Contact;
use crate::physics::components::CollisionShape;
use glam::{Quat, Vec3};
use hecs::Entity;

/// Test for collision between two shapes and generate contact information
pub fn test_collision(
    shape_a: &CollisionShape,
    transform_a: (Vec3, Quat),
    entity_a: Entity,
    shape_b: &CollisionShape,
    transform_b: (Vec3, Quat),
    entity_b: Entity,
) -> Option<Contact> {
    match (shape_a, shape_b) {
        (
            CollisionShape::Sphere { radius: radius_a },
            CollisionShape::Sphere { radius: radius_b },
        ) => sphere_sphere_collision(
            transform_a.0,
            *radius_a,
            entity_a,
            transform_b.0,
            *radius_b,
            entity_b,
        ),
        (CollisionShape::Sphere { radius }, CollisionShape::Box { half_extents }) => {
            sphere_box_collision(
                transform_a.0,
                *radius,
                entity_a,
                transform_b.0,
                transform_b.1,
                *half_extents,
                entity_b,
            )
        }
        (CollisionShape::Box { half_extents }, CollisionShape::Sphere { radius }) => {
            sphere_box_collision(
                transform_b.0,
                *radius,
                entity_b,
                transform_a.0,
                transform_a.1,
                *half_extents,
                entity_a,
            )
            .map(|contact| contact.flipped())
        }
        (
            CollisionShape::Box {
                half_extents: extents_a,
            },
            CollisionShape::Box {
                half_extents: extents_b,
            },
        ) => box_box_collision(
            transform_a.0,
            transform_a.1,
            *extents_a,
            entity_a,
            transform_b.0,
            transform_b.1,
            *extents_b,
            entity_b,
        ),
        // TODO: Implement capsule collisions
        _ => None,
    }
}

/// Test collision between two spheres
fn sphere_sphere_collision(
    pos_a: Vec3,
    radius_a: f32,
    entity_a: Entity,
    pos_b: Vec3,
    radius_b: f32,
    entity_b: Entity,
) -> Option<Contact> {
    let delta = pos_b - pos_a;
    let distance_sq = delta.length_squared();
    let radius_sum = radius_a + radius_b;

    if distance_sq > radius_sum * radius_sum {
        return None;
    }

    let distance = distance_sq.sqrt();
    let normal = if distance > 0.0 {
        delta / distance
    } else {
        // Spheres are at the same position, use arbitrary normal
        Vec3::Y
    };

    let penetration = radius_sum - distance;
    let contact_point = pos_a + normal * (radius_a - penetration * 0.5);

    Some(Contact::new(
        entity_a,
        entity_b,
        contact_point,
        normal,
        penetration,
    ))
}

/// Test collision between a sphere and a box
fn sphere_box_collision(
    sphere_pos: Vec3,
    sphere_radius: f32,
    sphere_entity: Entity,
    box_pos: Vec3,
    box_rot: Quat,
    box_half_extents: Vec3,
    box_entity: Entity,
) -> Option<Contact> {
    // Transform sphere to box's local space
    let local_sphere_pos = box_rot.conjugate() * (sphere_pos - box_pos);

    // Find closest point on box to sphere center
    let closest = Vec3::new(
        local_sphere_pos
            .x
            .clamp(-box_half_extents.x, box_half_extents.x),
        local_sphere_pos
            .y
            .clamp(-box_half_extents.y, box_half_extents.y),
        local_sphere_pos
            .z
            .clamp(-box_half_extents.z, box_half_extents.z),
    );

    let delta = local_sphere_pos - closest;
    let distance_sq = delta.length_squared();

    if distance_sq > sphere_radius * sphere_radius {
        return None;
    }

    let distance = distance_sq.sqrt();
    let local_normal = if distance > 0.0 {
        delta / distance
    } else {
        // Sphere center is inside box, find closest face
        let face_distances = [
            (
                box_half_extents.x - local_sphere_pos.x.abs(),
                Vec3::X * local_sphere_pos.x.signum(),
            ),
            (
                box_half_extents.y - local_sphere_pos.y.abs(),
                Vec3::Y * local_sphere_pos.y.signum(),
            ),
            (
                box_half_extents.z - local_sphere_pos.z.abs(),
                Vec3::Z * local_sphere_pos.z.signum(),
            ),
        ];

        face_distances
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap()
            .1
    };

    // Transform back to world space
    let world_normal = box_rot * local_normal;
    let _world_closest = box_pos + box_rot * closest;
    let penetration = sphere_radius - distance;

    // Contact point should be on the sphere surface, not on the box
    let contact_point = sphere_pos - world_normal * sphere_radius;

    Some(Contact::new(
        sphere_entity,
        box_entity,
        contact_point,
        world_normal,
        penetration,
    ))
}

/// Test collision between two boxes using SAT (Separating Axis Theorem)
#[allow(clippy::too_many_arguments)]
fn box_box_collision(
    pos_a: Vec3,
    rot_a: Quat,
    extents_a: Vec3,
    entity_a: Entity,
    pos_b: Vec3,
    rot_b: Quat,
    extents_b: Vec3,
    entity_b: Entity,
) -> Option<Contact> {
    // Get rotation matrices
    let axes_a = [rot_a * Vec3::X, rot_a * Vec3::Y, rot_a * Vec3::Z];
    let axes_b = [rot_b * Vec3::X, rot_b * Vec3::Y, rot_b * Vec3::Z];

    let center_delta = pos_b - pos_a;

    let mut min_penetration = f32::MAX;
    let mut best_axis = Vec3::ZERO;
    let mut _best_axis_index = 0;
    let mut _is_axis_from_a = true;

    // Test face normals of box A
    for (i, &axis) in axes_a.iter().enumerate() {
        let (penetration, flip) =
            test_separation_axis(&axis, center_delta, extents_a, extents_b, &axes_a, &axes_b)?;

        if penetration < min_penetration {
            min_penetration = penetration;
            best_axis = if flip { -axis } else { axis };
            _best_axis_index = i;
            _is_axis_from_a = true;
        }
    }

    // Test face normals of box B
    for (i, &axis) in axes_b.iter().enumerate() {
        let (penetration, flip) =
            test_separation_axis(&axis, center_delta, extents_a, extents_b, &axes_a, &axes_b)?;

        if penetration < min_penetration {
            min_penetration = penetration;
            best_axis = if flip { -axis } else { axis };
            _best_axis_index = i;
            _is_axis_from_a = false;
        }
    }

    // Test edge-edge combinations
    for i in 0..3 {
        for j in 0..3 {
            let axis = axes_a[i].cross(axes_b[j]);
            if axis.length_squared() < 1e-6 {
                continue; // Parallel edges
            }

            let axis = axis.normalize();
            let (penetration, flip) =
                test_separation_axis(&axis, center_delta, extents_a, extents_b, &axes_a, &axes_b)?;

            if penetration < min_penetration {
                min_penetration = penetration;
                best_axis = if flip { -axis } else { axis };
                // Edge cases handled differently in contact generation
                _is_axis_from_a = false;
            }
        }
    }

    // Generate contact point - use the deepest penetrating point
    let support_a = get_box_support_point(pos_a, &axes_a, extents_a, -best_axis);
    let support_b = get_box_support_point(pos_b, &axes_b, extents_b, best_axis);

    // The contact point is the average of the two support points
    // This gives us a point that's in the overlap region
    let contact_point = (support_a + support_b) * 0.5;

    Some(Contact::new(
        entity_a,
        entity_b,
        contact_point,
        best_axis,
        min_penetration,
    ))
}

/// Test a separation axis for the SAT algorithm
fn test_separation_axis(
    axis: &Vec3,
    center_delta: Vec3,
    extents_a: Vec3,
    extents_b: Vec3,
    axes_a: &[Vec3; 3],
    axes_b: &[Vec3; 3],
) -> Option<(f32, bool)> {
    let separation = center_delta.dot(*axis);

    // Project box A onto axis
    let radius_a = extents_a.x * axes_a[0].dot(*axis).abs()
        + extents_a.y * axes_a[1].dot(*axis).abs()
        + extents_a.z * axes_a[2].dot(*axis).abs();

    // Project box B onto axis
    let radius_b = extents_b.x * axes_b[0].dot(*axis).abs()
        + extents_b.y * axes_b[1].dot(*axis).abs()
        + extents_b.z * axes_b[2].dot(*axis).abs();

    let penetration = radius_a + radius_b - separation.abs();

    if penetration < 0.0 {
        None // Separated along this axis
    } else {
        Some((penetration, separation < 0.0))
    }
}

/// Get the support point of a box in a given direction
fn get_box_support_point(center: Vec3, axes: &[Vec3; 3], extents: Vec3, direction: Vec3) -> Vec3 {
    let mut support = center;

    for (i, &axis) in axes.iter().enumerate() {
        let extent = match i {
            0 => extents.x,
            1 => extents.y,
            2 => extents.z,
            _ => unreachable!(),
        };

        if axis.dot(direction) > 0.0 {
            support += axis * extent;
        } else {
            support -= axis * extent;
        }
    }

    support
}

/// Compute the minimum translation vector (MTV) to separate two colliding shapes
pub fn compute_mtv(contact: &Contact) -> Vec3 {
    contact.normal * contact.penetration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_sphere_collision() {
        let mut world = hecs::World::new();
        let entity_a = world.spawn(());
        let entity_b = world.spawn(());

        let contact = sphere_sphere_collision(
            Vec3::ZERO,
            1.0,
            entity_a,
            Vec3::new(1.5, 0.0, 0.0),
            1.0,
            entity_b,
        );

        assert!(contact.is_some());
        let contact = contact.unwrap();
        assert!((contact.penetration - 0.5).abs() < 1e-6);
        assert!((contact.normal - Vec3::X).length() < 1e-6);
    }

    #[test]
    fn test_sphere_box_collision() {
        let mut world = hecs::World::new();
        let sphere_entity = world.spawn(());
        let box_entity = world.spawn(());

        let contact = sphere_box_collision(
            Vec3::new(1.5, 0.0, 0.0), // Changed from 2.0 to 1.5 to ensure penetration
            1.0,
            sphere_entity,
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
            box_entity,
        );

        assert!(contact.is_some());
        let contact = contact.unwrap();
        assert!(contact.penetration > 0.0);
        assert!((contact.normal - Vec3::X).length() < 1e-6);
    }

    #[test]
    fn test_no_collision() {
        let mut world = hecs::World::new();
        let entity_a = world.spawn(());
        let entity_b = world.spawn(());

        let contact = sphere_sphere_collision(
            Vec3::ZERO,
            1.0,
            entity_a,
            Vec3::new(10.0, 0.0, 0.0),
            1.0,
            entity_b,
        );

        assert!(contact.is_none());
    }
}
