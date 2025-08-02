//! Audio-specific raycasting implementation
//!
//! Provides ray-AABB intersection tests for sound occlusion and room acoustics

use crate::core::entity::{Entity, Transform, World};
use crate::graphics::culling::AABB;
use glam::Vec3;

/// Ray structure for audio raycasting
#[derive(Debug, Clone, Copy)]
pub struct AudioRay {
    /// Ray origin in world space
    pub origin: Vec3,
    /// Ray direction (normalized)
    pub direction: Vec3,
}

/// Hit information from audio raycast
#[derive(Debug, Clone)]
pub struct AudioRayHit {
    /// Entity that was hit
    pub entity: Entity,
    /// Distance to the hit point
    pub distance: f32,
    /// Hit point in world space
    pub point: Vec3,
    /// Surface normal at hit point
    pub normal: Vec3,
}

/// Perform ray-AABB intersection test
///
/// Returns the distance to the intersection point if the ray hits the AABB
pub fn ray_aabb_intersection(ray: &AudioRay, aabb: &AABB, max_distance: f32) -> Option<f32> {
    // Handle division by zero for ray direction components
    let inv_dir = Vec3::new(
        if ray.direction.x.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / ray.direction.x
        },
        if ray.direction.y.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / ray.direction.y
        },
        if ray.direction.z.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            1.0 / ray.direction.z
        },
    );

    // Calculate intersection distances for each axis
    let t1 = (aabb.min - ray.origin) * inv_dir;
    let t2 = (aabb.max - ray.origin) * inv_dir;

    // Get min and max for each axis
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);

    // Find the largest minimum and smallest maximum
    let tmin = tmin.x.max(tmin.y).max(tmin.z).max(0.0);
    let tmax = tmax.x.min(tmax.y).min(tmax.z).min(max_distance);

    // Check if ray intersects the AABB
    if tmin <= tmax {
        Some(tmin)
    } else {
        None
    }
}

/// Transform AABB from local to world space
fn transform_aabb(aabb: &AABB, transform: &Transform) -> AABB {
    // Transform all 8 corners of the AABB
    let corners = [
        Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
        Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
        Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
        Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
        Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
        Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
        Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
        Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
    ];

    // Transform each corner
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for corner in &corners {
        let world_corner = transform.position + transform.rotation * (*corner * transform.scale);
        min = min.min(world_corner);
        max = max.max(world_corner);
    }

    AABB { min, max }
}

/// Perform audio raycast against all entities with AABBs
///
/// Returns the closest hit information if any entity is intersected
pub fn audio_raycast(
    world: &World,
    ray: AudioRay,
    max_distance: f32,
    exclude: Option<Entity>,
) -> Option<AudioRayHit> {
    let mut closest_hit: Option<(Entity, f32, AABB)> = None;

    // Query all entities with both AABB and Transform
    for (entity, (aabb, transform)) in world.query::<(&AABB, &Transform)>().iter() {
        // Skip excluded entity
        if Some(entity) == exclude {
            continue;
        }

        // Transform AABB to world space
        let world_aabb = transform_aabb(aabb, transform);

        // Test ray intersection
        if let Some(distance) = ray_aabb_intersection(&ray, &world_aabb, max_distance) {
            // Keep track of closest hit
            if closest_hit.as_ref().map_or(true, |(_, d, _)| distance < *d) {
                closest_hit = Some((entity, distance, world_aabb));
            }
        }
    }

    // Convert closest hit to AudioRayHit
    closest_hit.map(|(entity, distance, aabb)| {
        let hit_point = ray.origin + ray.direction * distance;

        // Calculate normal based on which face was hit
        let center = (aabb.min + aabb.max) * 0.5;
        let local_point = hit_point - center;
        let half_extents = (aabb.max - aabb.min) * 0.5;

        // Determine which face was hit by finding the axis with the largest relative position
        let relative = local_point / half_extents;
        let abs_relative = relative.abs();

        let normal = if abs_relative.x > abs_relative.y && abs_relative.x > abs_relative.z {
            Vec3::new(relative.x.signum(), 0.0, 0.0)
        } else if abs_relative.y > abs_relative.z {
            Vec3::new(0.0, relative.y.signum(), 0.0)
        } else {
            Vec3::new(0.0, 0.0, relative.z.signum())
        };

        AudioRayHit {
            entity,
            distance,
            point: hit_point,
            normal,
        }
    })
}

/// Perform multiple raycasts in a pattern for room acoustics estimation
pub fn audio_raycast_pattern(
    world: &World,
    origin: Vec3,
    directions: &[Vec3],
    max_distance: f32,
) -> Vec<Option<AudioRayHit>> {
    directions
        .iter()
        .map(|&direction| {
            let ray = AudioRay { origin, direction };
            audio_raycast(world, ray, max_distance, None)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_aabb_intersection() {
        let ray = AudioRay {
            origin: Vec3::new(0.0, 0.0, -5.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
        };

        let aabb = AABB {
            min: Vec3::new(-1.0, -1.0, -1.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        };

        let hit = ray_aabb_intersection(&ray, &aabb, 10.0);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_ray_aabb_miss() {
        let ray = AudioRay {
            origin: Vec3::new(5.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 1.0, 0.0),
        };

        let aabb = AABB {
            min: Vec3::new(-1.0, -1.0, -1.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        };

        let hit = ray_aabb_intersection(&ray, &aabb, 10.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_transform_aabb() {
        let aabb = AABB {
            min: Vec3::new(-1.0, -1.0, -1.0),
            max: Vec3::new(1.0, 1.0, 1.0),
        };

        let mut transform = Transform::default();
        transform.position = Vec3::new(5.0, 0.0, 0.0);
        transform.scale = Vec3::new(2.0, 2.0, 2.0);

        let world_aabb = transform_aabb(&aabb, &transform);
        assert!((world_aabb.min - Vec3::new(3.0, -2.0, -2.0)).length() < 0.001);
        assert!((world_aabb.max - Vec3::new(7.0, 2.0, 2.0)).length() < 0.001);
    }
}
