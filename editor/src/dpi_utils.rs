//! DPI scaling utilities for viewport handling
//!
//! Provides utilities for handling DPI scaling in multi-viewport scenarios.

use tracing::debug;

/// Information about DPI scaling
#[derive(Debug, Clone, Copy)]
pub struct DpiInfo {
    /// The display scale factor (e.g., 2.0 for 200% scaling)
    pub scale_factor: f32,
    /// Logical size (UI units)
    pub logical_size: [f32; 2],
    /// Physical size (pixels)
    pub physical_size: [u32; 2],
}

impl DpiInfo {
    /// Create DPI info from window physical size and scale factor
    pub fn from_window(physical_width: u32, physical_height: u32, scale_factor: f64) -> Self {
        let scale_factor = scale_factor as f32;
        let logical_size = [
            physical_width as f32 / scale_factor,
            physical_height as f32 / scale_factor,
        ];

        debug!(
            "DPI Info: physical={}x{}, scale={}, logical={:.1}x{:.1}",
            physical_width, physical_height, scale_factor, logical_size[0], logical_size[1]
        );

        Self {
            scale_factor,
            logical_size,
            physical_size: [physical_width, physical_height],
        }
    }

    /// Convert logical position to physical position
    pub fn logical_to_physical_pos(&self, logical_pos: [f32; 2]) -> [f32; 2] {
        [
            logical_pos[0] * self.scale_factor,
            logical_pos[1] * self.scale_factor,
        ]
    }

    /// Convert physical position to logical position
    pub fn physical_to_logical_pos(&self, physical_pos: [f32; 2]) -> [f32; 2] {
        [
            physical_pos[0] / self.scale_factor,
            physical_pos[1] / self.scale_factor,
        ]
    }

    /// Convert logical size to physical size
    pub fn logical_to_physical_size(&self, logical_size: [f32; 2]) -> [u32; 2] {
        [
            (logical_size[0] * self.scale_factor).round() as u32,
            (logical_size[1] * self.scale_factor).round() as u32,
        ]
    }

    /// Convert physical size to logical size
    pub fn physical_to_logical_size(&self, physical_size: [u32; 2]) -> [f32; 2] {
        [
            physical_size[0] as f32 / self.scale_factor,
            physical_size[1] as f32 / self.scale_factor,
        ]
    }
}

/// Ensure ImGui display size matches physical size through scale factor
pub fn validate_imgui_dpi_scaling(
    display_size: [f32; 2],
    framebuffer_scale: [f32; 2],
    physical_size: [u32; 2],
) -> bool {
    let expected_physical = [
        (display_size[0] * framebuffer_scale[0]).round() as u32,
        (display_size[1] * framebuffer_scale[1]).round() as u32,
    ];

    let matches =
        expected_physical[0] == physical_size[0] && expected_physical[1] == physical_size[1];

    if !matches {
        debug!(
            "DPI scaling mismatch: display={:?}, scale={:?}, expected_physical={:?}, actual_physical={:?}",
            display_size, framebuffer_scale, expected_physical, physical_size
        );
    }

    matches
}

/// Apply DPI-aware position adjustment for viewport windows
pub fn adjust_viewport_position_for_dpi(
    pos: [f32; 2],
    source_dpi: f32,
    target_dpi: f32,
) -> [f32; 2] {
    if (source_dpi - target_dpi).abs() < 0.01 {
        return pos;
    }

    let scale = target_dpi / source_dpi;
    [pos[0] * scale, pos[1] * scale]
}

/// Calculate the appropriate window size for a viewport considering DPI
pub fn calculate_viewport_window_size(
    requested_logical_size: [f32; 2],
    dpi_scale: f32,
    min_size: Option<[u32; 2]>,
    max_size: Option<[u32; 2]>,
) -> [u32; 2] {
    let mut physical_size = [
        (requested_logical_size[0] * dpi_scale).round() as u32,
        (requested_logical_size[1] * dpi_scale).round() as u32,
    ];

    // Apply min size constraints
    if let Some(min) = min_size {
        physical_size[0] = physical_size[0].max(min[0]);
        physical_size[1] = physical_size[1].max(min[1]);
    }

    // Apply max size constraints
    if let Some(max) = max_size {
        physical_size[0] = physical_size[0].min(max[0]);
        physical_size[1] = physical_size[1].min(max[1]);
    }

    physical_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpi_info_conversions() {
        let dpi = DpiInfo::from_window(3840, 2160, 2.0);

        // Test basic properties
        assert_eq!(dpi.scale_factor, 2.0);
        assert_eq!(dpi.physical_size, [3840, 2160]);
        assert_eq!(dpi.logical_size, [1920.0, 1080.0]);

        // Test position conversions
        let logical_pos = [100.0, 200.0];
        let physical_pos = dpi.logical_to_physical_pos(logical_pos);
        assert_eq!(physical_pos, [200.0, 400.0]);
        assert_eq!(dpi.physical_to_logical_pos(physical_pos), logical_pos);

        // Test size conversions
        let logical_size = [800.0, 600.0];
        let physical_size = dpi.logical_to_physical_size(logical_size);
        assert_eq!(physical_size, [1600, 1200]);
        assert_eq!(dpi.physical_to_logical_size(physical_size), logical_size);
    }

    #[test]
    fn test_validate_imgui_dpi_scaling() {
        // Valid scaling
        assert!(validate_imgui_dpi_scaling(
            [1920.0, 1080.0],
            [2.0, 2.0],
            [3840, 2160]
        ));

        // Invalid scaling
        assert!(!validate_imgui_dpi_scaling(
            [1920.0, 1080.0],
            [1.0, 1.0],
            [3840, 2160]
        ));
    }

    #[test]
    fn test_viewport_size_calculation() {
        // Normal calculation
        let size = calculate_viewport_window_size([800.0, 600.0], 2.0, None, None);
        assert_eq!(size, [1600, 1200]);

        // With min size constraint
        let size = calculate_viewport_window_size([100.0, 100.0], 2.0, Some([400, 300]), None);
        assert_eq!(size, [400, 300]);

        // With max size constraint
        let size = calculate_viewport_window_size([2000.0, 2000.0], 2.0, None, Some([1920, 1080]));
        assert_eq!(size, [1920, 1080]);
    }
}
