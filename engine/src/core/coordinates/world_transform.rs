//! WorldTransform component for large world coordinates
//!
//! Provides a high-precision transform component using 64-bit floating point
//! for position coordinates, enabling stable positioning at planetary scales.

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::core::entity::components::Transform;
use crate::io::component_registry::ComponentRegistry;
use glam::{DMat4, DVec3, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// World-space transform component with 64-bit precision position
///
/// Use this component for entities that need to exist at large distances
/// from the world origin (>1 million units) without precision loss.
///
/// For normal gameplay objects, the regular Transform component is sufficient.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, engine_derive::Component, engine_derive::EditorUI)]
#[component(name = "WorldTransform")]
pub struct WorldTransform {
    /// Position in world space using 64-bit precision
    pub position: DVec3,
    /// Rotation quaternion (32-bit is sufficient for rotations)
    pub rotation: Quat,
    /// Scale vector (32-bit is sufficient for scale)
    pub scale: Vec3,
}

impl Default for WorldTransform {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl WorldTransform {
    /// Create a new WorldTransform with the given position
    pub fn from_position(position: DVec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a new WorldTransform with position and rotation
    pub fn from_position_rotation(position: DVec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Default::default()
        }
    }

    /// Create a WorldTransform from a regular Transform with an offset
    ///
    /// This is useful for converting existing entities to use world coordinates
    pub fn from_transform(transform: &Transform, world_offset: DVec3) -> Self {
        Self {
            position: DVec3::new(
                transform.position.x as f64,
                transform.position.y as f64,
                transform.position.z as f64,
            ) + world_offset,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }

    /// Convert to a camera-relative Transform for rendering
    ///
    /// This is the core of the large world system - convert high-precision
    /// world coordinates to camera-relative coordinates for GPU rendering
    pub fn to_camera_relative(&self, camera_world_position: DVec3) -> Transform {
        let relative_position = self.position - camera_world_position;

        // Convert to f32 - this is safe because we're now camera-relative
        Transform {
            position: Vec3::new(
                relative_position.x as f32,
                relative_position.y as f32,
                relative_position.z as f32,
            ),
            rotation: self.rotation,
            scale: self.scale,
        }
    }

    /// Convert to a 4x4 transformation matrix using 64-bit precision
    pub fn to_matrix(&self) -> DMat4 {
        DMat4::from_scale_rotation_translation(
            self.scale.as_dvec3(),
            self.rotation.as_dquat(),
            self.position,
        )
    }

    /// Get the distance to another WorldTransform
    pub fn distance_to(&self, other: &WorldTransform) -> f64 {
        self.position.distance(other.position)
    }

    /// Check if this transform is within rendering distance of a camera
    pub fn is_within_render_distance(&self, camera_position: DVec3, max_distance: f64) -> bool {
        self.position.distance(camera_position) <= max_distance
    }

    /// Set the world position
    pub fn set_position(&mut self, position: DVec3) {
        self.position = position;
    }

    /// Translate by the given offset
    pub fn translate(&mut self, offset: DVec3) {
        self.position += offset;
    }

    /// Set the transform to look at a target position with the given up vector
    pub fn look_at(&mut self, target: DVec3, up: DVec3) {
        let forward = (target - self.position).normalize();
        let right = forward.cross(up).normalize();
        let actual_up = right.cross(forward);

        // Create rotation matrix from basis vectors (using f32 for the rotation)
        let rotation_matrix = glam::Mat3::from_cols(
            right.as_vec3(),
            actual_up.as_vec3(),
            -forward.as_vec3(), // Forward is negative Z in right-handed system
        );

        self.rotation = Quat::from_mat3(&rotation_matrix);
    }

    /// Convert to hierarchical galaxy position
    pub fn to_galaxy_position(&self, sector_size: f64) -> crate::core::coordinates::GalaxyPosition {
        crate::core::coordinates::GalaxyPosition::from_world_position(self.position, sector_size)
    }

    /// Create from hierarchical galaxy position
    pub fn from_galaxy_position(galaxy_pos: &crate::core::coordinates::GalaxyPosition) -> Self {
        Self::from_position(galaxy_pos.to_world_position())
    }

    /// Check if position is at galaxy scale (>10^15 meters from origin)
    pub fn is_galaxy_scale(&self) -> bool {
        self.position.length() > 1e15
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_transform_default() {
        let transform = WorldTransform::default();
        assert_eq!(transform.position, DVec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_precision_at_large_distances() {
        // Test precision at 100 million units from origin
        let world_pos = DVec3::new(100_000_000.0, 0.0, 0.0);
        let camera_pos = DVec3::new(99_999_999.0, 0.0, 0.0);

        let world_transform = WorldTransform {
            position: world_pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let relative = world_transform.to_camera_relative(camera_pos);

        // Should be exactly 1.0 unit away from camera
        assert!((relative.position.x - 1.0).abs() < 0.001);
        assert!(relative.position.y.abs() < f64::EPSILON as f32);
        assert!(relative.position.z.abs() < f64::EPSILON as f32);
    }

    #[test]
    fn test_from_transform_conversion() {
        let original = Transform {
            position: Vec3::new(10.0, 20.0, 30.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI / 4.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };

        let offset = DVec3::new(1000.0, 0.0, 0.0);
        let world_transform = WorldTransform::from_transform(&original, offset);

        assert_eq!(world_transform.position, DVec3::new(1010.0, 20.0, 30.0));
        assert_eq!(world_transform.rotation, original.rotation);
        assert_eq!(world_transform.scale, original.scale);
    }

    #[test]
    fn test_distance_calculation() {
        let transform1 = WorldTransform::from_position(DVec3::new(0.0, 0.0, 0.0));
        let transform2 = WorldTransform::from_position(DVec3::new(3.0, 4.0, 0.0));

        assert!((transform1.distance_to(&transform2) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_render_distance_check() {
        let transform = WorldTransform::from_position(DVec3::new(1000.0, 0.0, 0.0));
        let camera_pos = DVec3::ZERO;

        assert!(transform.is_within_render_distance(camera_pos, 1500.0));
        assert!(!transform.is_within_render_distance(camera_pos, 500.0));
    }

    #[test]
    fn test_look_at() {
        let mut transform = WorldTransform::from_position(DVec3::new(0.0, 0.0, 0.0));
        let target = DVec3::new(1.0, 0.0, 0.0);
        let up = DVec3::new(0.0, 1.0, 0.0);

        transform.look_at(target, up);

        // Should rotate to face the target
        // In a right-handed system looking towards +X should result in specific rotation
        let forward = transform.rotation * Vec3::NEG_Z; // Forward is -Z
        assert!((forward.x - 1.0).abs() < 0.001);
        assert!(forward.y.abs() < 0.001);
        assert!(forward.z.abs() < 0.001);
    }
}
