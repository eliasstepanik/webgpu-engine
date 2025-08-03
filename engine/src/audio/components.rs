//! Audio components for the ECS

use crate::audio::AudioHandle;
use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::ComponentRegistry;
use serde::{Deserialize, Serialize};

/// Audio source component for entities that emit sound
#[derive(
    Debug, Clone, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AudioSource")]
#[serde(default)]
pub struct AudioSource {
    /// Handle to the loaded sound
    #[serde(skip)]
    #[ui(hidden)]
    pub sound: Option<AudioHandle>,

    /// Path to the audio file (for serialization)
    #[ui(tooltip = "Path to the audio file")]
    pub sound_path: String,

    /// Volume (0.0 to 1.0)
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Sound volume")]
    pub volume: f32,

    /// Pitch multiplier (1.0 = normal pitch)
    #[ui(range = 0.1..4.0, speed = 0.01, tooltip = "Pitch multiplier")]
    pub pitch: f32,

    /// Whether the sound should loop
    #[ui(tooltip = "Loop the sound")]
    pub looping: bool,

    /// Whether this is a 3D spatial sound
    #[ui(tooltip = "Enable 3D spatial audio")]
    pub spatial: bool,

    /// Maximum distance for sound attenuation
    #[ui(range = 1.0..1000.0, speed = 1.0, tooltip = "Maximum hearing distance")]
    pub max_distance: f32,

    /// Rolloff factor for distance attenuation
    #[ui(range = 0.1..10.0, speed = 0.1, tooltip = "How quickly volume decreases with distance")]
    pub rolloff_factor: f32,

    /// Whether the sound should play automatically
    #[ui(tooltip = "Start playing automatically")]
    pub auto_play: bool,

    /// Whether the sound is currently playing
    #[serde(skip)]
    #[ui(readonly)]
    pub is_playing: bool,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            sound: None,
            sound_path: String::new(),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial: true,
            max_distance: 50.0,
            rolloff_factor: 1.0,
            auto_play: false,
            is_playing: false,
        }
    }
}

/// Audio listener component (typically attached to camera)
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AudioListener")]
#[serde(default)]
pub struct AudioListener {
    /// Whether this listener is active
    #[ui(tooltip = "Enable this audio listener")]
    pub active: bool,

    /// Master volume for this listener
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Master volume")]
    pub master_volume: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            active: true,
            master_volume: 1.0,
        }
    }
}

/// Ambient sound component for non-positional audio
#[derive(
    Debug, Clone, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AmbientSound")]
#[serde(default)]
pub struct AmbientSound {
    /// Handle to the loaded sound
    #[serde(skip)]
    #[ui(hidden)]
    pub sound: Option<AudioHandle>,

    /// Path to the audio file (for serialization)
    #[ui(tooltip = "Path to the audio file")]
    pub sound_path: String,

    /// Volume (0.0 to 1.0)
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Sound volume")]
    pub volume: f32,

    /// Fade in time in seconds
    #[ui(range = 0.0..10.0, speed = 0.1, tooltip = "Fade in duration")]
    pub fade_in_time: f32,

    /// Fade out time in seconds
    #[ui(range = 0.0..10.0, speed = 0.1, tooltip = "Fade out duration")]
    pub fade_out_time: f32,

    /// Whether the sound should loop
    #[ui(tooltip = "Loop the ambient sound")]
    pub looping: bool,

    /// Whether the sound should play automatically
    #[ui(tooltip = "Start playing automatically")]
    pub auto_play: bool,
}

impl Default for AmbientSound {
    fn default() -> Self {
        Self {
            sound: None,
            sound_path: String::new(),
            volume: 0.5,
            fade_in_time: 2.0,
            fade_out_time: 2.0,
            looping: true,
            auto_play: true,
        }
    }
}

/// Audio material properties for sound interaction
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AudioMaterial")]
#[serde(default)]
pub struct AudioMaterial {
    /// Sound absorption coefficient (0.0 = fully reflective, 1.0 = fully absorptive)
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "How much sound is absorbed")]
    pub absorption: f32,

    /// Scattering coefficient (0.0 = specular reflection, 1.0 = diffuse)
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "How diffuse the reflection is")]
    pub scattering: f32,

    /// Transmission coefficient (0.0 = fully occluding, 1.0 = transparent to sound)
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "How much sound passes through")]
    pub transmission: f32,
}
