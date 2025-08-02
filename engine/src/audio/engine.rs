//! Core audio engine using Kira

use kira::{
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
        Region,
    },
    AudioManager, AudioManagerSettings,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info};

/// Handle to a playing sound instance
#[derive(Debug, Clone)]
pub struct AudioHandle {
    inner: Arc<Mutex<Option<StaticSoundHandle>>>,
    id: u64,
}

// Custom serialization for AudioHandle (just saves the ID)
impl serde::Serialize for AudioHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.serialize(serializer)
    }
}

// Custom deserialization for AudioHandle (creates a dummy handle)
impl<'de> serde::Deserialize<'de> for AudioHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = u64::deserialize(deserializer)?;
        Ok(AudioHandle {
            inner: Arc::new(Mutex::new(None)),
            id,
        })
    }
}

impl AudioHandle {
    fn new(handle: StaticSoundHandle, id: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(handle))),
            id,
        }
    }

    /// Stop the sound
    pub fn stop(&self, _fade_out: Option<Duration>) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(handle) = guard.as_mut() {
                // Note: Kira v0.10 doesn't expose tween publicly, so no fade out
                let _ = handle.stop(Default::default());
            }
        }
    }

    /// Set the volume
    pub fn set_volume(&self, volume: f32, _tween: Option<()>) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(handle) = guard.as_mut() {
                let _ = handle.set_volume(volume, Default::default());
            }
        }
    }

    /// Set the playback rate (pitch)
    pub fn set_playback_rate(&self, rate: f32, _tween: Option<()>) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(handle) = guard.as_mut() {
                let _ = handle.set_playback_rate(rate as f64, Default::default());
            }
        }
    }

    /// Check if the sound is still playing
    pub fn is_playing(&self) -> bool {
        if let Ok(guard) = self.inner.lock() {
            if let Some(handle) = guard.as_ref() {
                return handle.state() == kira::sound::PlaybackState::Playing;
            }
        }
        false
    }
}

/// Core audio engine
pub struct AudioEngine {
    manager: AudioManager,
    loaded_sounds: HashMap<String, StaticSoundData>,
    next_handle_id: u64,
}

impl AudioEngine {
    /// Create a new audio engine
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing audio engine");

        let settings = AudioManagerSettings::default();

        match AudioManager::new(settings) {
            Ok(manager) => {
                info!("Audio backend initialized successfully");
                Ok(Self {
                    manager,
                    loaded_sounds: HashMap::new(),
                    next_handle_id: 0,
                })
            }
            Err(e) => {
                // Provide more context about the error
                let error_msg = format!("Failed to initialize audio backend: {}", e);
                if cfg!(target_os = "linux") && std::env::var("WSL_DISTRO_NAME").is_ok() {
                    Err(format!("{}\nDetected WSL environment. Audio may not work properly. Consider running natively on Windows or installing ALSA: sudo apt-get install pkg-config libasound2-dev", error_msg).into())
                } else {
                    Err(error_msg.into())
                }
            }
        }
    }

    /// Load a sound from a file path
    pub fn load_sound(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();

        // Check if already loaded
        if self.loaded_sounds.contains_key(&path_str) {
            debug!("Sound already loaded: {}", path_str);
            return Ok(());
        }

        debug!("Loading sound: {}", path_str);

        // Load the sound data
        let sound_data = StaticSoundData::from_file(path)?;
        self.loaded_sounds.insert(path_str.clone(), sound_data);

        info!("Loaded sound: {}", path_str);
        Ok(())
    }

    /// Play a sound with default settings
    pub fn play(&mut self, path: &str) -> Result<AudioHandle, Box<dyn std::error::Error>> {
        self.play_with_settings(path, 1.0, 1.0, false)
    }

    /// Play a sound with custom settings
    pub fn play_with_settings(
        &mut self,
        path: &str,
        volume: f32,
        pitch: f32,
        looping: bool,
    ) -> Result<AudioHandle, Box<dyn std::error::Error>> {
        // Ensure sound is loaded
        if !self.loaded_sounds.contains_key(path) {
            self.load_sound(path)?;
        }

        // Get the sound data
        let sound_data = self
            .loaded_sounds
            .get(path)
            .ok_or_else(|| format!("Sound not found: {}", path))?;

        // Configure playback settings
        let mut settings = StaticSoundSettings::default();
        settings.volume = volume.into();
        settings.playback_rate = (pitch as f64).into();
        if looping {
            settings.loop_region = Some(Region::default());
        }

        // Play the sound
        let handle = self.manager.play(sound_data.with_settings(settings))?;

        let id = self.next_handle_id;
        self.next_handle_id += 1;

        debug!("Playing sound: {} (id: {})", path, id);
        Ok(AudioHandle::new(handle, id))
    }

    /// Play a one-shot sound that doesn't need to be tracked
    pub fn play_one_shot(
        &mut self,
        path: &str,
        volume: f32,
        pitch: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.play_with_settings(path, volume, pitch, false)?;
        Ok(())
    }

    /// Set the master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        // Note: Kira doesn't have a direct master volume control
        // This would need to be implemented by tracking all active sounds
        // or using the mixer tracks feature
        debug!("Master volume set to: {}", volume);
    }

    /// Unload a sound from memory
    pub fn unload_sound(&mut self, path: &str) {
        if self.loaded_sounds.remove(path).is_some() {
            debug!("Unloaded sound: {}", path);
        }
    }

    /// Clear all loaded sounds
    pub fn clear_sounds(&mut self) {
        let count = self.loaded_sounds.len();
        self.loaded_sounds.clear();
        info!("Cleared {} loaded sounds", count);
    }

    /// Get the number of loaded sounds
    pub fn loaded_sound_count(&self) -> usize {
        self.loaded_sounds.len()
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create audio engine")
    }
}
