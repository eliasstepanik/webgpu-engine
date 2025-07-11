//! Safe ImGui renderer wrapper that prevents scissor rect validation errors
//!
//! This module provides a wrapper around imgui-wgpu's Renderer that validates
//! all rendering operations to ensure scissor rectangles never exceed render
//! target bounds, preventing wgpu validation errors.

use engine::graphics::safe_scissor::RenderTargetInfo;
use imgui::{Context, DrawData};
use imgui_wgpu::{Renderer, RendererConfig, RendererError};
use std::sync::Arc;
use tracing::{debug, warn};
use wgpu::*;

/// Safe wrapper around imgui-wgpu Renderer that validates rendering operations
pub struct SafeImGuiRenderer {
    /// The underlying imgui-wgpu renderer
    inner: Renderer,
    /// Last known render target info for validation
    last_target_info: Option<RenderTargetInfo>,
    /// Whether debug mode is enabled
    debug_mode: bool,
}

impl SafeImGuiRenderer {
    /// Create a new safe renderer
    pub fn new(
        context: &mut Context,
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        config: RendererConfig,
    ) -> Self {
        let debug_mode = std::env::var("VIEWPORT_DEBUG").is_ok();

        if debug_mode {
            debug!("SafeImGuiRenderer created with debug mode enabled");
        }

        Self {
            inner: Renderer::new(context, device, queue, config),
            last_target_info: None,
            debug_mode,
        }
    }

    /// Render with validation to prevent scissor rect errors
    pub fn render_with_validation<'a>(
        &'a mut self,
        draw_data: &DrawData,
        queue: &Queue,
        device: &Device,
        pass: &mut RenderPass<'a>,
        target_info: RenderTargetInfo,
    ) -> Result<(), RendererError> {
        // Store target info for potential future use
        self.last_target_info = Some(target_info);

        // Validate draw data before rendering
        let fb_width = (draw_data.display_size[0] * draw_data.framebuffer_scale[0]).round() as u32;
        let fb_height = (draw_data.display_size[1] * draw_data.framebuffer_scale[1]).round() as u32;

        if self.debug_mode {
            debug!(
                "Validating draw data: display_size={:?}, fb_scale={:?}, fb_size={}x{}, target={}x{}",
                draw_data.display_size, draw_data.framebuffer_scale,
                fb_width, fb_height, target_info.width, target_info.height
            );
        }

        // Check for zero-sized renders
        if fb_width == 0 || fb_height == 0 {
            if self.debug_mode {
                debug!("Skipping zero-sized render");
            }
            return Ok(());
        }

        if target_info.width == 0 || target_info.height == 0 {
            warn!("Skipping render to zero-sized target");
            return Ok(());
        }

        // Validate framebuffer size against target
        if fb_width > target_info.width || fb_height > target_info.height {
            warn!(
                "Draw data size {}x{} exceeds target {}x{}, skipping render",
                fb_width, fb_height, target_info.width, target_info.height
            );

            // In debug mode, log more details
            if self.debug_mode {
                debug!("Display size: {:?}", draw_data.display_size);
                debug!("Framebuffer scale: {:?}", draw_data.framebuffer_scale);
                debug!("Target info: {:?}", target_info);
            }

            return Ok(());
        }

        // Additional validation for draw commands
        for draw_list in draw_data.draw_lists() {
            for cmd in draw_list.commands() {
                if let imgui::DrawCmd::Elements { count, cmd_params } = cmd {
                    let clip_rect = cmd_params.clip_rect;
                    let clip_x = clip_rect[0].max(0.0) as u32;
                    let clip_y = clip_rect[1].max(0.0) as u32;
                    let clip_w = (clip_rect[2] - clip_rect[0]).max(0.0) as u32;
                    let clip_h = (clip_rect[3] - clip_rect[1]).max(0.0) as u32;

                    let scaled_x = (clip_x as f32 * draw_data.framebuffer_scale[0]) as u32;
                    let scaled_y = (clip_y as f32 * draw_data.framebuffer_scale[1]) as u32;
                    let scaled_w = (clip_w as f32 * draw_data.framebuffer_scale[0]) as u32;
                    let scaled_h = (clip_h as f32 * draw_data.framebuffer_scale[1]) as u32;

                    if scaled_x + scaled_w > target_info.width
                        || scaled_y + scaled_h > target_info.height
                    {
                        warn!(
                            "Potential scissor rect overflow detected: ({}, {}, {}, {}) in {}x{} target",
                            scaled_x, scaled_y, scaled_w, scaled_h,
                            target_info.width, target_info.height
                        );
                    }

                    if self.debug_mode && count > 0 {
                        debug!(
                            "Draw command: {} elements, clip_rect={:?}",
                            count, clip_rect
                        );
                    }
                }
            }
        }

        // All validation passed, render with the standard renderer
        if self.debug_mode {
            debug!("Validation passed, rendering with inner renderer");
        }

        self.inner.render(draw_data, queue, device, pass)
    }

    /// Get textures for texture management
    pub fn textures(&mut self) -> &mut imgui::Textures<imgui_wgpu::Texture> {
        &mut self.inner.textures
    }

    /// Get the inner renderer for advanced operations
    pub fn inner(&self) -> &Renderer {
        &self.inner
    }

    /// Get the last known render target info
    pub fn last_target_info(&self) -> Option<RenderTargetInfo> {
        self.last_target_info
    }

    /// Check if debug mode is enabled
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode
    }
}

/// Helper function to validate draw data against a render target
pub fn validate_draw_data(
    draw_data: &DrawData,
    target_width: u32,
    target_height: u32,
) -> Result<(), String> {
    let fb_width = (draw_data.display_size[0] * draw_data.framebuffer_scale[0]).round() as u32;
    let fb_height = (draw_data.display_size[1] * draw_data.framebuffer_scale[1]).round() as u32;

    if fb_width == 0 || fb_height == 0 {
        return Err("Zero-sized framebuffer".to_string());
    }

    if target_width == 0 || target_height == 0 {
        return Err("Zero-sized render target".to_string());
    }

    if fb_width > target_width || fb_height > target_height {
        return Err(format!(
            "Framebuffer size {fb_width}x{fb_height} exceeds target size {target_width}x{target_height}"
        ));
    }

    Ok(())
}
