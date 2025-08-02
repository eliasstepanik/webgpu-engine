//! Core audio engine using Rodio

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Handle to a playing sound instance
#[derive(Debug, Clone)]
pub struct AudioHandle {
    inner: Arc<Mutex<Option<Sink>>>,
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
    fn new(sink: Sink, id: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(sink))),
            id,
        }
    }

    /// Stop the sound
    pub fn stop(&self, _fade_out: Option<Duration>) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(sink) = guard.take() {
                // Note: Rodio doesn't have built-in fade out, just stop
                sink.stop();
            }
        }
    }

    /// Set the volume
    pub fn set_volume(&self, volume: f32, _tween: Option<()>) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(sink) = guard.as_ref() {
                sink.set_volume(volume);
            }
        }
    }

    /// Set the playback rate (pitch)
    pub fn set_playback_rate(&self, rate: f32, _tween: Option<()>) {
        if let Ok(guard) = self.inner.lock() {
            if let Some(sink) = guard.as_ref() {
                sink.set_speed(rate);
            }
        }
    }

    /// Check if the sound is still playing
    pub fn is_playing(&self) -> bool {
        if let Ok(guard) = self.inner.lock() {
            if let Some(sink) = guard.as_ref() {
                return !sink.empty();
            }
        }
        false
    }
}

/// Core audio engine
pub struct AudioEngine {
    // Rodio requires keeping OutputStream alive
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    
    // Device management
    current_device: Option<String>,
    
    // Sound management (cache decoded audio data)
    loaded_sounds: HashMap<String, Arc<Vec<u8>>>,
    next_handle_id: u64,
}

impl AudioEngine {
    /// Create a new audio engine
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing audio engine with Rodio");

        match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                info!("Audio backend initialized successfully");
                Ok(Self {
                    _stream: stream,
                    stream_handle,
                    current_device: None,
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

        // Load and decode the audio file
        let file = BufReader::new(File::open(path)?);
        let source = Decoder::new(file)?;
        
        // Collect samples into a buffer
        // Note: We collect as i16 samples for efficiency
        let samples: Vec<i16> = source.collect();
        
        // Convert to bytes for storage
        let mut bytes = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        
        self.loaded_sounds.insert(path_str.clone(), Arc::new(bytes));

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
            .ok_or_else(|| format!("Sound not found: {}", path))?
            .clone();

        // Create a new sink for this sound
        let sink = Sink::try_new(&self.stream_handle)?;
        
        // Configure playback settings
        sink.set_volume(volume);
        sink.set_speed(pitch);
        
        // Create source from cached data
        let cursor = Cursor::new(sound_data);
        let source = Decoder::new(cursor)?;
        
        if looping {
            // Play looped
            sink.append(source.repeat_infinite());
        } else {
            // Play once
            sink.append(source);
        }

        let id = self.next_handle_id;
        self.next_handle_id += 1;

        debug!("Playing sound: {} (id: {})", path, id);
        Ok(AudioHandle::new(sink, id))
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
        // Note: Rodio doesn't have a direct master volume control
        // This would need to be implemented by tracking all active sounds
        debug!("Master volume set to: {} (not implemented in rodio backend)", volume);
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

    // New device enumeration methods

    /// Enumerate available audio output devices
    pub fn enumerate_devices() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        use cpal::traits::{DeviceTrait, HostTrait};
        
        let host = cpal::default_host();
        let mut devices = Vec::new();
        
        // Add default device first
        if let Some(default_device) = host.default_output_device() {
            if let Ok(name) = default_device.name() {
                devices.push(format!("{} (Default)", name));
            }
        }
        
        // Add all other devices
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    // Skip if it's the default device (already added)
                    if !devices.iter().any(|d| d.starts_with(&name)) {
                        devices.push(name);
                    }
                }
            }
        }
        
        Ok(devices)
    }

    /// Set the output device by name
    pub fn set_output_device(&mut self, device_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        use cpal::traits::{DeviceTrait, HostTrait};
        
        // Clean device name (remove "(Default)" suffix if present)
        let clean_name = device_name.trim_end_matches(" (Default)");
        
        let host = cpal::default_host();
        
        // Find the device
        let device = if device_name.contains("(Default)") {
            // Use default device
            host.default_output_device()
                .ok_or("No default output device found")?
        } else {
            // Find specific device
            host.output_devices()?
                .find(|d| d.name().ok().as_deref() == Some(clean_name))
                .ok_or_else(|| format!("Device '{}' not found", clean_name))?
        };
        
        // Create new stream with the selected device
        let (stream, stream_handle) = OutputStream::try_from_device(&device)?;
        
        // Replace the stream (this will stop all currently playing sounds)
        self._stream = stream;
        self.stream_handle = stream_handle;
        self.current_device = Some(device_name.to_string());
        
        info!("Audio output device changed to: {}", device_name);
        Ok(())
    }

    /// Get the current device name
    pub fn get_current_device(&self) -> Option<String> {
        self.current_device.clone()
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create audio engine")
    }
}