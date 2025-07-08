//! GPU uniform buffer types
//!
//! Provides uniform buffer structures that match the WGSL shader definitions.
//! These structures are used to pass data from the CPU to the GPU.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;

/// Camera uniform buffer data
///
/// This struct matches the CameraUniform struct in the WGSL shader.
/// Contains the view-projection matrix for transforming vertices.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    /// Combined view-projection matrix
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Create a new camera uniform from a view-projection matrix
    pub fn new(view_proj: Mat4) -> Self {
        Self {
            view_proj: view_proj.to_cols_array_2d(),
        }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

/// Object uniform buffer data
///
/// This struct matches the ObjectUniform struct in the WGSL shader.
/// Contains per-object data like model matrix and material color.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ObjectUniform {
    /// Model matrix for world transformation
    pub model: [[f32; 4]; 4],
    /// Material color (RGBA)
    pub color: [f32; 4],
}

impl ObjectUniform {
    /// Create a new object uniform
    pub fn new(model: Mat4, color: [f32; 4]) -> Self {
        Self {
            model: model.to_cols_array_2d(),
            color,
        }
    }
}

impl Default for ObjectUniform {
    fn default() -> Self {
        Self {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Helper trait for creating GPU buffers from uniform types
pub trait UniformBuffer: Pod {
    /// Create a GPU buffer containing this uniform data
    fn create_buffer(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    /// Update an existing buffer with new data
    fn update_buffer(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[*self]));
    }
}

impl UniformBuffer for CameraUniform {}
impl UniformBuffer for ObjectUniform {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_uniform_size() {
        use std::mem;
        // Camera uniform should be 64 bytes (16 floats * 4 bytes)
        assert_eq!(mem::size_of::<CameraUniform>(), 64);
    }

    #[test]
    fn test_object_uniform_size() {
        use std::mem;
        // Object uniform should be 80 bytes (16 floats for matrix + 4 floats for color) * 4 bytes
        assert_eq!(mem::size_of::<ObjectUniform>(), 80);
    }

    #[test]
    fn test_camera_uniform_creation() {
        let view_proj = Mat4::perspective_rh(45.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let uniform = CameraUniform::new(view_proj);
        assert_eq!(uniform.view_proj, view_proj.to_cols_array_2d());
    }

    #[test]
    fn test_object_uniform_creation() {
        let model = Mat4::from_translation(glam::Vec3::new(1.0, 2.0, 3.0));
        let color = [1.0, 0.0, 0.0, 1.0];
        let uniform = ObjectUniform::new(model, color);
        assert_eq!(uniform.model, model.to_cols_array_2d());
        assert_eq!(uniform.color, color);
    }
}
