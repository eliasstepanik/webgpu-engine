//! Dock zone detection for window edge snapping
//!
//! Provides functionality to detect when a panel is being dragged near
//! a window edge and should snap to it.

use super::{DockEdge, DockedState};
use tracing::debug;

/// Default threshold distance in pixels for edge snapping
pub const DEFAULT_DOCK_THRESHOLD: f32 = 20.0;

/// Represents a docking zone along one edge of the window
#[derive(Debug, Clone)]
pub struct DockZone {
    /// Which edge this zone represents
    pub edge: DockEdge,
    /// Distance from edge to trigger snapping (in pixels)
    pub threshold: f32,
}

impl DockZone {
    /// Create a new dock zone with the default threshold
    pub fn new(edge: DockEdge) -> Self {
        Self {
            edge,
            threshold: DEFAULT_DOCK_THRESHOLD,
        }
    }

    /// Create a new dock zone with a custom threshold
    pub fn with_threshold(edge: DockEdge, threshold: f32) -> Self {
        Self { edge, threshold }
    }

    /// Check if a panel position is within this dock zone
    pub fn check_snap(
        &self,
        panel_pos: (f32, f32),
        panel_size: (f32, f32),
        window_size: (f32, f32),
    ) -> Option<DockedState> {
        let (x, y) = panel_pos;
        let (width, height) = panel_size;
        let (win_width, win_height) = window_size;

        // Check distance from edge based on panel position
        let in_zone = match self.edge {
            DockEdge::Left => x < self.threshold,
            DockEdge::Right => x + width > win_width - self.threshold,
            DockEdge::Top => y < self.threshold,
            DockEdge::Bottom => y + height > win_height - self.threshold,
        };

        if in_zone {
            // Calculate offset along the edge (normalized 0.0-1.0)
            let offset = match self.edge {
                DockEdge::Left | DockEdge::Right => {
                    if win_height > height {
                        (y / (win_height - height)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                }
                DockEdge::Top | DockEdge::Bottom => {
                    if win_width > width {
                        (x / (win_width - width)).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                }
            };

            // Size perpendicular to edge
            let size = match self.edge {
                DockEdge::Left | DockEdge::Right => width,
                DockEdge::Top | DockEdge::Bottom => height,
            };

            debug!(
                edge = ?self.edge,
                offset = offset,
                size = size,
                "Panel in dock zone"
            );

            Some(DockedState::new(self.edge, offset, size))
        } else {
            None
        }
    }

    /// Get the visual guide position for this dock zone
    pub fn get_guide_rect(&self, window_size: (f32, f32)) -> (f32, f32, f32, f32) {
        let (win_width, win_height) = window_size;

        match self.edge {
            DockEdge::Left => (0.0, 0.0, self.threshold, win_height),
            DockEdge::Right => (win_width - self.threshold, 0.0, self.threshold, win_height),
            DockEdge::Top => (0.0, 0.0, win_width, self.threshold),
            DockEdge::Bottom => (0.0, win_height - self.threshold, win_width, self.threshold),
        }
    }
}

/// Check all dock zones and return the first one that the panel is in
pub fn check_dock_zones(
    panel_pos: (f32, f32),
    panel_size: (f32, f32),
    window_size: (f32, f32),
    threshold: Option<f32>,
) -> Option<DockedState> {
    let threshold = threshold.unwrap_or(DEFAULT_DOCK_THRESHOLD);

    // Check all four edges
    let zones = [
        DockZone::with_threshold(DockEdge::Left, threshold),
        DockZone::with_threshold(DockEdge::Right, threshold),
        DockZone::with_threshold(DockEdge::Top, threshold),
        DockZone::with_threshold(DockEdge::Bottom, threshold),
    ];

    // Return the first zone that matches
    for zone in &zones {
        if let Some(docked) = zone.check_snap(panel_pos, panel_size, window_size) {
            return Some(docked);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_left_edge_detection() {
        let zone = DockZone::new(DockEdge::Left);
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 300.0);

        // Panel near left edge
        let docked = zone.check_snap((10.0, 100.0), panel_size, window_size);
        assert!(docked.is_some());
        let docked = docked.unwrap();
        assert_eq!(docked.edge, DockEdge::Left);
        assert!((docked.offset - 0.333).abs() < 0.01); // 100 / (600-300)

        // Panel far from left edge
        let docked = zone.check_snap((50.0, 100.0), panel_size, window_size);
        assert!(docked.is_none());
    }

    #[test]
    fn test_right_edge_detection() {
        let zone = DockZone::new(DockEdge::Right);
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 300.0);

        // Panel near right edge
        let docked = zone.check_snap((590.0, 150.0), panel_size, window_size);
        assert!(docked.is_some());
        let docked = docked.unwrap();
        assert_eq!(docked.edge, DockEdge::Right);
        assert_eq!(docked.size, 200.0);

        // Panel far from right edge
        let docked = zone.check_snap((500.0, 150.0), panel_size, window_size);
        assert!(docked.is_none());
    }

    #[test]
    fn test_top_edge_detection() {
        let zone = DockZone::new(DockEdge::Top);
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 100.0);

        // Panel near top edge
        let docked = zone.check_snap((300.0, 15.0), panel_size, window_size);
        assert!(docked.is_some());
        let docked = docked.unwrap();
        assert_eq!(docked.edge, DockEdge::Top);
        assert!((docked.offset - 0.5).abs() < 0.01); // 300 / (800-200)
    }

    #[test]
    fn test_bottom_edge_detection() {
        let zone = DockZone::new(DockEdge::Bottom);
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 100.0);

        // Panel near bottom edge
        let docked = zone.check_snap((600.0, 485.0), panel_size, window_size);
        assert!(docked.is_some());
        let docked = docked.unwrap();
        assert_eq!(docked.edge, DockEdge::Bottom);
        assert_eq!(docked.offset, 1.0); // 600 / (800-200)
    }

    #[test]
    fn test_custom_threshold() {
        let zone = DockZone::with_threshold(DockEdge::Left, 50.0);
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 300.0);

        // Within custom threshold
        let docked = zone.check_snap((40.0, 100.0), panel_size, window_size);
        assert!(docked.is_some());

        // Outside default threshold but within custom
        let zone_default = DockZone::new(DockEdge::Left);
        let docked = zone_default.check_snap((40.0, 100.0), panel_size, window_size);
        assert!(docked.is_none());
    }

    #[test]
    fn test_check_all_zones() {
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 300.0);

        // Test left edge
        let docked = check_dock_zones((5.0, 150.0), panel_size, window_size, None);
        assert!(docked.is_some());
        assert_eq!(docked.unwrap().edge, DockEdge::Left);

        // Test right edge
        let docked = check_dock_zones((595.0, 150.0), panel_size, window_size, None);
        assert!(docked.is_some());
        assert_eq!(docked.unwrap().edge, DockEdge::Right);

        // Test no edge
        let docked = check_dock_zones((300.0, 200.0), panel_size, window_size, None);
        assert!(docked.is_none());
    }
}
