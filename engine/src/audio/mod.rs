//! Audio system for 3D spatial sound with occlusion and environmental effects
//!
//! This module provides a comprehensive audio system with:
//! - 3D spatial audio with distance attenuation
//! - Ray-based sound occlusion
//! - Material-based sound absorption
//! - Dynamic reverb and environmental acoustics
//! - Doppler effect for moving sources

pub mod ambient;
pub mod components;
pub mod engine;
pub mod listener;
pub mod material;
pub mod propagation;
pub mod raycast;
pub mod resources;
pub mod reverb;
pub mod source;
pub mod system;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use components::{AmbientSound, AudioListener, AudioMaterial, AudioSource};
pub use engine::{AudioEngine, AudioHandle};
pub use raycast::{audio_raycast, AudioRay, AudioRayHit};
pub use resources::AudioAsset;
pub use reverb::ReverbZone;
pub use system::audio_update_system;

/// Maximum distance for room acoustics estimation
pub const MAX_ROOM_SIZE: f32 = 100.0;
