//! Renderer viewport backend for ImGui multi-viewport support
//!
//! This implements the RendererViewportBackend trait to handle rendering
//! for secondary viewports (windows that are dragged outside the main window).

use crate::safe_imgui_renderer::SafeImGuiRenderer;
use crate::viewport_workarounds;
use engine::graphics::RenderTargetInfo;
use imgui::{Id, RendererViewportBackend, Viewport};
use imgui_wgpu::RendererConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use wgpu::*;
use winit::window::WindowId;

/// Renderer backend for ImGui viewports
pub struct ViewportRendererBackend {
    /// Device reference
    device: Arc<Device>,
    /// Queue reference
    queue: Arc<Queue>,
    /// Texture format for creating new renderers
    texture_format: TextureFormat,
    /// Per-viewport renderer data
    viewport_renderers: HashMap<Id, ViewportRendererData>,
    /// Mapping from viewport ID to window ID
    viewport_to_window: HashMap<Id, WindowId>,
    /// Window manager reference (passed during rendering)
    window_manager: Option<*const engine::windowing::WindowManager>,
    /// Main context reference for creating renderers
    main_context: *mut imgui::Context,
}

/// Data for rendering a specific viewport
struct ViewportRendererData {
    renderer: SafeImGuiRenderer,
    window_id: WindowId,
}

// Safety: We only store a pointer to WindowManager during rendering
unsafe impl Send for ViewportRendererBackend {}
unsafe impl Sync for ViewportRendererBackend {}

impl ViewportRendererBackend {
    /// Create a new viewport renderer backend
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        texture_format: TextureFormat,
        context: &mut imgui::Context,
    ) -> Self {
        info!("Creating viewport renderer backend");

        Self {
            device,
            queue,
            texture_format,
            viewport_renderers: HashMap::new(),
            viewport_to_window: HashMap::new(),
            window_manager: None,
            main_context: context as *mut _,
        }
    }

    /// Set the window manager for the current frame
    ///
    /// # Safety
    /// The caller must ensure that the window manager reference remains valid for the entire frame.
    /// This method stores a raw pointer to the window manager that will be used during rendering.
    pub unsafe fn set_window_manager(&mut self, window_manager: &engine::windowing::WindowManager) {
        self.window_manager = Some(window_manager as *const _);
    }

    /// Clear the window manager reference
    pub fn clear_window_manager(&mut self) {
        self.window_manager = None;
    }

    /// Register a viewport with its window ID
    pub fn register_viewport(&mut self, viewport_id: Id, window_id: WindowId) {
        self.viewport_to_window.insert(viewport_id, window_id);
        info!(
            "Registered viewport {:?} with window {:?}",
            viewport_id, window_id
        );
    }
}

impl RendererViewportBackend for ViewportRendererBackend {
    fn create_window(&mut self, viewport: &mut Viewport) {
        info!(
            "=== RENDERER CREATE_WINDOW === for viewport {:?}",
            viewport.id
        );

        // Skip main viewport
        if viewport
            .flags
            .contains(imgui::ViewportFlags::IS_PLATFORM_WINDOW)
        {
            debug!("Skipping renderer setup for main viewport");
            return;
        }

        // We'll create the renderer when we first need to render to this viewport
        // For now, just log that we're ready
        info!("Renderer backend ready for viewport {:?}", viewport.id);
    }

    fn destroy_window(&mut self, viewport: &mut Viewport) {
        info!(
            "=== RENDERER DESTROY_WINDOW === for viewport {:?}",
            viewport.id
        );

        // Remove renderer for this viewport
        if let Some(data) = self.viewport_renderers.remove(&viewport.id) {
            info!(
                "Destroyed renderer for viewport {:?} (window {:?})",
                viewport.id, data.window_id
            );
        }

        self.viewport_to_window.remove(&viewport.id);
    }

    fn set_window_size(&mut self, viewport: &mut Viewport, size: [f32; 2]) {
        debug!(
            "Renderer set window size for viewport {:?} to {:?}",
            viewport.id, size
        );
        // Renderer will adapt to new size automatically when rendering
    }

    fn render_window(&mut self, viewport: &mut Viewport) {
        // Skip if this is the main viewport (handled separately)
        if !viewport
            .flags
            .contains(imgui::ViewportFlags::IS_PLATFORM_WINDOW)
        {
            debug!("Skipping render for main viewport {:?}", viewport.id);
            return;
        }

        info!(
            "=== RENDERER RENDER_WINDOW === for viewport {:?} (flags: {:?})",
            viewport.id, viewport.flags
        );

        // Get the window manager
        let window_manager = match self.window_manager {
            Some(wm) => unsafe { &*wm },
            None => {
                error!("Window manager not set for rendering!");
                return;
            }
        };

        // Get the window ID for this viewport
        let window_id = match self.viewport_to_window.get(&viewport.id) {
            Some(id) => *id,
            None => {
                warn!("No window ID found for viewport {:?}", viewport.id);
                return;
            }
        };

        // Get the window data
        let window_data = match window_manager.get_window(window_id) {
            Some(data) => data,
            None => {
                warn!(
                    "No window found for viewport {:?} (window {:?})",
                    viewport.id, window_id
                );
                return;
            }
        };

        // Create renderer for this viewport if needed
        if !self.viewport_renderers.contains_key(&viewport.id) {
            // Use the main context to create the renderer
            let config = RendererConfig {
                texture_format: self.texture_format,
                ..Default::default()
            };

            // Safety: We know the main context is valid for the lifetime of this backend
            let renderer = unsafe {
                let ctx = &mut *self.main_context;
                SafeImGuiRenderer::new(ctx, &self.device, &self.queue, config)
            };

            self.viewport_renderers.insert(
                viewport.id,
                ViewportRendererData {
                    renderer,
                    window_id,
                },
            );

            info!("Created renderer for viewport {:?}", viewport.id);
        }

        // Get the renderer data
        let renderer_data = match self.viewport_renderers.get_mut(&viewport.id) {
            Some(data) => data,
            None => {
                error!("Failed to get renderer data for viewport {:?}", viewport.id);
                return;
            }
        };

        // Get draw data for this viewport
        let draw_data = match viewport.draw_data() {
            Some(data) => data,
            None => {
                warn!("No draw data for viewport {:?}", viewport.id);
                return;
            }
        };

        // Render to the viewport's surface
        match window_data.surface.get_current_texture() {
            Ok(surface_texture) => {
                let view = surface_texture
                    .texture
                    .create_view(&TextureViewDescriptor::default());

                let mut encoder = self
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some(&format!("Viewport {:?} Encoder", viewport.id)),
                    });

                {
                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some(&format!("ImGui Viewport {:?} Pass", viewport.id)),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.1,
                                    a: 1.0,
                                }),
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    // Get the actual render target size
                    let surface_width = surface_texture.texture.width();
                    let surface_height = surface_texture.texture.height();

                    debug!("Viewport {:?} - Surface size: {}x{}, Draw data display size: {:?}, framebuffer size: {:?}",
                          viewport.id, surface_width, surface_height,
                          draw_data.display_size,
                          [draw_data.display_size[0] * draw_data.framebuffer_scale[0],
                           draw_data.display_size[1] * draw_data.framebuffer_scale[1]]);

                    let target_info = RenderTargetInfo {
                        width: surface_width,
                        height: surface_height,
                    };

                    // Apply workarounds before rendering
                    if viewport_workarounds::should_skip_viewport_render(
                        viewport.id,
                        target_info.width,
                        target_info.height,
                        draw_data,
                    ) {
                        debug!("Skipping viewport render due to size validation");
                        return;
                    }

                    if let Err(e) = renderer_data.renderer.render_with_validation(
                        draw_data,
                        &self.queue,
                        &self.device,
                        &mut render_pass,
                        target_info,
                    ) {
                        error!("Failed to render viewport {:?}: {}", viewport.id, e);
                    }
                }

                self.queue.submit(std::iter::once(encoder.finish()));
                surface_texture.present();

                debug!("Rendered viewport {:?} successfully", viewport.id);
            }
            Err(e) => {
                error!(
                    "Failed to get surface texture for viewport {:?}: {}",
                    viewport.id, e
                );
            }
        }
    }

    fn swap_buffers(&mut self, viewport: &mut Viewport) {
        // Buffer swapping is handled in render_window when we call present()
        debug!(
            "Swap buffers called for viewport {:?} (handled in render_window)",
            viewport.id
        );
    }
}
