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
    mesh_library::MeshLibrary,
    pipeline::{DepthTexture, RenderPipeline},
    render_target::RenderTarget,
    uniform::{CameraUniform, ObjectUniform, UniformBuffer},
};
use serde::{Deserialize, Serialize};
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
pub struct Renderer {
    /// Render context with device and queue
    context: Arc<RenderContext>,
    /// Basic 3D render pipeline
    basic_pipeline: RenderPipeline,
    /// Depth texture for depth testing
    depth_texture: DepthTexture,
    /// Camera uniform buffer
    camera_uniform_buffer: wgpu::Buffer,
    /// Camera bind group
    camera_bind_group: wgpu::BindGroup,
    /// Cached mesh GPU data
    mesh_cache: HashMap<String, MeshGpuData>,
    /// Mesh library for default meshes and fallbacks
    mesh_library: MeshLibrary,
    /// Current surface format
    surface_format: wgpu::TextureFormat,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(context: Arc<RenderContext>) -> Self {
        info!("Initializing renderer");

        // Default format - will be updated when rendering to a surface
        let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        // Create render pipeline
        let basic_pipeline = RenderPipeline::new_basic_3d(&context.device, surface_format);

        // Create depth texture with default size
        let depth_texture = DepthTexture::new(&context.device, 1280, 720);

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
            mesh_library: MeshLibrary::new(),
            surface_format,
        }
    }

    /// Resize the renderer when the window size changes
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // Recreate the depth texture with the new size
            self.depth_texture =
                DepthTexture::new(&self.context.device, new_size.width, new_size.height);
        }
    }

    /// Update the surface format if it changes
    pub fn update_surface_format(&mut self, format: wgpu::TextureFormat) {
        if self.surface_format != format {
            self.surface_format = format;
            // Recreate pipeline with new format
            self.basic_pipeline = RenderPipeline::new_basic_3d(&self.context.device, format);
            // Recreate camera bind group
            self.camera_bind_group = self
                .basic_pipeline
                .create_camera_bind_group(&self.context.device, &self.camera_uniform_buffer);
        }
    }

    /// Upload a mesh to the GPU and return its ID
    pub fn upload_mesh(&mut self, mesh: &Mesh, name: &str) -> MeshId {
        let vertex_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{name} Vertex Buffer")),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{name} Index Buffer")),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let mesh_data = MeshGpuData {
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
        };

        let mesh_id = MeshId(name.to_string());
        self.mesh_cache.insert(name.to_string(), mesh_data);

        info!(name = %name, vertices = mesh.vertices.len(), indices = mesh.indices.len(), "Uploaded mesh to GPU");
        mesh_id
    }

    /// Get or create a mesh from the library
    fn get_or_create_mesh(&mut self, mesh_id: &MeshId) -> Result<(), String> {
        if !self.mesh_cache.contains_key(&mesh_id.0) {
            // Try to get mesh from library
            if let Some(mesh) = self.mesh_library.get_or_generate(&mesh_id.0) {
                self.upload_mesh(&mesh, &mesh_id.0);
            } else {
                // Use fallback cube mesh
                let fallback = MeshLibrary::error_mesh();
                self.upload_mesh(&fallback, &mesh_id.0);
                debug!("Using fallback mesh for ID: {}", mesh_id.0);
            }
        }
        Ok(())
    }

    /// Render a frame to a surface
    pub fn render(
        &mut self,
        world: &World,
        surface: &wgpu::Surface,
    ) -> Result<(), wgpu::SurfaceError> {
        // Get the current surface texture
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update surface format if needed
        self.update_surface_format(self.context.get_preferred_format(surface));

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
            let entities_to_render: Vec<_> = render_query
                .iter()
                .map(|(entity, (mesh_id, material, transform))| {
                    (entity, mesh_id.clone(), *material, *transform)
                })
                .collect();

            for (entity, mesh_id, material, transform) in entities_to_render {
                debug!(entity = ?entity, "Rendering entity");

                // Ensure mesh is loaded (may borrow self mutably)
                let _ = self.get_or_create_mesh(&mesh_id);

                // Now access mesh data (no mut borrow)
                let mesh_data = &self.mesh_cache[&mesh_id.0];

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
                render_pass
                    .set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                // Draw
                render_pass.draw_indexed(0..mesh_data.num_indices, 0, 0..1);
            }
        }

        // Submit command buffer
        self.context.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render a world using a specific camera entity to a surface
    ///
    /// This method provides more control over which camera to use for rendering
    /// and is useful for scene-loaded entities with predefined cameras.
    pub fn render_world(
        &mut self,
        world: &World,
        camera_entity: hecs::Entity,
        surface: &wgpu::Surface,
    ) -> Result<(), wgpu::SurfaceError> {
        // Get the current surface texture
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update surface format if needed
        self.update_surface_format(self.context.get_preferred_format(surface));

        // Get the specified camera and its transform
        let camera_data = world.query_one::<(&Camera, &GlobalTransform)>(camera_entity);
        if let Ok(mut query_one) = camera_data {
            if let Some((camera, camera_transform)) = query_one.get() {
                // Update camera uniform
                let view_proj = camera.view_projection_matrix(camera_transform);
                let camera_uniform = CameraUniform::new(view_proj);
                camera_uniform.update_buffer(&self.context.queue, &self.camera_uniform_buffer);
            } else {
                debug!("Camera entity missing required components, skipping render");
                return Ok(());
            }
        } else {
            debug!("Camera entity not found, skipping render");
            return Ok(());
        }

        // Create command encoder
        let mut encoder = self
            .context
            .create_command_encoder(Some("Render World Encoder"));

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render World Pass"),
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
            let entities_to_render: Vec<_> = render_query
                .iter()
                .map(|(entity, (mesh_id, material, transform))| {
                    (entity, mesh_id.clone(), *material, *transform)
                })
                .collect();

            for (entity, mesh_id, material, transform) in entities_to_render {
                debug!(entity = ?entity, mesh_id = %mesh_id.0, "Rendering entity from world");

                // Ensure mesh is loaded (may borrow self mutably)
                let _ = self.get_or_create_mesh(&mesh_id);

                // Now access mesh data (no mut borrow)
                let mesh_data = &self.mesh_cache[&mesh_id.0];

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
                render_pass
                    .set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                // Draw
                render_pass.draw_indexed(0..mesh_data.num_indices, 0, 0..1);
            }
        }

        // Submit command buffer
        self.context.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render to a custom render target instead of the surface
    ///
    /// This is used by the editor to render the game view to a texture
    /// that can be displayed in an egui viewport window.
    pub fn render_to_target(
        &mut self,
        world: &World,
        render_target: &RenderTarget,
    ) -> Result<(), wgpu::SurfaceError> {
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
        let mut encoder = self
            .context
            .create_command_encoder(Some("Render to Target Encoder"));

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render to Target Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_target.view,
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
            let entities_to_render: Vec<_> = render_query
                .iter()
                .map(|(entity, (mesh_id, material, transform))| {
                    (entity, mesh_id.clone(), *material, *transform)
                })
                .collect();

            for (entity, mesh_id, material, transform) in entities_to_render {
                debug!(entity = ?entity, "Rendering entity to target");

                // Ensure mesh is loaded (may borrow self mutably)
                let _ = self.get_or_create_mesh(&mesh_id);

                // Now access mesh data (no mut borrow)
                let mesh_data = &self.mesh_cache[&mesh_id.0];

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
                render_pass
                    .set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                // Draw
                render_pass.draw_indexed(0..mesh_data.num_indices, 0, 0..1);
            }
        }

        // Submit command buffer
        self.context.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

/// Component to associate an entity with a mesh ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MeshId(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_id_creation() {
        let id = MeshId("cube".to_string());
        assert_eq!(id.0, "cube");
    }
}
