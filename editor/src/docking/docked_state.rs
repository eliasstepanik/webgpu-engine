//! Docked panel state management
//!
//! Defines the state of a docked panel including which edge it's docked to,
//! its position along that edge, and its size.

use serde::{Deserialize, Serialize};

/// Represents which edge of the main window a panel is docked to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DockEdge {
    /// Docked to the left edge of the window
    Left,
    /// Docked to the right edge of the window
    Right,
    /// Docked to the top edge of the window
    Top,
    /// Docked to the bottom edge of the window
    Bottom,
}

/// State of a docked panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockedState {
    /// Which edge the panel is docked to
    pub edge: DockEdge,
    /// Offset along the edge (0.0-1.0)
    /// For Left/Right: 0.0 = top, 1.0 = bottom
    /// For Top/Bottom: 0.0 = left, 1.0 = right
    pub offset: f32,
    /// Size perpendicular to the edge in pixels
    /// For Left/Right: width of the panel
    /// For Top/Bottom: height of the panel
    pub size: f32,
}

impl DockedState {
    /// Create a new docked state
    pub fn new(edge: DockEdge, offset: f32, size: f32) -> Self {
        Self {
            edge,
            offset: offset.clamp(0.0, 1.0),
            size: size.max(50.0), // Minimum size
        }
    }

    /// Calculate the absolute position for a docked panel given the window size
    pub fn calculate_position(
        &self,
        panel_size: (f32, f32),
        window_size: (f32, f32),
    ) -> (f32, f32) {
        match self.edge {
            DockEdge::Left => {
                let y = (window_size.1 - panel_size.1) * self.offset;
                (0.0, y)
            }
            DockEdge::Right => {
                let x = window_size.0 - panel_size.0;
                let y = (window_size.1 - panel_size.1) * self.offset;
                (x, y)
            }
            DockEdge::Top => {
                let x = (window_size.0 - panel_size.0) * self.offset;
                (x, 0.0)
            }
            DockEdge::Bottom => {
                let x = (window_size.0 - panel_size.0) * self.offset;
                let y = window_size.1 - panel_size.1;
                (x, y)
            }
        }
    }

    /// Update the offset based on a new position
    pub fn update_offset(
        &mut self,
        position: (f32, f32),
        panel_size: (f32, f32),
        window_size: (f32, f32),
    ) {
        self.offset = match self.edge {
            DockEdge::Left | DockEdge::Right => {
                if window_size.1 > panel_size.1 {
                    (position.1 / (window_size.1 - panel_size.1)).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            DockEdge::Top | DockEdge::Bottom => {
                if window_size.0 > panel_size.0 {
                    (position.0 / (window_size.0 - panel_size.0)).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docked_state_creation() {
        let state = DockedState::new(DockEdge::Left, 0.5, 200.0);
        assert_eq!(state.edge, DockEdge::Left);
        assert_eq!(state.offset, 0.5);
        assert_eq!(state.size, 200.0);
    }

    #[test]
    fn test_offset_clamping() {
        let state = DockedState::new(DockEdge::Top, 1.5, 150.0);
        assert_eq!(state.offset, 1.0); // Clamped to 1.0

        let state = DockedState::new(DockEdge::Bottom, -0.5, 150.0);
        assert_eq!(state.offset, 0.0); // Clamped to 0.0
    }

    #[test]
    fn test_minimum_size() {
        let state = DockedState::new(DockEdge::Right, 0.3, 25.0);
        assert_eq!(state.size, 50.0); // Enforced minimum
    }

    #[test]
    fn test_position_calculation() {
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 300.0);

        // Left edge, centered
        let state = DockedState::new(DockEdge::Left, 0.5, 200.0);
        let pos = state.calculate_position(panel_size, window_size);
        assert_eq!(pos, (0.0, 150.0)); // (0, (600-300)*0.5)

        // Right edge, at top
        let state = DockedState::new(DockEdge::Right, 0.0, 200.0);
        let pos = state.calculate_position(panel_size, window_size);
        assert_eq!(pos, (600.0, 0.0)); // (800-200, 0)

        // Top edge, centered
        let state = DockedState::new(DockEdge::Top, 0.5, 100.0);
        let pos = state.calculate_position(panel_size, window_size);
        assert_eq!(pos, (300.0, 0.0)); // ((800-200)*0.5, 0)

        // Bottom edge, at right
        let state = DockedState::new(DockEdge::Bottom, 1.0, 100.0);
        let pos = state.calculate_position(panel_size, window_size);
        assert_eq!(pos, (600.0, 300.0)); // (800-200, 600-300)
    }

    #[test]
    fn test_update_offset() {
        let window_size = (800.0, 600.0);
        let panel_size = (200.0, 300.0);

        // Test left edge
        let mut state = DockedState::new(DockEdge::Left, 0.0, 200.0);
        state.update_offset((0.0, 150.0), panel_size, window_size);
        assert!((state.offset - 0.5).abs() < 0.01); // Should be 0.5

        // Test top edge
        let mut state = DockedState::new(DockEdge::Top, 0.0, 100.0);
        state.update_offset((300.0, 0.0), panel_size, window_size);
        assert!((state.offset - 0.5).abs() < 0.01); // Should be 0.5
    }
}
