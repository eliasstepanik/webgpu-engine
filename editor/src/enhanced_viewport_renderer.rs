//! Enhanced viewport renderer using the new imgui-rs viewport APIs
//!
//! This module leverages the viewport support added to imgui-rs

use crate::safe_imgui_renderer::SafeImGuiRenderer;
use engine::graphics::RenderTargetInfo;
use imgui::{Context, DrawData, Id};
use imgui_wgpu::RendererConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use wgpu::*;
use wgpu::{
    LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
    TextureViewDescriptor,
};
use winit::window::WindowId;

/// Multi-viewport renderer using enhanced imgui-rs
pub struct EnhancedViewportRenderer {
    /// Main renderer with safety validation
    main_renderer: SafeImGuiRenderer,
    /// Per-viewport renderers
    viewport_renderers: HashMap<Id, ViewportRendererData>,
    /// Device reference
    device: Arc<Device>,
    /// Queue reference
    queue: Arc<Queue>,
    /// Texture format for creating new renderers
    texture_format: TextureFormat,
}

/// Data for rendering a specific viewport
struct ViewportRendererData {
    _renderer: SafeImGuiRenderer,
    _window_id: WindowId,
}

impl EnhancedViewportRenderer {
    /// Create a new enhanced viewport renderer
    pub fn new(
        imgui: &mut Context,
        device: Arc<Device>,
        queue: Arc<Queue>,
        config: RendererConfig,
    ) -> Self {
        // Extract texture format before moving config
        let texture_format = config.texture_format;

        let main_renderer = SafeImGuiRenderer::new(imgui, &device, &queue, config);

        info!("Created enhanced viewport renderer with imgui-rs viewport support");

        // Set renderer backend flag
        imgui
            .io_mut()
            .backend_flags
            .insert(imgui::BackendFlags::RENDERER_HAS_VIEWPORTS);
        info!("Set BackendFlags::RENDERER_HAS_VIEWPORTS");

        Self {
            main_renderer,
            viewport_renderers: HashMap::new(),
            device,
            queue,
            texture_format,
        }
    }

    /// Register a viewport for rendering
    pub fn register_viewport(&mut self, imgui: &mut Context, viewport_id: Id, window_id: WindowId) {
        // Create renderer for this viewport
        let config = RendererConfig {
            texture_format: self.texture_format,
            ..Default::default()
        };
        let renderer = SafeImGuiRenderer::new(imgui, &self.device, &self.queue, config);

        self.viewport_renderers.insert(
            viewport_id,
            ViewportRendererData {
                _renderer: renderer,
                _window_id: window_id,
            },
        );

        info!(
            "Registered viewport {:?} with window {:?} for rendering",
            viewport_id, window_id
        );
    }

    /// Unregister a viewport
    pub fn unregister_viewport(&mut self, viewport_id: Id) {
        if self.viewport_renderers.remove(&viewport_id).is_some() {
            info!("Unregistered viewport {:?}", viewport_id);
        }
    }

    /// Render all viewports
    pub fn render_all_viewports(
        &mut self,
        imgui: &mut Context,
        window_manager: &engine::windowing::WindowManager,
        clear_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Update platform windows first
        imgui.update_platform_windows();

        // For now, just render the main viewport
        // TODO: Check what viewport APIs are actually available in the fork
        warn!("Multi-viewport rendering not yet implemented - checking available APIs");

        // Render main viewport
        let draw_data = imgui.render();
        self.render_main_viewport(draw_data, window_manager, clear_color)?;

        // Let imgui handle platform window presentation
        imgui.render_platform_windows_default();

        Ok(())
    }

    /// Render the main viewport
    fn render_main_viewport(
        &mut self,
        draw_data: &DrawData,
        window_manager: &engine::windowing::WindowManager,
        clear_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let main_window = window_manager.get_main_window();
        let surface_texture = main_window.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let surface_width = surface_texture.texture.width();
        let surface_height = surface_texture.texture.height();

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Main Viewport Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ImGui Main Viewport Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.main_renderer.render_with_validation(
            draw_data,
            &self.queue,
            &self.device,
            &mut render_pass,
            RenderTargetInfo {
                width: surface_width,
                height: surface_height,
            },
        )?;

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }

    /// Get the main renderer (for texture management, etc.)
    pub fn main_renderer(&mut self) -> &mut SafeImGuiRenderer {
        &mut self.main_renderer
    }

    /// Handle viewport creation (called from ViewportBackend)
    pub fn on_viewport_created(
        &mut self,
        imgui: &mut Context,
        viewport_id: Id,
        window_id: WindowId,
    ) {
        self.register_viewport(imgui, viewport_id, window_id);
    }

    /// Handle viewport destruction
    pub fn on_viewport_destroyed(&mut self, viewport_id: Id) {
        self.unregister_viewport(viewport_id);
    }
}
