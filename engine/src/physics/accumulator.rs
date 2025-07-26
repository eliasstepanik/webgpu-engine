//! Thread-safe physics accumulator for fixed timestep physics updates
//!
//! This module provides a thread-safe accumulator that handles partial physics updates
//! and interpolation between fixed timesteps for smooth rendering.

use std::sync::RwLock;
use tracing::warn;

/// Thread-safe physics accumulator for managing fixed timestep updates
#[derive(Debug)]
pub struct PhysicsAccumulator {
    /// Accumulated time since last physics update
    accumulator: RwLock<f32>,
    /// Fixed timestep for physics updates
    pub fixed_timestep: f32,
}

impl PhysicsAccumulator {
    /// Create a new physics accumulator with the given fixed timestep
    pub fn new(fixed_timestep: f32) -> Self {
        Self {
            accumulator: RwLock::new(0.0),
            fixed_timestep,
        }
    }

    /// Add delta time to the accumulator
    /// Returns the number of physics steps to perform
    pub fn accumulate(&self, delta_time: f32) -> u32 {
        let mut acc = self
            .accumulator
            .write()
            .expect("Failed to lock physics accumulator for write");

        *acc += delta_time;

        // Safety check: prevent spiral of death
        if *acc > self.fixed_timestep * 8.0 {
            warn!(
                "Physics accumulator too large: {} seconds. Clamping to prevent spiral of death.",
                *acc
            );
            *acc = self.fixed_timestep * 8.0;
        }

        // Calculate number of steps
        let steps = (*acc / self.fixed_timestep) as u32;
        *acc -= steps as f32 * self.fixed_timestep;

        steps
    }

    /// Get the interpolation alpha value for rendering
    /// Alpha is in range [0, 1] representing how far between physics frames we are
    pub fn get_interpolation_alpha(&self) -> f32 {
        let acc = self
            .accumulator
            .read()
            .expect("Failed to lock physics accumulator for read");
        *acc / self.fixed_timestep
    }

    /// Reset the accumulator to zero
    pub fn reset(&self) {
        let mut acc = self
            .accumulator
            .write()
            .expect("Failed to lock physics accumulator for write");
        *acc = 0.0;
    }

    /// Get the current accumulated time
    pub fn get_accumulated_time(&self) -> f32 {
        let acc = self
            .accumulator
            .read()
            .expect("Failed to lock physics accumulator for read");
        *acc
    }
}

impl Default for PhysicsAccumulator {
    fn default() -> Self {
        // Default to 30Hz physics updates
        Self::new(1.0 / 30.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator_basic() {
        let acc = PhysicsAccumulator::new(1.0 / 60.0); // 60Hz physics

        // Test accumulation
        let steps = acc.accumulate(1.0 / 30.0); // 2 frames worth
        assert_eq!(steps, 2);
        assert!((acc.get_interpolation_alpha() - 0.0).abs() < 0.001);

        // Test partial accumulation
        let steps = acc.accumulate(1.0 / 120.0); // Half a frame
        assert_eq!(steps, 0);
        assert!((acc.get_interpolation_alpha() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_accumulator_spiral_of_death() {
        let acc = PhysicsAccumulator::new(1.0 / 60.0);

        // Large delta time should be clamped
        let steps = acc.accumulate(1.0); // 1 second
        assert!(steps <= 8); // Should be clamped to max 8 steps
    }

    #[test]
    fn test_reset() {
        let acc = PhysicsAccumulator::new(1.0 / 60.0);
        acc.accumulate(1.0 / 120.0);
        acc.reset();
        assert_eq!(acc.get_accumulated_time(), 0.0);
    }
}
