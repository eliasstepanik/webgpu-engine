//! Camera component and utilities
//!
//! Provides camera functionality for 3D rendering, including perspective and
//! orthographic projections, and view matrix calculation from transforms.

use crate::core::entity::{components::GlobalWorldTransform, GlobalTransform};
use glam::{DVec3, Mat4};
use serde::{Deserialize, Serialize};

/// Camera component that defines projection parameters for rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
    /// Field of view in radians (for perspective projection)
    pub fov_y_radians: f32,
    /// Aspect ratio (width / height)
    pub aspect_ratio: f32,
    /// Near clipping plane distance
    pub z_near: f32,
    /// Far clipping plane distance
    pub z_far: f32,
    /// Projection mode (perspective or orthographic)
    pub projection_mode: ProjectionMode,
    /// Use logarithmic depth buffer for better precision at large distances
    pub use_logarithmic_depth: bool,
}

/// Projection mode for the camera
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProjectionMode {
    /// Perspective projection with depth
    Perspective,
    /// Orthographic projection (parallel)
    Orthographic {
        /// Height of the orthographic view
        height: f32,
    },
}

impl Default for Camera {
    fn default() -> Self {
        Self::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0)
    }
}

impl Camera {
    /// Create a perspective camera
    ///
    /// # Arguments
    /// * `fov_y_degrees` - Vertical field of view in degrees
    /// * `aspect_ratio` - Width divided by height
    /// * `z_near` - Near clipping plane distance
    /// * `z_far` - Far clipping plane distance
    pub fn perspective(fov_y_degrees: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            fov_y_radians: fov_y_degrees.to_radians(),
            aspect_ratio,
            z_near,
            z_far,
            projection_mode: ProjectionMode::Perspective,
            use_logarithmic_depth: false,
        }
    }

    /// Create a perspective camera with logarithmic depth buffer
    ///
    /// This is recommended for large world scenarios to improve depth precision
    pub fn perspective_logarithmic(
        fov_y_degrees: f32,
        aspect_ratio: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            fov_y_radians: fov_y_degrees.to_radians(),
            aspect_ratio,
            z_near,
            z_far,
            projection_mode: ProjectionMode::Perspective,
            use_logarithmic_depth: true,
        }
    }

    /// Create an orthographic camera
    ///
    /// # Arguments
    /// * `height` - Height of the orthographic view
    /// * `aspect_ratio` - Width divided by height
    /// * `z_near` - Near clipping plane distance
    /// * `z_far` - Far clipping plane distance
    pub fn orthographic(height: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            fov_y_radians: 0.0, // Not used for orthographic
            aspect_ratio,
            z_near,
            z_far,
            projection_mode: ProjectionMode::Orthographic { height },
            use_logarithmic_depth: false,
        }
    }

    /// Calculate the projection matrix for this camera
    pub fn projection_matrix(&self) -> Mat4 {
        match self.projection_mode {
            ProjectionMode::Perspective => Mat4::perspective_rh(
                self.fov_y_radians,
                self.aspect_ratio,
                self.z_near,
                self.z_far,
            ),
            ProjectionMode::Orthographic { height } => {
                let half_height = height * 0.5;
                let half_width = half_height * self.aspect_ratio;
                Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    self.z_near,
                    self.z_far,
                )
            }
        }
    }

    /// Calculate the view matrix from a camera's global transform
    ///
    /// The view matrix is the inverse of the camera's world transform
    pub fn view_matrix(camera_transform: &GlobalTransform) -> Mat4 {
        camera_transform.matrix.inverse()
    }

    /// Calculate the combined view-projection matrix
    ///
    /// This is commonly used in shaders for transforming vertices
    pub fn view_projection_matrix(&self, camera_transform: &GlobalTransform) -> Mat4 {
        self.projection_matrix() * Self::view_matrix(camera_transform)
    }

    /// Update the aspect ratio (useful when window resizes)
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    /// Calculate the view matrix from a high-precision world transform
    pub fn view_matrix_world(camera_transform: &GlobalWorldTransform) -> Mat4 {
        // For large world scenarios, we use the camera-relative transform
        // The actual camera position in world coordinates doesn't matter for rendering
        camera_transform.matrix.inverse().as_mat4()
    }

    /// Calculate view-projection matrix for large world rendering
    ///
    /// This version works with high-precision world transforms and converts
    /// to camera-relative coordinates for GPU rendering
    pub fn view_projection_matrix_world(
        &self,
        camera_transform: &GlobalWorldTransform,
        camera_world_pos: DVec3,
    ) -> Mat4 {
        // Create a camera-relative transform for rendering
        let camera_relative = camera_transform.to_camera_relative(camera_world_pos);
        self.projection_matrix() * Self::view_matrix(&camera_relative)
    }

    /// Enable or disable logarithmic depth buffer
    pub fn set_logarithmic_depth(&mut self, enabled: bool) {
        self.use_logarithmic_depth = enabled;
    }

    /// Get the logarithmic depth coefficient for shaders
    ///
    /// This value is used in logarithmic depth shaders to transform depth values
    pub fn logarithmic_depth_coefficient(&self) -> f32 {
        if self.use_logarithmic_depth {
            2.0 / (self.z_far / self.z_near + 1.0).ln()
        } else {
            0.0 // Not used in linear depth
        }
    }
}

/// Camera world position component for large world coordinate tracking
///
/// This component tracks the camera's position in high-precision world coordinates.
/// It's separate from the regular transform to maintain backward compatibility.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CameraWorldPosition {
    /// Camera position in world coordinates using 64-bit precision
    pub position: DVec3,
}

impl Default for CameraWorldPosition {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
        }
    }
}

impl CameraWorldPosition {
    /// Create a new camera world position
    pub fn new(position: DVec3) -> Self {
        Self { position }
    }

    /// Update the world position
    pub fn set_position(&mut self, position: DVec3) {
        self.position = position;
    }

    /// Translate the camera by the given offset
    pub fn translate(&mut self, offset: DVec3) {
        self.position += offset;
    }

    /// Get the distance to a world position
    pub fn distance_to(&self, world_pos: DVec3) -> f64 {
        self.position.distance(world_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_camera_perspective_projection() {
        let camera = Camera::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0);
        let proj = camera.projection_matrix();

        // Perspective projection has w=0 in the last row
        assert_eq!(proj.w_axis.w, 0.0);

        // Near/far planes should affect the matrix
        assert!(proj.z_axis.z < 0.0);
    }

    #[test]
    fn test_camera_orthographic_projection() {
        let camera = Camera::orthographic(10.0, 16.0 / 9.0, 0.1, 1000.0);
        let proj = camera.projection_matrix();

        // Orthographic projection has w=1 in the last row
        assert_eq!(proj.w_axis.w, 1.0);
    }

    #[test]
    fn test_view_matrix() {
        use crate::core::entity::Transform;

        // Camera at (0, 0, 5) looking at origin
        let transform = Transform::from_position(Vec3::new(0.0, 0.0, 5.0));
        let global = GlobalTransform::from_matrix(transform.to_matrix());

        let view = Camera::view_matrix(&global);

        // View matrix should translate in opposite direction
        assert_eq!(view.w_axis.z, -5.0);
    }

    #[test]
    fn test_camera_default() {
        let camera = Camera::default();
        assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
        assert_eq!(camera.aspect_ratio, 16.0 / 9.0);
    }

    #[test]
    fn test_set_aspect_ratio() {
        let mut camera = Camera::default();
        camera.set_aspect_ratio(4.0 / 3.0);
        assert_eq!(camera.aspect_ratio, 4.0 / 3.0);
    }
}
