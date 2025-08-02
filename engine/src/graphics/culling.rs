//! Frustum culling components and systems for efficient rendering

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::ComponentRegistry;
use glam::{Mat4, Vec3, Vec4};
use serde::{Deserialize, Serialize};

/// Axis-Aligned Bounding Box component
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AABB")]
pub struct AABB {
    #[ui(readonly, tooltip = "Minimum bounds")]
    pub min: Vec3,
    #[ui(readonly, tooltip = "Maximum bounds")]
    pub max: Vec3,
}

impl AABB {
    /// Create a new AABB from min and max points
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from a list of points
    pub fn from_points(points: &[Vec3]) -> Self {
        if points.is_empty() {
            return Self {
                min: Vec3::ZERO,
                max: Vec3::ZERO,
            };
        }

        let mut min = points[0];
        let mut max = points[0];

        for point in points.iter().skip(1) {
            min = min.min(*point);
            max = max.max(*point);
        }

        Self { min, max }
    }

    /// Get the center of the AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size/extents of the AABB
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Transform the AABB by a matrix
    pub fn transform(&self, transform: Mat4) -> Self {
        // Get all 8 corners of the AABB
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        // Transform all corners and find new bounds
        let transformed_corners: Vec<Vec3> = corners
            .iter()
            .map(|&corner| transform.transform_point3(corner))
            .collect();

        Self::from_points(&transformed_corners)
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }
}

/// Visibility component for culling state
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "Visibility")]
pub struct Visibility {
    #[ui(readonly, tooltip = "Is visible in current frame")]
    pub is_visible: bool,
    #[ui(readonly, tooltip = "Was visible in previous frame")]
    pub was_visible_last_frame: bool,
}

impl Visibility {
    /// Create a new visibility component, visible by default
    pub fn new() -> Self {
        Self {
            is_visible: true,
            was_visible_last_frame: true,
        }
    }

    /// Update visibility for the new frame
    pub fn update(&mut self, visible: bool) {
        self.was_visible_last_frame = self.is_visible;
        self.is_visible = visible;
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::new()
    }
}

/// View frustum with 6 planes
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    planes: [Vec4; 6], // left, right, bottom, top, near, far
}

impl Frustum {
    /// Extract frustum planes from a view-projection matrix
    pub fn from_matrix(view_proj: Mat4) -> Self {
        let m = view_proj.to_cols_array();

        // Extract planes from matrix rows
        // Each plane is represented as Ax + By + Cz + D = 0
        let left = Vec4::new(m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]);

        let right = Vec4::new(m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]);

        let bottom = Vec4::new(m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]);

        let top = Vec4::new(m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]);

        let near = Vec4::new(m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]);

        let far = Vec4::new(m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]);

        // Normalize planes
        let planes = [
            Self::normalize_plane(left),
            Self::normalize_plane(right),
            Self::normalize_plane(bottom),
            Self::normalize_plane(top),
            Self::normalize_plane(near),
            Self::normalize_plane(far),
        ];

        Self { planes }
    }

    /// Normalize a plane equation
    fn normalize_plane(plane: Vec4) -> Vec4 {
        let normal_length = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
        if normal_length > 0.0 {
            plane / normal_length
        } else {
            plane
        }
    }

    /// Test if an AABB is visible within the frustum
    pub fn is_aabb_visible(&self, aabb: &AABB) -> bool {
        // Test against all 6 planes
        for plane in &self.planes {
            if Self::is_aabb_outside_plane(aabb, *plane) {
                return false;
            }
        }
        true
    }

    /// Check if AABB is completely on the negative side of a plane
    fn is_aabb_outside_plane(aabb: &AABB, plane: Vec4) -> bool {
        // Find the AABB vertex that is furthest along the plane normal
        let p = Vec3::new(
            if plane.x > 0.0 {
                aabb.max.x
            } else {
                aabb.min.x
            },
            if plane.y > 0.0 {
                aabb.max.y
            } else {
                aabb.min.y
            },
            if plane.z > 0.0 {
                aabb.max.z
            } else {
                aabb.min.z
            },
        );

        // If this vertex is on the negative side, the entire AABB is outside
        plane.dot(p.extend(1.0)) < 0.0
    }

    /// Get the frustum planes for debug visualization
    pub fn planes(&self) -> &[Vec4; 6] {
        &self.planes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
        assert_eq!(aabb.size(), Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_aabb_from_points() {
        let points = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];

        let aabb = AABB::from_points(&points);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_visibility_update() {
        let mut visibility = Visibility::new();
        assert!(visibility.is_visible);
        assert!(visibility.was_visible_last_frame);

        visibility.update(false);
        assert!(!visibility.is_visible);
        assert!(visibility.was_visible_last_frame);

        visibility.update(true);
        assert!(visibility.is_visible);
        assert!(!visibility.was_visible_last_frame);
    }

    #[test]
    fn test_frustum_aabb_intersection() {
        // Create a proper perspective projection matrix
        let aspect_ratio = 16.0 / 9.0;
        let fov_y_radians = std::f32::consts::PI / 4.0; // 45 degrees
        let near = 0.1;
        let far = 100.0;

        let proj = Mat4::perspective_rh(fov_y_radians, aspect_ratio, near, far);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0), // eye
            Vec3::ZERO,               // center
            Vec3::Y,                  // up
        );
        let view_proj = proj * view;

        let frustum = Frustum::from_matrix(view_proj);

        // Test AABB at origin (should be visible)
        let aabb_center = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(frustum.is_aabb_visible(&aabb_center));

        // Test AABB far behind camera (should not be visible)
        let aabb_behind = AABB::new(Vec3::new(-1.0, -1.0, 10.0), Vec3::new(1.0, 1.0, 12.0));
        assert!(!frustum.is_aabb_visible(&aabb_behind));

        // Test AABB far to the right (should not be visible)
        let aabb_right = AABB::new(Vec3::new(50.0, -1.0, -1.0), Vec3::new(52.0, 1.0, 1.0));
        assert!(!frustum.is_aabb_visible(&aabb_right));

        // Test AABB far beyond far plane (should not be visible)
        let aabb_far = AABB::new(Vec3::new(-1.0, -1.0, -200.0), Vec3::new(1.0, 1.0, -198.0));
        assert!(!frustum.is_aabb_visible(&aabb_far));
    }

    #[test]
    fn test_frustum_plane_extraction() {
        // Create a simple orthographic projection for easier verification
        let left = -10.0;
        let right = 10.0;
        let bottom = -10.0;
        let top = 10.0;
        let near = 0.1;
        let far = 100.0;

        let proj = Mat4::orthographic_rh(left, right, bottom, top, near, far);
        let frustum = Frustum::from_matrix(proj);

        // Test points that should be inside/outside
        let inside = AABB::new(Vec3::new(0.0, 0.0, -1.0), Vec3::new(1.0, 1.0, -0.5));
        assert!(frustum.is_aabb_visible(&inside));

        // Test AABB straddling the left plane
        let straddle_left = AABB::new(Vec3::new(-11.0, 0.0, -1.0), Vec3::new(-9.0, 1.0, -0.5));
        assert!(frustum.is_aabb_visible(&straddle_left));

        // Test AABB completely outside left plane
        let outside_left = AABB::new(Vec3::new(-15.0, 0.0, -1.0), Vec3::new(-11.0, 1.0, -0.5));
        assert!(!frustum.is_aabb_visible(&outside_left));
    }

    #[test]
    fn test_aabb_transform() {
        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

        // Test translation
        let translation = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let translated = aabb.transform(translation);
        assert_eq!(translated.min, Vec3::new(4.0, -1.0, -1.0));
        assert_eq!(translated.max, Vec3::new(6.0, 1.0, 1.0));

        // Test scaling
        let scale = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
        let scaled = aabb.transform(scale);
        assert_eq!(scaled.min, Vec3::new(-2.0, -2.0, -2.0));
        assert_eq!(scaled.max, Vec3::new(2.0, 2.0, 2.0));

        // Test rotation (90 degrees around Y)
        let rotation = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let rotated = aabb.transform(rotation);
        // After 90 degree rotation around Y, X and Z swap
        assert!((rotated.min.x - (-1.0)).abs() < 0.001);
        assert!((rotated.min.z - (-1.0)).abs() < 0.001);
        assert!((rotated.max.x - 1.0).abs() < 0.001);
        assert!((rotated.max.z - 1.0).abs() < 0.001);
    }
}
