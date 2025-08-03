//! Core audio engine using Rodio

use crate::audio::panning::{calculate_pan_volumes, MonoToStereoPanned, SourceExt};
use rodio::{cpal, Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info};

/// Handle to a playing sound instance
#[derive(Clone)]
pub struct AudioHandle {
    inner: Arc<Mutex<Option<Sink>>>,
    id: u64,
    /// Current pan value (-1.0 = left, 0.0 = center, 1.0 = right)
    pan: Arc<Mutex<f32>>,
    /// Base volume (before panning adjustment)
    base_volume: Arc<Mutex<f32>>,
}

impl std::fmt::Debug for AudioHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioHandle")
            .field("id", &self.id)
            .field("is_playing", &self.is_playing())
            .finish()
    }
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
            pan: Arc::new(Mutex::new(0.0)),
            base_volume: Arc::new(Mutex::new(1.0)),
        })
    }
}

impl AudioHandle {
    fn new(sink: Sink, id: u64, pan: f32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(sink))),
            id,
            pan: Arc::new(Mutex::new(pan)),
            base_volume: Arc::new(Mutex::new(1.0)),
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
        // Store base volume
        if let Ok(mut base_vol) = self.base_volume.lock() {
            *base_vol = volume;
        }

        // Apply volume with panning adjustment
        self.update_volume();
    }

    /// Update the actual volume based on base volume and pan
    fn update_volume(&self) {
        if let (Ok(guard), Ok(_pan), Ok(base_vol)) =
            (self.inner.lock(), self.pan.lock(), self.base_volume.lock())
        {
            if let Some(sink) = guard.as_ref() {
                // For now, just apply the base volume
                // Proper stereo panning would require per-channel control
                // TODO: When Rodio supports per-channel volume or spatial audio,
                // we can implement proper panning
                sink.set_volume(*base_vol);
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

    /// Set the stereo panning (-1.0 = full left, 0.0 = center, 1.0 = full right)
    pub fn set_panning(&self, pan: f32) {
        // Store pan value
        if let Ok(mut pan_lock) = self.pan.lock() {
            *pan_lock = pan.clamp(-1.0, 1.0);
        }

        // Update volume to apply panning
        self.update_volume();
    }

    /// Check if the sound is still playing
    pub fn is_playing(&self) -> bool {
        if let Ok(guard) = self.inner.lock() {
            if let Some(sink) = guard.as_ref() {
                // Check both if the sink is empty and if it's paused
                // A sink might not be empty but could be paused
                return !sink.empty() && !sink.is_paused();
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
    /// Create a panned source based on channel count
    fn create_panned_source<S>(source: S, pan: f32) -> Box<dyn Source<Item = f32> + Send>
    where
        S: Source<Item = f32> + Send + 'static,
    {
        match source.channels() {
            1 => {
                // Mono: convert to stereo with panning
                let (left, right) = calculate_pan_volumes(pan);
                Box::new(MonoToStereoPanned::new(source, left, right))
            }
            2 => {
                // Stereo: adjust channel volumes
                Box::new(source.panned(pan))
            }
            _ => {
                // Multi-channel: pass through
                Box::new(source)
            }
        }
    }

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

        // Load the audio file into memory
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        use std::io::Read;
        file.read_to_end(&mut bytes)?;

        // Verify it's a valid audio file by trying to decode it
        let cursor = Cursor::new(bytes.clone());
        let _test_decode = Decoder::new(cursor)?;

        // Store the original file bytes
        self.loaded_sounds.insert(path_str.clone(), Arc::new(bytes));

        info!("Loaded sound: {}", path_str);
        Ok(())
    }

    /// Play a sound with default settings
    pub fn play(&mut self, path: &str) -> Result<AudioHandle, Box<dyn std::error::Error>> {
        self.play_with_settings(path, 1.0, 1.0, false, 0.0)
    }

    /// Play a sound with custom settings
    pub fn play_with_settings(
        &mut self,
        path: &str,
        volume: f32,
        pitch: f32,
        looping: bool,
        pan: f32,
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
        // We need to clone the data because Cursor requires ownership
        let cursor = Cursor::new((*sound_data).clone());
        let source = Decoder::new(cursor)?;

        // Convert to f32 samples and apply panning
        let f32_source = source.convert_samples::<f32>();
        let panned_source = Self::create_panned_source(f32_source, pan);

        if looping {
            // Play looped
            sink.append(panned_source.repeat_infinite());
        } else {
            // Play once
            sink.append(panned_source);
        }

        // Ensure the sink is playing (it should start automatically, but just in case)
        sink.play();

        let id = self.next_handle_id;
        self.next_handle_id += 1;

        debug!(
            "Playing sound: {} (id: {}, volume: {}, looping: {}, pan: {})",
            path, id, volume, looping, pan
        );

        let handle = AudioHandle::new(sink, id, pan);
        handle.set_volume(volume, None); // Set initial volume
        Ok(handle)
    }

    /// Play a one-shot sound that doesn't need to be tracked
    pub fn play_one_shot(
        &mut self,
        path: &str,
        volume: f32,
        pitch: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.play_with_settings(path, volume, pitch, false, 0.0)?;
        Ok(())
    }

    /// Set the master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        // Note: Rodio doesn't have a direct master volume control
        // This would need to be implemented by tracking all active sounds
        debug!(
            "Master volume set to: {} (not implemented in rodio backend)",
            volume
        );
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
    pub fn set_output_device(
        &mut self,
        device_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
