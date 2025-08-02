//! Environmental reverb and room acoustics

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::ComponentRegistry;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Reverb zone component for environmental acoustics
#[derive(
    engine_derive::Component, engine_derive::EditorUI, Debug, Clone, Serialize, Deserialize,
)]
pub struct ReverbZone {
    /// Zone radius
    #[ui(range = 1.0..100.0, speed = 0.5, tooltip = "Radius of the reverb zone")]
    pub radius: f32,

    /// Reverb preset
    pub preset: ReverbPreset,

    /// Mix amount (0.0 = dry, 1.0 = full reverb)
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Reverb wet/dry mix")]
    pub mix: f32,

    /// Zone priority (higher priority zones override lower ones)
    #[ui(range = 0.0..10.0, speed = 1.0, tooltip = "Zone priority for overlaps")]
    pub priority: i32,
}

/// Reverb presets for different environments
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReverbPreset {
    /// Small room reverb
    Room,
    /// Large hall reverb
    Hall,
    /// Cave/cavern reverb
    Cave,
    /// Cathedral reverb
    Cathedral,
    /// Outdoor space
    Outdoor,
    /// Custom reverb parameters
    Custom {
        decay_time: f32,
        damping: f32,
        room_size: f32,
    },
}

impl Default for ReverbZone {
    fn default() -> Self {
        Self {
            radius: 10.0,
            preset: ReverbPreset::Room,
            mix: 0.3,
            priority: 0,
        }
    }
}

/// Calculate reverb parameters based on room dimensions
pub fn calculate_room_reverb(room_dimensions: Vec3) -> ReverbParameters {
    let volume = room_dimensions.x * room_dimensions.y * room_dimensions.z;
    let surface_area = 2.0
        * (room_dimensions.x * room_dimensions.y
            + room_dimensions.x * room_dimensions.z
            + room_dimensions.y * room_dimensions.z);

    // Sabine's equation approximation
    let avg_dimension = (room_dimensions.x + room_dimensions.y + room_dimensions.z) / 3.0;
    let decay_time = 0.161 * volume / surface_area;

    ReverbParameters {
        decay_time: decay_time.clamp(0.1, 10.0),
        damping: (1.0 - avg_dimension / 100.0).clamp(0.0, 1.0),
        room_size: (volume / 1000.0).clamp(0.0, 1.0),
        early_reflections: true,
        late_reflections: true,
    }
}

/// Reverb parameters for audio processing
#[derive(Debug, Clone, Copy)]
pub struct ReverbParameters {
    /// RT60 decay time in seconds
    pub decay_time: f32,
    /// High frequency damping (0.0 = bright, 1.0 = dark)
    pub damping: f32,
    /// Room size factor (0.0 = small, 1.0 = large)
    pub room_size: f32,
    /// Enable early reflections
    pub early_reflections: bool,
    /// Enable late reflections
    pub late_reflections: bool,
}

impl ReverbPreset {
    /// Convert preset to reverb parameters
    pub fn to_parameters(&self) -> ReverbParameters {
        match self {
            Self::Room => ReverbParameters {
                decay_time: 0.4,
                damping: 0.5,
                room_size: 0.3,
                early_reflections: true,
                late_reflections: true,
            },
            Self::Hall => ReverbParameters {
                decay_time: 2.0,
                damping: 0.3,
                room_size: 0.8,
                early_reflections: true,
                late_reflections: true,
            },
            Self::Cave => ReverbParameters {
                decay_time: 5.0,
                damping: 0.2,
                room_size: 1.0,
                early_reflections: true,
                late_reflections: true,
            },
            Self::Cathedral => ReverbParameters {
                decay_time: 8.0,
                damping: 0.4,
                room_size: 1.0,
                early_reflections: true,
                late_reflections: true,
            },
            Self::Outdoor => ReverbParameters {
                decay_time: 0.2,
                damping: 0.9,
                room_size: 0.1,
                early_reflections: false,
                late_reflections: false,
            },
            Self::Custom {
                decay_time,
                damping,
                room_size,
            } => ReverbParameters {
                decay_time: *decay_time,
                damping: *damping,
                room_size: *room_size,
                early_reflections: true,
                late_reflections: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_reverb_calculation() {
        let small_room = Vec3::new(4.0, 3.0, 3.0);
        let params = calculate_room_reverb(small_room);
        assert!(params.decay_time < 1.0);

        let large_hall = Vec3::new(30.0, 10.0, 20.0);
        let params = calculate_room_reverb(large_hall);
        assert!(params.decay_time > 1.0);
    }

    #[test]
    fn test_reverb_presets() {
        let room = ReverbPreset::Room.to_parameters();
        assert!(room.decay_time < 1.0);

        let cathedral = ReverbPreset::Cathedral.to_parameters();
        assert!(cathedral.decay_time > 5.0);
    }
}
