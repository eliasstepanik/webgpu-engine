//! Main renderer implementation
//!
//! The Renderer struct orchestrates all rendering operations, managing
//! render pipelines, GPU resources, and the rendering of entities.

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::core::camera::{Camera, CameraWorldPosition};
use crate::core::entity::{components::GlobalWorldTransform, GlobalTransform, World};
use crate::graphics::{
    context::RenderContext,
    material::Material,
    mesh::Mesh,
    mesh_library::MeshLibrary,
    pipeline::{DepthTexture, RenderPipeline},
    render_target::RenderTarget,
    uniform::{CameraUniform, ObjectUniform, UniformBuffer},
};
use crate::io::component_registry::ComponentRegistry;
use glam::{DVec3, Mat4, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};
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
    /// Outline render pipeline for selection
    outline_pipeline: RenderPipeline,
    /// Depth texture for depth testing
    depth_texture: DepthTexture,
    /// Camera uniform buffer
    camera_uniform_buffer: wgpu::Buffer,
    /// Camera bind group
    camera_bind_group: wgpu::BindGroup,
    /// Outline camera bind group
    outline_camera_bind_group: wgpu::BindGroup,
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

        // Create render pipelines
        let basic_pipeline = RenderPipeline::new_basic_3d(&context.device, surface_format);
        let outline_pipeline = RenderPipeline::new_outline(&context.device, surface_format);

        // Create depth texture with default size
        let depth_texture = DepthTexture::new(&context.device, 1280, 720);

        // Create camera uniform buffer
        let camera_uniform = CameraUniform::default();
        let camera_uniform_buffer =
            camera_uniform.create_buffer(&context.device, Some("Camera Uniform Buffer"));

        // Create camera bind groups
        let camera_bind_group =
            basic_pipeline.create_camera_bind_group(&context.device, &camera_uniform_buffer);
        let outline_camera_bind_group =
            outline_pipeline.create_camera_bind_group(&context.device, &camera_uniform_buffer);

        Self {
            context,
            basic_pipeline,
            outline_pipeline,
            depth_texture,
            camera_uniform_buffer,
            camera_bind_group,
            outline_camera_bind_group,
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
            self.outline_camera_bind_group = self
                .outline_pipeline
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
            // First, check if it's a file path
            if mesh_id.0.contains('/') || mesh_id.0.contains('\\') || mesh_id.0.ends_with(".obj") {
                // Try to load from file
                let path = std::path::Path::new(&mesh_id.0);
                match crate::graphics::mesh_loader::load_mesh_from_file(path) {
                    Ok(mesh) => {
                        info!("Loaded mesh from file: {}", mesh_id.0);
                        self.upload_mesh(&mesh, &mesh_id.0);
                    }
                    Err(e) => {
                        error!("Failed to load mesh from file {}: {}", mesh_id.0, e);
                        // Use fallback mesh
                        let fallback = MeshLibrary::error_mesh();
                        self.upload_mesh(&fallback, &mesh_id.0);
                    }
                }
            } else if let Some(mesh) = self.mesh_library.get_or_generate(&mesh_id.0) {
                // Try to get mesh from library
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
        self.render_with_selection(world, surface, None)
    }

    /// Render a frame to a surface with optional selection highlighting
    pub fn render_with_selection(
        &mut self,
        world: &World,
        surface: &wgpu::Surface,
        selected_entity: Option<hecs::Entity>,
    ) -> Result<(), wgpu::SurfaceError> {
        // Get the current surface texture
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update surface format if needed
        self.update_surface_format(self.context.get_preferred_format(surface));

        // Find the active camera and determine camera world position
        let mut camera_view_proj = None;
        let mut camera_world_position = DVec3::ZERO;

        // First try to find a camera with WorldTransform (large world camera)
        let mut world_camera_query =
            world.query::<(&Camera, &GlobalWorldTransform, Option<&CameraWorldPosition>)>();
        if let Some((_, (camera, world_transform, world_pos))) = world_camera_query.iter().next() {
            // Use world position if available, otherwise derive from transform
            camera_world_position = if let Some(pos) = world_pos {
                pos.position
            } else {
                world_transform.position()
            };

            // Calculate view-projection using camera-relative coordinates
            let view_proj =
                camera.view_projection_matrix_world(world_transform, camera_world_position);
            camera_view_proj = Some(view_proj);
        } else {
            // Fall back to regular camera with GlobalTransform
            let mut regular_camera_query =
                world.query::<(&Camera, &GlobalTransform, Option<&CameraWorldPosition>)>();
            if let Some((_, (camera, transform, cam_world_pos))) =
                regular_camera_query.iter().next()
            {
                let view_proj = camera.view_projection_matrix(transform);
                camera_view_proj = Some(view_proj);

                // Use CameraWorldPosition if available for exact position, otherwise extract from transform
                camera_world_position = if let Some(world_pos) = cam_world_pos {
                    world_pos.position
                } else {
                    // Fallback to extracting from transform (less precise for parented cameras)
                    DVec3::new(
                        transform.position().x as f64,
                        transform.position().y as f64,
                        transform.position().z as f64,
                    )
                };
            }
        }

        if let Some(view_proj) = camera_view_proj {
            // Update camera uniform
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

            // Query and render all entities with mesh, material, and transform
            // Handle both regular and world transforms
            let mut entities_to_render = Vec::new();

            // Collect entities with regular GlobalTransform
            let mut regular_query = world.query::<(&MeshId, &Material, &GlobalTransform)>();
            for (entity, (mesh_id, material, transform)) in regular_query.iter() {
                // Get object position in f64 for precision
                let object_pos = transform.position();
                let object_pos_f64 = DVec3::new(
                    object_pos.x as f64,
                    object_pos.y as f64,
                    object_pos.z as f64,
                );

                // Calculate camera-relative position with f64 precision
                let relative_pos_f64 = object_pos_f64 - camera_world_position;

                // Convert back to f32 for rendering
                let camera_relative_pos = Vec3::new(
                    relative_pos_f64.x as f32,
                    relative_pos_f64.y as f32,
                    relative_pos_f64.z as f32,
                );

                // Decompose to get rotation and scale (these don't need adjustment)
                let (scale, rotation, _) = transform.matrix.to_scale_rotation_translation();

                // Reconstruct matrix with camera-relative position
                let camera_relative_matrix =
                    Mat4::from_scale_rotation_translation(scale, rotation, camera_relative_pos);

                entities_to_render.push((
                    entity,
                    mesh_id.clone(),
                    *material,
                    camera_relative_matrix,
                ));
            }

            // Collect entities with WorldTransform and convert to camera-relative
            let mut world_query = world.query::<(&MeshId, &Material, &GlobalWorldTransform)>();
            for (entity, (mesh_id, material, world_transform)) in world_query.iter() {
                // Convert world transform to camera-relative transform for rendering
                let camera_relative_transform =
                    world_transform.to_camera_relative(camera_world_position);
                entities_to_render.push((
                    entity,
                    mesh_id.clone(),
                    *material,
                    camera_relative_transform.matrix,
                ));
            }

            // First pass: Render outline for selected entity
            if let Some(selected) = selected_entity {
                // Find the selected entity in our render list
                if let Some((_, mesh_id, _, transform_matrix)) = entities_to_render
                    .iter()
                    .find(|(e, _, _, _)| *e == selected)
                {
                    // Ensure mesh is loaded
                    let _ = self.get_or_create_mesh(mesh_id);
                    let mesh_data = &self.mesh_cache[&mesh_id.0];

                    // Use outline pipeline
                    render_pass.set_pipeline(&self.outline_pipeline.pipeline);
                    render_pass.set_bind_group(0, &self.outline_camera_bind_group, &[]);

                    // Create outline uniform with bright color
                    let outline_color = [0.0, 1.0, 1.0, 1.0]; // Bright cyan outline for better visibility
                    let outline_uniform = ObjectUniform::new(*transform_matrix, outline_color);
                    let outline_buffer = outline_uniform
                        .create_buffer(&self.context.device, Some("Outline Uniform"));
                    let outline_bind_group = self
                        .outline_pipeline
                        .create_object_bind_group(&self.context.device, &outline_buffer);

                    render_pass.set_bind_group(1, &outline_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, mesh_data.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        mesh_data.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(0..mesh_data.num_indices, 0, 0..1);
                }
            }

            // Always ensure basic pipeline is set for normal rendering
            render_pass.set_pipeline(&self.basic_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Second pass: Render all entities normally
            for (entity, mesh_id, material, transform_matrix) in entities_to_render {
                debug!(entity = ?entity, "Rendering entity");

                // Ensure mesh is loaded (may borrow self mutably)
                let _ = self.get_or_create_mesh(&mesh_id);

                // Now access mesh data (no mut borrow)
                let mesh_data = &self.mesh_cache[&mesh_id.0];

                // Create object uniform
                let object_uniform = ObjectUniform::new(transform_matrix, material.color);
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

        // Get the specified camera and its transform, and determine camera world position
        #[allow(unused_assignments)]
        let mut camera_world_position = DVec3::ZERO;

        // First try to get camera with WorldTransform (large world camera)
        let world_camera_data = world
            .query_one::<(&Camera, &GlobalWorldTransform, Option<&CameraWorldPosition>)>(
                camera_entity,
            );
        if let Ok(mut query_one) = world_camera_data {
            if let Some((camera, world_transform, world_pos)) = query_one.get() {
                // Use world position if available, otherwise derive from transform
                camera_world_position = if let Some(pos) = world_pos {
                    pos.position
                } else {
                    world_transform.position()
                };

                // Calculate view-projection using camera-relative coordinates
                let view_proj =
                    camera.view_projection_matrix_world(world_transform, camera_world_position);
                let camera_uniform = CameraUniform::new(view_proj);
                camera_uniform.update_buffer(&self.context.queue, &self.camera_uniform_buffer);
            } else {
                debug!("Camera entity missing required world transform components, trying regular transform");

                // Fall back to regular camera with GlobalTransform
                let camera_data = world
                    .query_one::<(&Camera, &GlobalTransform, Option<&CameraWorldPosition>)>(
                        camera_entity,
                    );
                if let Ok(mut query_one) = camera_data {
                    if let Some((camera, camera_transform, cam_world_pos)) = query_one.get() {
                        // Update camera uniform
                        let view_proj = camera.view_projection_matrix(camera_transform);
                        let camera_uniform = CameraUniform::new(view_proj);
                        camera_uniform
                            .update_buffer(&self.context.queue, &self.camera_uniform_buffer);

                        // Use CameraWorldPosition if available for exact position, otherwise extract from transform
                        camera_world_position = if let Some(world_pos) = cam_world_pos {
                            world_pos.position
                        } else {
                            // Fallback to extracting from transform (less precise for parented cameras)
                            DVec3::new(
                                camera_transform.position().x as f64,
                                camera_transform.position().y as f64,
                                camera_transform.position().z as f64,
                            )
                        };
                    } else {
                        debug!("Camera entity missing required components, skipping render");
                        return Ok(());
                    }
                } else {
                    debug!("Camera entity not found, skipping render");
                    return Ok(());
                }
            }
        } else {
            // Fall back to regular camera with GlobalTransform
            let camera_data = world
                .query_one::<(&Camera, &GlobalTransform, Option<&CameraWorldPosition>)>(
                    camera_entity,
                );
            if let Ok(mut query_one) = camera_data {
                if let Some((camera, camera_transform, cam_world_pos)) = query_one.get() {
                    // Update camera uniform
                    let view_proj = camera.view_projection_matrix(camera_transform);
                    let camera_uniform = CameraUniform::new(view_proj);
                    camera_uniform.update_buffer(&self.context.queue, &self.camera_uniform_buffer);

                    // Use CameraWorldPosition if available for exact position, otherwise extract from transform
                    camera_world_position = if let Some(world_pos) = cam_world_pos {
                        world_pos.position
                    } else {
                        // Fallback to extracting from transform (less precise for parented cameras)
                        DVec3::new(
                            camera_transform.position().x as f64,
                            camera_transform.position().y as f64,
                            camera_transform.position().z as f64,
                        )
                    };
                } else {
                    debug!("Camera entity missing required components, skipping render");
                    return Ok(());
                }
            } else {
                debug!("Camera entity not found, skipping render");
                return Ok(());
            }
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

            // Query and render all entities with mesh, material, and transform
            // Handle both regular and world transforms
            let mut entities_to_render = Vec::new();

            // Collect entities with regular GlobalTransform
            let mut regular_query = world.query::<(&MeshId, &Material, &GlobalTransform)>();
            for (entity, (mesh_id, material, transform)) in regular_query.iter() {
                // Get object position in f64 for precision
                let object_pos = transform.position();
                let object_pos_f64 = DVec3::new(
                    object_pos.x as f64,
                    object_pos.y as f64,
                    object_pos.z as f64,
                );

                // Calculate camera-relative position with f64 precision
                let relative_pos_f64 = object_pos_f64 - camera_world_position;

                // Convert back to f32 for rendering
                let camera_relative_pos = Vec3::new(
                    relative_pos_f64.x as f32,
                    relative_pos_f64.y as f32,
                    relative_pos_f64.z as f32,
                );

                // Decompose to get rotation and scale (these don't need adjustment)
                let (scale, rotation, _) = transform.matrix.to_scale_rotation_translation();

                // Reconstruct matrix with camera-relative position
                let camera_relative_matrix =
                    Mat4::from_scale_rotation_translation(scale, rotation, camera_relative_pos);

                entities_to_render.push((
                    entity,
                    mesh_id.clone(),
                    *material,
                    camera_relative_matrix,
                ));
            }

            // Collect entities with WorldTransform and convert to camera-relative
            let mut world_query = world.query::<(&MeshId, &Material, &GlobalWorldTransform)>();
            for (entity, (mesh_id, material, world_transform)) in world_query.iter() {
                // Convert world transform to camera-relative transform for rendering
                let camera_relative_transform =
                    world_transform.to_camera_relative(camera_world_position);
                entities_to_render.push((
                    entity,
                    mesh_id.clone(),
                    *material,
                    camera_relative_transform.matrix,
                ));
            }

            for (entity, mesh_id, material, transform_matrix) in entities_to_render {
                debug!(entity = ?entity, mesh_id = %mesh_id.0, "Rendering entity from world");

                // Ensure mesh is loaded (may borrow self mutably)
                let _ = self.get_or_create_mesh(&mesh_id);

                // Now access mesh data (no mut borrow)
                let mesh_data = &self.mesh_cache[&mesh_id.0];

                // Create object uniform
                let object_uniform = ObjectUniform::new(transform_matrix, material.color);
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
        self.render_to_target_with_selection(world, render_target, None)
    }

    /// Render to a specific render target with optional selection highlighting
    pub fn render_to_target_with_selection(
        &mut self,
        world: &World,
        render_target: &RenderTarget,
        selected_entity: Option<hecs::Entity>,
    ) -> Result<(), wgpu::SurfaceError> {
        // Find the active camera and determine camera world position
        let mut camera_world_position = DVec3::ZERO;

        // First try to find a camera with WorldTransform (large world camera)
        let mut world_camera_query =
            world.query::<(&Camera, &GlobalWorldTransform, Option<&CameraWorldPosition>)>();
        if let Some((_, (camera, world_transform, world_pos))) = world_camera_query.iter().next() {
            // Use world position if available, otherwise derive from transform
            camera_world_position = if let Some(pos) = world_pos {
                pos.position
            } else {
                world_transform.position()
            };

            // Calculate view-projection using camera-relative coordinates
            let view_proj =
                camera.view_projection_matrix_world(world_transform, camera_world_position);
            let camera_uniform = CameraUniform::new(view_proj);
            camera_uniform.update_buffer(&self.context.queue, &self.camera_uniform_buffer);
        } else {
            // Fall back to regular camera with GlobalTransform
            let mut regular_camera_query =
                world.query::<(&Camera, &GlobalTransform, Option<&CameraWorldPosition>)>();
            if let Some((_, (camera, transform, cam_world_pos))) =
                regular_camera_query.iter().next()
            {
                let view_proj = camera.view_projection_matrix(transform);
                let camera_uniform = CameraUniform::new(view_proj);
                camera_uniform.update_buffer(&self.context.queue, &self.camera_uniform_buffer);

                // Use CameraWorldPosition if available for exact position, otherwise extract from transform
                camera_world_position = if let Some(world_pos) = cam_world_pos {
                    world_pos.position
                } else {
                    // Fallback to extracting from transform (less precise for parented cameras)
                    DVec3::new(
                        transform.position().x as f64,
                        transform.position().y as f64,
                        transform.position().z as f64,
                    )
                };
            }
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
                    view: &render_target.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Query and render all entities with mesh, material, and transform
            // Handle both regular and world transforms
            let mut entities_to_render = Vec::new();

            // Collect entities with regular GlobalTransform
            let mut regular_query = world.query::<(&MeshId, &Material, &GlobalTransform)>();
            for (entity, (mesh_id, material, transform)) in regular_query.iter() {
                // Get object position in f64 for precision
                let object_pos = transform.position();
                let object_pos_f64 = DVec3::new(
                    object_pos.x as f64,
                    object_pos.y as f64,
                    object_pos.z as f64,
                );

                // Calculate camera-relative position with f64 precision
                let relative_pos_f64 = object_pos_f64 - camera_world_position;

                // Convert back to f32 for rendering
                let camera_relative_pos = Vec3::new(
                    relative_pos_f64.x as f32,
                    relative_pos_f64.y as f32,
                    relative_pos_f64.z as f32,
                );

                // Decompose to get rotation and scale (these don't need adjustment)
                let (scale, rotation, _) = transform.matrix.to_scale_rotation_translation();

                // Reconstruct matrix with camera-relative position
                let camera_relative_matrix =
                    Mat4::from_scale_rotation_translation(scale, rotation, camera_relative_pos);

                entities_to_render.push((
                    entity,
                    mesh_id.clone(),
                    *material,
                    camera_relative_matrix,
                ));
            }

            // Collect entities with WorldTransform and convert to camera-relative
            let mut world_query = world.query::<(&MeshId, &Material, &GlobalWorldTransform)>();
            for (entity, (mesh_id, material, world_transform)) in world_query.iter() {
                // Convert world transform to camera-relative transform for rendering
                let camera_relative_transform =
                    world_transform.to_camera_relative(camera_world_position);
                entities_to_render.push((
                    entity,
                    mesh_id.clone(),
                    *material,
                    camera_relative_transform.matrix,
                ));
            }

            // First pass: Render outline for selected entity
            if let Some(selected) = selected_entity {
                // Find the selected entity in our render list
                if let Some((_, mesh_id, _, transform_matrix)) = entities_to_render
                    .iter()
                    .find(|(e, _, _, _)| *e == selected)
                {
                    // Ensure mesh is loaded
                    let _ = self.get_or_create_mesh(mesh_id);
                    let mesh_data = &self.mesh_cache[&mesh_id.0];

                    // Use outline pipeline
                    render_pass.set_pipeline(&self.outline_pipeline.pipeline);
                    render_pass.set_bind_group(0, &self.outline_camera_bind_group, &[]);

                    // Create outline uniform with bright color
                    let outline_color = [0.0, 1.0, 1.0, 1.0]; // Bright cyan outline for better visibility
                    let outline_uniform = ObjectUniform::new(*transform_matrix, outline_color);
                    let outline_buffer = outline_uniform
                        .create_buffer(&self.context.device, Some("Outline Uniform"));
                    let outline_bind_group = self
                        .outline_pipeline
                        .create_object_bind_group(&self.context.device, &outline_buffer);

                    render_pass.set_bind_group(1, &outline_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, mesh_data.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        mesh_data.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(0..mesh_data.num_indices, 0, 0..1);
                }
            }

            // Always ensure basic pipeline is set for normal rendering
            render_pass.set_pipeline(&self.basic_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Second pass: Render all entities normally
            for (entity, mesh_id, material, transform_matrix) in entities_to_render {
                debug!(entity = ?entity, "Rendering entity to target");

                // Ensure mesh is loaded (may borrow self mutably)
                let _ = self.get_or_create_mesh(&mesh_id);

                // Now access mesh data (no mut borrow)
                let mesh_data = &self.mesh_cache[&mesh_id.0];

                // Create object uniform
                let object_uniform = ObjectUniform::new(transform_matrix, material.color);
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

    /// Get a list of all available mesh names
    pub fn get_available_meshes(&self) -> Vec<String> {
        let mut meshes = self.mesh_library.available_meshes();

        // Add any uploaded meshes that aren't in the library
        for mesh_name in self.mesh_cache.keys() {
            if !meshes.contains(mesh_name) {
                meshes.push(mesh_name.clone());
            }
        }

        meshes.sort();
        meshes
    }
}

/// Component to associate an entity with a mesh ID
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    engine_derive::Component,
    engine_derive::EditorUI,
)]
#[component(name = "MeshId")]
pub struct MeshId(
    #[ui(tooltip = "Mesh identifier (e.g. cube, sphere, or path/to/model.obj)")] pub String,
);

impl Default for MeshId {
    fn default() -> Self {
        Self("cube".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_id_creation() {
        let id = MeshId("cube".to_string());
        assert_eq!(id.0, "cube");
    }
}
