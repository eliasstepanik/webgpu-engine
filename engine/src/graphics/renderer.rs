//! Main renderer implementation
//!
//! The Renderer struct orchestrates all rendering operations, managing
//! render pipelines, GPU resources, and the rendering of entities.

use crate::core::camera::Camera;
use crate::core::entity::{GlobalTransform, World};
use crate::graphics::{
    context::RenderContext,
    material::Material,
    mesh::Mesh,
    pipeline::{DepthTexture, RenderPipeline},
    uniform::{CameraUniform, ObjectUniform, UniformBuffer},
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use wgpu::util::DeviceExt;

/// GPU resources for a mesh
struct MeshGpuData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

/// Main renderer that manages all rendering operations
pub struct Renderer<'window> {
    /// Render context with device and queue
    context: Arc<RenderContext<'window>>,
    /// Basic 3D render pipeline
    basic_pipeline: RenderPipeline,
    /// Depth texture for depth testing
    depth_texture: DepthTexture,
    /// Camera uniform buffer
    camera_uniform_buffer: wgpu::Buffer,
    /// Camera bind group
    camera_bind_group: wgpu::BindGroup,
    /// Cached mesh GPU data
    mesh_cache: HashMap<u64, MeshGpuData>,
    /// Next mesh ID for cache
    next_mesh_id: u64,
}

impl<'window> Renderer<'window> {
    /// Create a new renderer
    pub fn new(context: Arc<RenderContext<'window>>) -> Self {
        info!("Initializing renderer");

        // Create render pipeline
        let basic_pipeline =
            RenderPipeline::new_basic_3d(&context.device, context.surface_config.format);

        // Create depth texture
        let depth_texture = DepthTexture::new(
            &context.device,
            context.surface_config.width,
            context.surface_config.height,
        );

        // Create camera uniform buffer
        let camera_uniform = CameraUniform::default();
        let camera_uniform_buffer =
            camera_uniform.create_buffer(&context.device, Some("Camera Uniform Buffer"));

        // Create camera bind group
        let camera_bind_group =
            basic_pipeline.create_camera_bind_group(&context.device, &camera_uniform_buffer);

        Self {
            context,
            basic_pipeline,
            depth_texture,
            camera_uniform_buffer,
            camera_bind_group,
            mesh_cache: HashMap::new(),
            next_mesh_id: 0,
        }
    }

    /// Resize the renderer when the window size changes
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.depth_texture =
                DepthTexture::new(&self.context.device, new_size.width, new_size.height);
        }
    }

    /// Upload a mesh to the GPU and return its ID
    pub fn upload_mesh(&mut self, mesh: &Mesh) -> u64 {
        let vertex_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let mesh_data = MeshGpuData {
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
        };

        let id = self.next_mesh_id;
        self.next_mesh_id += 1;
        self.mesh_cache.insert(id, mesh_data);

        id
    }

    /// Render a frame
    ///
    /// This queries the world for renderable entities and draws them.
    pub fn render(&mut self, world: &World) -> Result<(), wgpu::SurfaceError> {
        // Get the current surface texture
        let output = self.context.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Find the active camera
        let mut camera_data = None;
        let mut camera_query = world.query::<(&Camera, &GlobalTransform)>();
        if let Some((_, (camera, transform))) = camera_query.iter().next() {
            camera_data = Some((camera, transform));
        }

        if let Some((camera, camera_transform)) = camera_data {
            // Update camera uniform
            let view_proj = camera.view_projection_matrix(camera_transform);
            let camera_uniform = CameraUniform::new(view_proj);
            camera_uniform.update_buffer(&self.context.queue, &self.camera_uniform_buffer);
        }

        // Create command encoder
        let mut encoder = self.context.create_command_encoder(Some("Render Encoder"));

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Set pipeline and camera bind group
            render_pass.set_pipeline(&self.basic_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Query and render all entities with mesh, material, and transform
            let mut render_query = world.query::<(&MeshId, &Material, &GlobalTransform)>();
            for (entity, (mesh_id, material, transform)) in render_query.iter() {
                debug!(entity = ?entity, "Rendering entity");

                if let Some(mesh_data) = self.mesh_cache.get(&mesh_id.0) {
                    // Create object uniform
                    let object_uniform = ObjectUniform::new(transform.matrix, material.color);
                    let object_buffer =
                        object_uniform.create_buffer(&self.context.device, Some("Object Uniform"));
                    let object_bind_group = self
                        .basic_pipeline
                        .create_object_bind_group(&self.context.device, &object_buffer);

                    // Set object bind group
                    render_pass.set_bind_group(1, &object_bind_group, &[]);

                    // Set vertex and index buffers
                    render_pass.set_vertex_buffer(0, mesh_data.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        mesh_data.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    // Draw
                    render_pass.draw_indexed(0..mesh_data.num_indices, 0, 0..1);
                }
            }
        }

        // Submit command buffer
        self.context.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

/// Component to associate an entity with a mesh ID
#[derive(Debug, Clone, Copy)]
pub struct MeshId(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_id_creation() {
        let id = MeshId(42);
        assert_eq!(id.0, 42);
    }
}
