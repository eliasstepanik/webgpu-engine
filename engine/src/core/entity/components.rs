//! Core components for the entity system

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Transform component representing position, rotation, and scale in local space
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    /// Position in local space
    pub position: Vec3,
    /// Rotation in local space as a quaternion
    pub rotation: Quat,
    /// Scale in local space
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Create a new transform with the given position
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Create a new transform with the given position and rotation
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Default::default()
        }
    }

    /// Convert this transform to a transformation matrix
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Set the transform to look at a target position with the given up vector
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        let forward = (target - self.position).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);

        let rotation_matrix = Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            (-forward).extend(0.0),
            Vec3::ZERO.extend(1.0),
        );

        self.rotation = Quat::from_mat4(&rotation_matrix);
        self
    }

    /// Set the scale of the transform
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }
}

/// Global transform component representing the world-space transformation matrix
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlobalTransform {
    /// World-space transformation matrix
    pub matrix: Mat4,
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }
}

impl GlobalTransform {
    /// Create a new global transform from a matrix
    pub fn from_matrix(matrix: Mat4) -> Self {
        Self { matrix }
    }

    /// Get the world position from the transformation matrix
    pub fn position(&self) -> Vec3 {
        self.matrix.w_axis.truncate()
    }
}

/// Parent component establishing a parent-child relationship
///
/// Note: hecs::Entity doesn't implement Serialize/Deserialize,
/// so we need custom serialization for scene loading/saving
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub hecs::Entity);

/// Serializable data for Parent component
///
/// This is used for scene serialization since hecs::Entity cannot be serialized directly.
/// The entity_id is remapped during scene loading to match the new entity IDs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParentData {
    /// Entity ID that will be remapped during scene loading
    pub entity_id: u64,
}

impl From<(Parent, u64)> for ParentData {
    fn from((_parent, id): (Parent, u64)) -> Self {
        Self { entity_id: id }
    }
}

impl ParentData {
    /// Create ParentData from a Parent component and entity ID mapping
    pub fn from_parent_with_id(_parent: Parent, entity_id: u64) -> Self {
        Self { entity_id }
    }

    /// Try to convert ParentData back to Parent using an entity mapper
    pub fn try_to_parent<F>(&self, entity_mapper: F) -> Option<Parent>
    where
        F: Fn(u64) -> Option<hecs::Entity>,
    {
        entity_mapper(self.entity_id).map(Parent)
    }
}

/// Name component for user-friendly entity identification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    /// Create a new name component
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_to_matrix() {
        let transform = Transform {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let matrix = transform.to_matrix();
        assert_eq!(matrix.w_axis.truncate(), transform.position);
    }

    #[test]
    fn test_global_transform_position() {
        let transform = Transform::from_position(Vec3::new(5.0, 10.0, 15.0));
        let global = GlobalTransform::from_matrix(transform.to_matrix());
        assert_eq!(global.position(), Vec3::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn test_name_component() {
        let name = Name::new("Test Entity");
        assert_eq!(name.0, "Test Entity");

        // Test default
        let default_name = Name::default();
        assert_eq!(default_name.0, "");

        // Test serialization
        let json = serde_json::to_string(&name).unwrap();
        let deserialized: Name = serde_json::from_str(&json).unwrap();
        assert_eq!(name.0, deserialized.0);
    }
}
