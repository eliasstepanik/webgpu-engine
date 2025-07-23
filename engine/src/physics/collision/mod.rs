//! Collision detection subsystem

pub mod broad_phase;
pub mod narrow_phase;
pub mod shapes;

use glam::Vec3;
use hecs::Entity;

/// Contact information between two colliding bodies
#[derive(Debug, Clone)]
pub struct Contact {
    /// First body entity
    pub entity_a: Entity,
    /// Second body entity
    pub entity_b: Entity,
    /// World space contact point
    pub position: Vec3,
    /// Contact normal pointing from A to B
    pub normal: Vec3,
    /// Penetration depth (negative for separation)
    pub penetration: f32,
    /// Tangent vector for friction (perpendicular to normal)
    pub tangent: Vec3,
    /// Bitangent vector for friction (perpendicular to normal and tangent)
    pub bitangent: Vec3,
}

impl Contact {
    /// Create a new contact
    pub fn new(
        entity_a: Entity,
        entity_b: Entity,
        position: Vec3,
        normal: Vec3,
        penetration: f32,
    ) -> Self {
        // Create orthonormal basis for friction
        let (tangent, bitangent) = create_tangent_basis(normal);

        Self {
            entity_a,
            entity_b,
            position,
            normal,
            penetration,
            tangent,
            bitangent,
        }
    }

    /// Flip the contact (swap A and B)
    pub fn flipped(self) -> Self {
        Self {
            entity_a: self.entity_b,
            entity_b: self.entity_a,
            position: self.position,
            normal: -self.normal,
            penetration: self.penetration,
            tangent: self.tangent,
            bitangent: self.bitangent,
        }
    }
}

/// Create an orthonormal basis given a normal vector
fn create_tangent_basis(normal: Vec3) -> (Vec3, Vec3) {
    // Choose a vector that's not parallel to the normal
    let up = if normal.y.abs() < 0.9 {
        Vec3::Y
    } else {
        Vec3::X
    };

    let tangent = up.cross(normal).normalize();
    let bitangent = normal.cross(tangent);

    (tangent, bitangent)
}

/// Axis-aligned bounding box for broad phase
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// Create a new AABB from min and max points
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a center point and half-extents
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Check if this AABB overlaps with another
    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Expand this AABB to include a point
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Get the center of the AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the half-extents of the AABB
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Merge two AABBs
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_tangent_basis() {
        let normal = Vec3::Y;
        let (tangent, bitangent) = create_tangent_basis(normal);

        // Check orthonormality
        assert!((normal.dot(tangent)).abs() < 1e-6);
        assert!((normal.dot(bitangent)).abs() < 1e-6);
        assert!((tangent.dot(bitangent)).abs() < 1e-6);

        // Check unit length
        assert!((tangent.length() - 1.0).abs() < 1e-6);
        assert!((bitangent.length() - 1.0).abs() < 1e-6);
    }
}
