//! Origin management for large world coordinate systems
//!
//! Provides origin shifting functionality to maintain precision by keeping
//! the camera near the coordinate system origin in single-player scenarios.

use glam::DVec3;
use tracing::{debug, info, warn};

/// Manages coordinate system origin for large world support
///
/// In single-player games, this can shift the world origin to keep the camera
/// near (0,0,0) for optimal floating-point precision. For multiplayer games,
/// origin shifting is disabled as the server needs precision for all players.
#[derive(Debug, Clone)]
pub struct CoordinateSystem {
    /// Current camera position in world coordinates
    pub camera_origin: DVec3,
    /// Distance threshold for triggering origin shifts
    pub origin_threshold: f64,
    /// Whether origin shifting is enabled (disable for multiplayer)
    pub enable_origin_shift: bool,
    /// Total offset from the original world origin
    pub total_origin_offset: DVec3,
    /// History of origin shifts for debugging
    origin_shift_history: Vec<OriginShift>,
}

/// Record of an origin shift operation
#[derive(Debug, Clone)]
struct OriginShift {
    /// World time when the shift occurred
    timestamp: std::time::Instant,
    /// Previous origin position
    old_origin: DVec3,
    /// New origin position
    new_origin: DVec3,
    /// Camera position that triggered the shift
    trigger_camera_pos: DVec3,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl CoordinateSystem {
    /// Create a new coordinate system manager
    pub fn new() -> Self {
        Self {
            camera_origin: DVec3::ZERO,
            origin_threshold: 50_000.0, // 50km default
            enable_origin_shift: false, // Disabled by default for safety
            total_origin_offset: DVec3::ZERO,
            origin_shift_history: Vec::new(),
        }
    }

    /// Create a coordinate system with specific settings
    pub fn with_config(origin_threshold: f64, enable_origin_shift: bool) -> Self {
        Self {
            camera_origin: DVec3::ZERO,
            origin_threshold,
            enable_origin_shift,
            total_origin_offset: DVec3::ZERO,
            origin_shift_history: Vec::new(),
        }
    }

    /// Update the camera position and potentially shift origin
    ///
    /// Returns true if an origin shift occurred, which requires updating
    /// all world transforms in the game.
    pub fn update_camera_origin(&mut self, new_camera_world_pos: DVec3) -> bool {
        let old_camera_origin = self.camera_origin;
        self.camera_origin = new_camera_world_pos;

        if self.enable_origin_shift && self.should_shift_origin() {
            self.perform_origin_shift(new_camera_world_pos);
            true
        } else {
            false
        }
    }

    /// Check if origin should be shifted based on camera distance
    pub fn should_shift_origin(&self) -> bool {
        if !self.enable_origin_shift {
            return false;
        }

        let distance_from_origin = self.camera_origin.length();
        distance_from_origin > self.origin_threshold
    }

    /// Get the camera position relative to the current origin
    pub fn get_camera_relative_position(&self) -> DVec3 {
        // In origin shifting mode, camera should always be near origin
        if self.enable_origin_shift {
            DVec3::ZERO
        } else {
            self.camera_origin
        }
    }

    /// Convert a world position to the current coordinate system
    pub fn world_to_current(&self, world_pos: DVec3) -> DVec3 {
        world_pos - self.total_origin_offset
    }

    /// Convert a current coordinate system position to world coordinates
    pub fn current_to_world(&self, current_pos: DVec3) -> DVec3 {
        current_pos + self.total_origin_offset
    }

    /// Get the total offset from the original world origin
    pub fn get_total_origin_offset(&self) -> DVec3 {
        self.total_origin_offset
    }

    /// Get information about recent origin shifts
    pub fn get_origin_shift_history(&self) -> &[OriginShift] {
        &self.origin_shift_history
    }

    /// Clear the origin shift history
    pub fn clear_history(&mut self) {
        self.origin_shift_history.clear();
    }

    /// Enable or disable origin shifting
    ///
    /// Warning: Disabling origin shifting during gameplay can cause precision
    /// issues if entities are far from origin.
    pub fn set_origin_shift_enabled(&mut self, enabled: bool) {
        if !enabled && self.enable_origin_shift {
            warn!(
                "Disabling origin shifting with total offset: {:?}",
                self.total_origin_offset
            );
        }
        self.enable_origin_shift = enabled;
    }

    /// Set the distance threshold for origin shifting
    pub fn set_origin_threshold(&mut self, threshold: f64) {
        debug!("Setting origin shift threshold to: {}", threshold);
        self.origin_threshold = threshold;
    }

    /// Perform the actual origin shift
    fn perform_origin_shift(&mut self, trigger_camera_pos: DVec3) {
        let shift_amount = self.camera_origin;
        let old_origin = DVec3::ZERO;
        let new_origin = -shift_amount;

        // Record the shift for history
        let shift_record = OriginShift {
            timestamp: std::time::Instant::now(),
            old_origin,
            new_origin,
            trigger_camera_pos,
        };

        self.origin_shift_history.push(shift_record);

        // Limit history size to prevent memory growth
        if self.origin_shift_history.len() > 100 {
            self.origin_shift_history.drain(..50);
        }

        // Update the total offset
        self.total_origin_offset += shift_amount;

        // Reset camera to origin
        self.camera_origin = DVec3::ZERO;

        info!(
            "Origin shift performed: offset={:?}, total_offset={:?}",
            shift_amount, self.total_origin_offset
        );
    }

    /// Get statistics about the coordinate system
    pub fn get_stats(&self) -> CoordinateSystemStats {
        CoordinateSystemStats {
            camera_world_position: self.camera_origin + self.total_origin_offset,
            camera_relative_position: self.camera_origin,
            total_origin_offset: self.total_origin_offset,
            origin_shifts_performed: self.origin_shift_history.len(),
            origin_shift_enabled: self.enable_origin_shift,
            origin_threshold: self.origin_threshold,
        }
    }
}

/// Statistics about the coordinate system state
#[derive(Debug, Clone)]
pub struct CoordinateSystemStats {
    /// Current camera position in absolute world coordinates
    pub camera_world_position: DVec3,
    /// Camera position relative to current origin
    pub camera_relative_position: DVec3,
    /// Total offset from original world origin
    pub total_origin_offset: DVec3,
    /// Number of origin shifts performed
    pub origin_shifts_performed: usize,
    /// Whether origin shifting is enabled
    pub origin_shift_enabled: bool,
    /// Current origin shift threshold
    pub origin_threshold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::EPSILON;

    #[test]
    fn test_coordinate_system_default() {
        let coord_system = CoordinateSystem::new();
        assert_eq!(coord_system.camera_origin, DVec3::ZERO);
        assert!(!coord_system.enable_origin_shift);
        assert_eq!(coord_system.origin_threshold, 50_000.0);
    }

    #[test]
    fn test_should_shift_origin_disabled() {
        let mut coord_system = CoordinateSystem::new();
        coord_system.camera_origin = DVec3::new(100_000.0, 0.0, 0.0);

        // Should not shift when disabled
        assert!(!coord_system.should_shift_origin());
    }

    #[test]
    fn test_should_shift_origin_enabled() {
        let mut coord_system = CoordinateSystem::new();
        coord_system.enable_origin_shift = true;
        coord_system.origin_threshold = 10_000.0;

        // Should not shift when within threshold
        coord_system.camera_origin = DVec3::new(5_000.0, 0.0, 0.0);
        assert!(!coord_system.should_shift_origin());

        // Should shift when beyond threshold
        coord_system.camera_origin = DVec3::new(15_000.0, 0.0, 0.0);
        assert!(coord_system.should_shift_origin());
    }

    #[test]
    fn test_origin_shift_updates_total_offset() {
        let mut coord_system = CoordinateSystem::with_config(10_000.0, true);

        let camera_pos = DVec3::new(15_000.0, 5_000.0, 0.0);
        let shifted = coord_system.update_camera_origin(camera_pos);

        assert!(shifted);
        assert_eq!(coord_system.camera_origin, DVec3::ZERO);
        assert_eq!(coord_system.total_origin_offset, camera_pos);
    }

    #[test]
    fn test_world_to_current_conversion() {
        let mut coord_system = CoordinateSystem::with_config(10_000.0, true);

        // Perform an origin shift
        let camera_pos = DVec3::new(15_000.0, 0.0, 0.0);
        coord_system.update_camera_origin(camera_pos);

        // Test conversion
        let world_pos = DVec3::new(20_000.0, 100.0, 200.0);
        let current_pos = coord_system.world_to_current(world_pos);
        let back_to_world = coord_system.current_to_world(current_pos);

        assert!((back_to_world - world_pos).length() < EPSILON);
    }

    #[test]
    fn test_origin_shift_history() {
        let mut coord_system = CoordinateSystem::with_config(10_000.0, true);

        // Perform multiple shifts
        coord_system.update_camera_origin(DVec3::new(15_000.0, 0.0, 0.0));
        coord_system.update_camera_origin(DVec3::new(12_000.0, 0.0, 0.0));

        let history = coord_system.get_origin_shift_history();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut coord_system = CoordinateSystem::with_config(10_000.0, true);
        coord_system.update_camera_origin(DVec3::new(15_000.0, 5_000.0, 0.0));

        let stats = coord_system.get_stats();
        assert_eq!(
            stats.camera_world_position,
            DVec3::new(15_000.0, 5_000.0, 0.0)
        );
        assert_eq!(stats.camera_relative_position, DVec3::ZERO);
        assert_eq!(stats.origin_shifts_performed, 1);
        assert!(stats.origin_shift_enabled);
    }
}
