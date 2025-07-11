//! Temporary workarounds for viewport issues
//!
//! This module contains quick fixes and workarounds for the viewport system
//! issues until proper fixes can be implemented.

use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, warn};

/// Global flag for verbose viewport debugging
static VIEWPORT_DEBUG: Lazy<AtomicBool> =
    Lazy::new(|| AtomicBool::new(std::env::var("VIEWPORT_DEBUG").is_ok()));

/// Check if viewport debug mode is enabled
pub fn is_debug_enabled() -> bool {
    VIEWPORT_DEBUG.load(Ordering::Relaxed)
}

/// Log a viewport debug message (only if VIEWPORT_DEBUG is set)
macro_rules! viewport_debug {
    ($($arg:tt)*) => {
        if is_debug_enabled() {
            tracing::debug!($($arg)*);
        }
    };
}

// Log a viewport trace message (only if VIEWPORT_DEBUG is set)
// Currently unused but may be useful for future debugging
#[allow(unused_macros)]
macro_rules! viewport_trace {
    ($($arg:tt)*) => {
        if is_debug_enabled() {
            tracing::trace!($($arg)*);
        }
    };
}

/// Check if we should skip rendering a viewport due to invalid size
pub fn should_skip_viewport_render(
    viewport_id: imgui::Id,
    surface_width: u32,
    surface_height: u32,
    draw_data: &imgui::DrawData,
) -> bool {
    if surface_width == 0 || surface_height == 0 {
        viewport_debug!(
            "Skipping viewport {:?} render: zero-sized surface",
            viewport_id
        );
        return true;
    }

    let display_size = draw_data.display_size;
    let fb_scale = draw_data.framebuffer_scale;

    let fb_width = (display_size[0] * fb_scale[0]).round() as u32;
    let fb_height = (display_size[1] * fb_scale[1]).round() as u32;

    if fb_width > surface_width || fb_height > surface_height {
        warn!(
            "Viewport {:?} framebuffer size ({}x{}) exceeds surface size ({}x{})",
            viewport_id, fb_width, fb_height, surface_width, surface_height
        );
        return true;
    }

    if fb_width == 0 || fb_height == 0 {
        viewport_debug!(
            "Skipping viewport {:?} render: zero-sized framebuffer",
            viewport_id
        );
        return true;
    }

    false
}

/// Adjust draw data to fit within surface bounds
pub fn clamp_draw_data_to_surface(
    draw_data: &mut imgui::DrawData,
    surface_width: u32,
    surface_height: u32,
) {
    let fb_scale = draw_data.framebuffer_scale;

    // Calculate maximum logical size that fits in the surface
    let max_logical_width = surface_width as f32 / fb_scale[0];
    let max_logical_height = surface_height as f32 / fb_scale[1];

    // Clamp display size
    if draw_data.display_size[0] > max_logical_width {
        warn!(
            "Clamping display width from {} to {}",
            draw_data.display_size[0], max_logical_width
        );
        draw_data.display_size[0] = max_logical_width;
    }

    if draw_data.display_size[1] > max_logical_height {
        warn!(
            "Clamping display height from {} to {}",
            draw_data.display_size[1], max_logical_height
        );
        draw_data.display_size[1] = max_logical_height;
    }
}

/// Validate and log viewport state for debugging
pub fn debug_viewport_state(
    viewport: &imgui::Viewport,
    window_size: Option<(u32, u32)>,
    surface_size: Option<(u32, u32)>,
) {
    if !is_debug_enabled() {
        return;
    }

    viewport_debug!(
        "Viewport {:?} state: pos={:?}, size={:?}, work_pos={:?}, work_size={:?}",
        viewport.id,
        viewport.pos,
        viewport.size,
        viewport.work_pos,
        viewport.work_size
    );

    if let Some((w, h)) = window_size {
        viewport_debug!("  Window size: {}x{}", w, h);
    }

    if let Some((w, h)) = surface_size {
        viewport_debug!("  Surface size: {}x{}", w, h);
    }

    if let Some(draw_data) = viewport.draw_data() {
        let fb_width = (draw_data.display_size[0] * draw_data.framebuffer_scale[0]).round() as u32;
        let fb_height = (draw_data.display_size[1] * draw_data.framebuffer_scale[1]).round() as u32;

        viewport_debug!(
            "  Draw data: display_size={:?}, fb_scale={:?}, fb_size={}x{}",
            draw_data.display_size,
            draw_data.framebuffer_scale,
            fb_width,
            fb_height
        );
    }
}

/// Emergency fix: Force single-window mode if viewports are broken
pub fn force_single_window_mode() -> bool {
    // Check environment variable for emergency override
    std::env::var("FORCE_SINGLE_WINDOW").is_ok()
}

/// Initialize workarounds module
pub fn init() {
    if force_single_window_mode() {
        warn!("FORCE_SINGLE_WINDOW mode enabled - viewport system disabled");
    }

    if is_debug_enabled() {
        debug!("VIEWPORT_DEBUG mode enabled - verbose viewport logging active");
    }
}
