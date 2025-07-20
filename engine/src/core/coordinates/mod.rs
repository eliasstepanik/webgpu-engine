//! Large world coordinate system
//!
//! Provides support for worlds beyond the precision limits of single-precision
//! floating point by implementing a dual coordinate system:
//! - f64 world coordinates for high-precision positioning
//! - f32 camera-relative coordinates for GPU rendering
//!
//! This enables games with planetary-scale worlds without precision artifacts.
//!
//! For galaxy-scale coordinates (>10^15 meters), use the hierarchical
//! coordinate system provided by the galaxy_coordinates module.

pub mod galaxy_coordinates;
pub mod origin_manager;
pub mod world_transform;

#[cfg(test)]
mod tests;

pub use galaxy_coordinates::{GalaxyCoordinateSystem, GalaxyPosition, GalaxySector};
pub use origin_manager::CoordinateSystem;
pub use world_transform::WorldTransform;

/// Configuration for large world coordinate systems
#[derive(Debug, Clone)]
pub struct LargeWorldConfig {
    /// Enable large world coordinate support
    pub enable_large_world: bool,
    /// Distance threshold for origin shifting (in world units)
    pub origin_shift_threshold: f64,
    /// Use logarithmic depth buffer for better z-precision
    pub use_logarithmic_depth: bool,
    /// Maximum rendering distance from camera
    pub max_render_distance: f64,
}

impl Default for LargeWorldConfig {
    fn default() -> Self {
        Self {
            enable_large_world: true,             // Always enabled
            origin_shift_threshold: 50_000.0,     // 50km
            use_logarithmic_depth: true,          // Always use logarithmic depth
            max_render_distance: 1_000_000_000.0, // 1 billion units
        }
    }
}
