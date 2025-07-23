//! Collision shape utilities and transformations

use crate::physics::components::CollisionShape;
use glam::{Quat, Vec3};

use super::AABB;

impl CollisionShape {
    /// Get the AABB for this shape in local space
    pub fn local_aabb(&self) -> AABB {
        match self {
            CollisionShape::Sphere { radius } => {
                AABB::from_center_half_extents(Vec3::ZERO, Vec3::splat(*radius))
            }
            CollisionShape::Box { half_extents } => {
                AABB::from_center_half_extents(Vec3::ZERO, *half_extents)
            }
            CollisionShape::Capsule {
                radius,
                half_height,
            } => AABB::from_center_half_extents(
                Vec3::ZERO,
                Vec3::new(*radius, half_height + radius, *radius),
            ),
        }
    }

    /// Get the AABB for this shape transformed by position and rotation
    pub fn world_aabb(&self, position: Vec3, rotation: Quat) -> AABB {
        match self {
            CollisionShape::Sphere { radius } => {
                // Spheres are rotation-invariant
                AABB::from_center_half_extents(position, Vec3::splat(*radius))
            }
            CollisionShape::Box { half_extents } => {
                // Transform the 8 corners of the box
                let corners = [
                    Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
                    Vec3::new(half_extents.x, -half_extents.y, -half_extents.z),
                    Vec3::new(-half_extents.x, half_extents.y, -half_extents.z),
                    Vec3::new(half_extents.x, half_extents.y, -half_extents.z),
                    Vec3::new(-half_extents.x, -half_extents.y, half_extents.z),
                    Vec3::new(half_extents.x, -half_extents.y, half_extents.z),
                    Vec3::new(-half_extents.x, half_extents.y, half_extents.z),
                    Vec3::new(half_extents.x, half_extents.y, half_extents.z),
                ];

                let mut aabb = AABB::new(Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
                for corner in &corners {
                    let world_corner = position + rotation * corner;
                    aabb.expand_to_include(world_corner);
                }
                aabb
            }
            CollisionShape::Capsule {
                radius,
                half_height,
            } => {
                // Transform the capsule endpoints
                let top = position + rotation * Vec3::new(0.0, *half_height, 0.0);
                let bottom = position + rotation * Vec3::new(0.0, -*half_height, 0.0);

                // Create AABBs for the spheres at each end
                let top_aabb = AABB::from_center_half_extents(top, Vec3::splat(*radius));
                let bottom_aabb = AABB::from_center_half_extents(bottom, Vec3::splat(*radius));

                top_aabb.merge(&bottom_aabb)
            }
        }
    }

    /// Get support point in given direction (for GJK algorithm)
    pub fn support(&self, direction: Vec3) -> Vec3 {
        match self {
            CollisionShape::Sphere { radius } => direction.normalize() * radius,
            CollisionShape::Box { half_extents } => Vec3::new(
                if direction.x > 0.0 {
                    half_extents.x
                } else {
                    -half_extents.x
                },
                if direction.y > 0.0 {
                    half_extents.y
                } else {
                    -half_extents.y
                },
                if direction.z > 0.0 {
                    half_extents.z
                } else {
                    -half_extents.z
                },
            ),
            CollisionShape::Capsule {
                radius,
                half_height,
            } => {
                // Find which hemisphere the direction points to
                let y_component = direction.y;
                let hemisphere_center = if y_component > 0.0 {
                    Vec3::new(0.0, *half_height, 0.0)
                } else {
                    Vec3::new(0.0, -*half_height, 0.0)
                };

                // Support point is the hemisphere center plus radius in the direction
                hemisphere_center + direction.normalize() * radius
            }
        }
    }

    /// Transform support point to world space
    pub fn world_support(&self, direction: Vec3, position: Vec3, rotation: Quat) -> Vec3 {
        // Transform direction to local space
        let local_direction = rotation.conjugate() * direction;
        // Get local support point
        let local_support = self.support(local_direction);
        // Transform back to world space
        position + rotation * local_support
    }

    /// Get the volume of the shape
    pub fn volume(&self) -> f32 {
        match self {
            CollisionShape::Sphere { radius } => {
                // V = 4/3 * π * r³
                (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3)
            }
            CollisionShape::Box { half_extents } => {
                // V = 2x * 2y * 2z
                8.0 * half_extents.x * half_extents.y * half_extents.z
            }
            CollisionShape::Capsule {
                radius,
                half_height,
            } => {
                // V = π * r² * h + 4/3 * π * r³
                let cylinder_volume = std::f32::consts::PI * radius.powi(2) * (2.0 * half_height);
                let sphere_volume = (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3);
                cylinder_volume + sphere_volume
            }
        }
    }

    /// Check if a point is inside the shape (in local space)
    pub fn contains_point(&self, point: Vec3) -> bool {
        match self {
            CollisionShape::Sphere { radius } => point.length() <= *radius,
            CollisionShape::Box { half_extents } => {
                point.x.abs() <= half_extents.x
                    && point.y.abs() <= half_extents.y
                    && point.z.abs() <= half_extents.z
            }
            CollisionShape::Capsule {
                radius,
                half_height,
            } => {
                // Clamp point to the capsule line segment
                let clamped_y = point.y.clamp(-*half_height, *half_height);
                let closest_point = Vec3::new(0.0, clamped_y, 0.0);
                (point - closest_point).length() <= *radius
            }
        }
    }

    /// Get the closest point on the shape surface to a given point (in local space)
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        match self {
            CollisionShape::Sphere { radius } => {
                if point.length() > 0.0 {
                    point.normalize() * radius
                } else {
                    Vec3::X * radius // Arbitrary point on sphere if at center
                }
            }
            CollisionShape::Box { half_extents } => {
                // Clamp point to box bounds
                Vec3::new(
                    point.x.clamp(-half_extents.x, half_extents.x),
                    point.y.clamp(-half_extents.y, half_extents.y),
                    point.z.clamp(-half_extents.z, half_extents.z),
                )
            }
            CollisionShape::Capsule {
                radius,
                half_height,
            } => {
                // Find closest point on line segment
                let clamped_y = point.y.clamp(-*half_height, *half_height);
                let closest_on_line = Vec3::new(0.0, clamped_y, 0.0);

                // Find closest point on sphere centered at closest_on_line
                let to_point = point - closest_on_line;
                if to_point.length() > 0.0 {
                    closest_on_line + to_point.normalize() * radius
                } else {
                    closest_on_line + Vec3::X * radius
                }
            }
        }
    }
}

/// Ray for raycasting
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at distance t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Raycast result
pub struct RaycastHit {
    /// Distance along the ray to the hit point
    pub distance: f32,
    /// World space hit point
    pub position: Vec3,
    /// Surface normal at the hit point
    pub normal: Vec3,
}

impl CollisionShape {
    /// Perform a raycast against this shape (in local space)
    pub fn raycast(&self, ray: &Ray, max_distance: f32) -> Option<RaycastHit> {
        match self {
            CollisionShape::Sphere { radius } => {
                // Ray-sphere intersection
                let oc = ray.origin;
                let a = ray.direction.dot(ray.direction);
                let b = 2.0 * oc.dot(ray.direction);
                let c = oc.dot(oc) - radius * radius;
                let discriminant = b * b - 4.0 * a * c;

                if discriminant < 0.0 {
                    return None;
                }

                let sqrt_discriminant = discriminant.sqrt();
                let t1 = (-b - sqrt_discriminant) / (2.0 * a);
                let t2 = (-b + sqrt_discriminant) / (2.0 * a);

                let t = if t1 > 0.0 && t1 < max_distance {
                    t1
                } else if t2 > 0.0 && t2 < max_distance {
                    t2
                } else {
                    return None;
                };

                let position = ray.at(t);
                let normal = position.normalize();

                Some(RaycastHit {
                    distance: t,
                    position,
                    normal,
                })
            }
            CollisionShape::Box { half_extents } => {
                // Ray-box intersection using slab method
                let inv_dir = Vec3::new(
                    1.0 / ray.direction.x,
                    1.0 / ray.direction.y,
                    1.0 / ray.direction.z,
                );

                let t1 = (-half_extents - ray.origin) * inv_dir;
                let t2 = (*half_extents - ray.origin) * inv_dir;

                let t_min = t1.min(t2);
                let t_max = t1.max(t2);

                let t_enter = t_min.x.max(t_min.y).max(t_min.z).max(0.0);
                let t_exit = t_max.x.min(t_max.y).min(t_max.z);

                if t_enter > t_exit || t_enter > max_distance {
                    return None;
                }

                let position = ray.at(t_enter);

                // Determine which face was hit
                let eps = 1e-6;
                let normal = if (position.x - half_extents.x).abs() < eps {
                    Vec3::X
                } else if (position.x + half_extents.x).abs() < eps {
                    -Vec3::X
                } else if (position.y - half_extents.y).abs() < eps {
                    Vec3::Y
                } else if (position.y + half_extents.y).abs() < eps {
                    -Vec3::Y
                } else if (position.z - half_extents.z).abs() < eps {
                    Vec3::Z
                } else {
                    -Vec3::Z
                };

                Some(RaycastHit {
                    distance: t_enter,
                    position,
                    normal,
                })
            }
            CollisionShape::Capsule { .. } => {
                // TODO: Implement ray-capsule intersection
                // For now, return None
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_volume() {
        let sphere = CollisionShape::Sphere { radius: 1.0 };
        let expected = (4.0 / 3.0) * std::f32::consts::PI;
        assert!((sphere.volume() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_box_contains_point() {
        let box_shape = CollisionShape::Box {
            half_extents: Vec3::new(1.0, 2.0, 3.0),
        };

        assert!(box_shape.contains_point(Vec3::ZERO));
        assert!(box_shape.contains_point(Vec3::new(0.5, 1.5, 2.5)));
        assert!(!box_shape.contains_point(Vec3::new(1.5, 0.0, 0.0)));
        assert!(!box_shape.contains_point(Vec3::new(0.0, 2.5, 0.0)));
    }

    #[test]
    fn test_sphere_raycast() {
        let sphere = CollisionShape::Sphere { radius: 1.0 };
        let ray = Ray::new(Vec3::new(-2.0, 0.0, 0.0), Vec3::X);

        let hit = sphere.raycast(&ray, 10.0);
        assert!(hit.is_some());

        let hit = hit.unwrap();
        assert!((hit.distance - 1.0).abs() < 1e-6);
        assert!((hit.position - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-6);
        assert!((hit.normal - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-6);
    }
}
