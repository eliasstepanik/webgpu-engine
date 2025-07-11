//! Safe scissor rect handling to prevent validation errors
//!
//! This module provides a defensive wrapper around wgpu's set_scissor_rect
//! that ensures scissor rectangles are always within valid bounds.

use tracing::{debug, warn};
use wgpu::RenderPass;

/// Information about a render target for scissor rect validation
#[derive(Debug, Clone, Copy)]
pub struct RenderTargetInfo {
    pub width: u32,
    pub height: u32,
}

/// Safe wrapper around set_scissor_rect that prevents validation errors
///
/// This function ensures that scissor rectangles are always within the bounds
/// of the render target, preventing wgpu validation errors.
///
/// # Arguments
/// * `pass` - The render pass to set the scissor rect on
/// * `x` - X coordinate of the scissor rect
/// * `y` - Y coordinate of the scissor rect  
/// * `width` - Width of the scissor rect
/// * `height` - Height of the scissor rect
/// * `target_info` - Information about the render target bounds
pub fn safe_set_scissor_rect(
    pass: &mut RenderPass,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    target_info: RenderTargetInfo,
) {
    // Validate inputs
    if target_info.width == 0 || target_info.height == 0 {
        warn!(
            "Skipping scissor rect for zero-sized render target: {:?}",
            target_info
        );
        return;
    }

    // Clamp x and y to be within bounds
    let clamped_x = x.min(target_info.width.saturating_sub(1));
    let clamped_y = y.min(target_info.height.saturating_sub(1));

    // Calculate maximum allowed width and height from the clamped position
    let max_width = target_info.width.saturating_sub(clamped_x);
    let max_height = target_info.height.saturating_sub(clamped_y);

    // Clamp width and height
    let clamped_width = width.min(max_width).max(1);
    let clamped_height = height.min(max_height).max(1);

    // Log if we had to clamp
    if clamped_x != x || clamped_y != y || clamped_width != width || clamped_height != height {
        debug!(
            "Scissor rect clamped: ({}, {}, {}, {}) -> ({}, {}, {}, {}) for target {:?}",
            x, y, width, height, clamped_x, clamped_y, clamped_width, clamped_height, target_info
        );
    }

    // Ensure we're not setting a degenerate scissor rect
    if clamped_width == 0 || clamped_height == 0 {
        warn!(
            "Skipping degenerate scissor rect: ({}, {}, {}, {})",
            clamped_x, clamped_y, clamped_width, clamped_height
        );
        return;
    }

    // Final validation
    let end_x = clamped_x + clamped_width;
    let end_y = clamped_y + clamped_height;

    if end_x > target_info.width || end_y > target_info.height {
        warn!(
            "Scissor rect extends beyond render target: ({}, {}) > ({}, {})",
            end_x, end_y, target_info.width, target_info.height
        );
        // Extra safety: clamp one more time
        let final_width = clamped_width.min(target_info.width.saturating_sub(clamped_x));
        let final_height = clamped_height.min(target_info.height.saturating_sub(clamped_y));

        pass.set_scissor_rect(clamped_x, clamped_y, final_width, final_height);
    } else {
        // Safe to set
        pass.set_scissor_rect(clamped_x, clamped_y, clamped_width, clamped_height);
    }
}

/// Converts viewport-relative coordinates to render target coordinates with safety checks
///
/// This handles DPI scaling and ensures coordinates are within valid bounds.
pub fn viewport_to_render_target_scissor(
    viewport_x: f32,
    viewport_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    framebuffer_scale: [f32; 2],
    target_info: RenderTargetInfo,
) -> (u32, u32, u32, u32) {
    // Convert to physical pixels
    let phys_x = (viewport_x * framebuffer_scale[0]).round() as u32;
    let phys_y = (viewport_y * framebuffer_scale[1]).round() as u32;
    let phys_width = (viewport_width * framebuffer_scale[0]).round() as u32;
    let phys_height = (viewport_height * framebuffer_scale[1]).round() as u32;

    // Clamp to target bounds
    let clamped_x = phys_x.min(target_info.width.saturating_sub(1));
    let clamped_y = phys_y.min(target_info.height.saturating_sub(1));
    let clamped_width = phys_width
        .min(target_info.width.saturating_sub(clamped_x))
        .max(1);
    let clamped_height = phys_height
        .min(target_info.height.saturating_sub(clamped_y))
        .max(1);

    (clamped_x, clamped_y, clamped_width, clamped_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scissor_clamping() {
        let target = RenderTargetInfo {
            width: 800,
            height: 600,
        };

        // Test normal case
        let (x, y, w, h) =
            viewport_to_render_target_scissor(100.0, 100.0, 200.0, 200.0, [1.0, 1.0], target);
        assert_eq!((x, y, w, h), (100, 100, 200, 200));

        // Test overflow case
        let (x, y, w, h) =
            viewport_to_render_target_scissor(700.0, 500.0, 200.0, 200.0, [1.0, 1.0], target);
        assert_eq!((x, y, w, h), (700, 500, 100, 100));

        // Test DPI scaling
        let (x, y, w, h) =
            viewport_to_render_target_scissor(50.0, 50.0, 100.0, 100.0, [2.0, 2.0], target);
        assert_eq!((x, y, w, h), (100, 100, 200, 200));
    }
}
