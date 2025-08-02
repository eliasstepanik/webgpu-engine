//! Stub implementations for when audio feature is disabled

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::ComponentRegistry;
use serde::{Deserialize, Serialize};

// Stub types that match the real audio API but do nothing

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioHandle;

#[derive(Debug)]
pub struct AudioEngine;

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    pub fn load_sound(
        &mut self,
        _path: impl AsRef<std::path::Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn play(&mut self, _path: &str) -> Result<AudioHandle, Box<dyn std::error::Error>> {
        Ok(AudioHandle)
    }

    pub fn play_with_settings(
        &mut self,
        _path: &str,
        _volume: f32,
        _pitch: f32,
        _looping: bool,
    ) -> Result<AudioHandle, Box<dyn std::error::Error>> {
        Ok(AudioHandle)
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AudioSource")]
pub struct AudioSource {
    pub sound: Option<AudioHandle>,
    pub sound_path: String,
    pub volume: f32,
    pub pitch: f32,
    pub looping: bool,
    pub spatial: bool,
    pub max_distance: f32,
    pub rolloff_factor: f32,
    pub auto_play: bool,
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

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AudioListener")]
pub struct AudioListener {
    pub active: bool,
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

#[derive(
    Debug, Clone, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AmbientSound")]
pub struct AmbientSound {
    pub sound: Option<AudioHandle>,
    pub sound_path: String,
    pub volume: f32,
    pub fade_in_time: f32,
    pub fade_out_time: f32,
    pub looping: bool,
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

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "AudioMaterial")]
pub struct AudioMaterial {
    pub absorption: f32,
    pub scattering: f32,
    pub transmission: f32,
}

impl Default for AudioMaterial {
    fn default() -> Self {
        Self {
            absorption: 0.1,
            scattering: 0.5,
            transmission: 0.0,
        }
    }
}

pub struct AudioSystemState;

impl Default for AudioSystemState {
    fn default() -> Self {
        Self
    }
}

pub fn audio_update_system(
    _world: &mut crate::core::entity::World,
    _audio_engine: &mut AudioEngine,
    _state: &mut AudioSystemState,
    _delta_time: f32,
) {
    // No-op when audio is disabled
}

// Stub implementations for other audio module exports
pub use AudioHandle as AudioAsset;

pub mod raycast {
    pub struct AudioRay;
    pub struct AudioRayHit;
    pub fn audio_raycast(
        _: &crate::core::entity::World,
        _: AudioRay,
        _: f32,
        _: Option<crate::core::entity::Entity>,
    ) -> Option<AudioRayHit> {
        None
    }
}
