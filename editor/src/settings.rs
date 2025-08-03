//! Editor settings management
//!
//! This module provides persistent settings storage for the editor,
//! including audio configuration and other user preferences.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Main editor settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    /// Audio-related settings
    #[serde(default)]
    pub audio: AudioSettings,

    /// Settings version for future migration support
    #[serde(default)]
    pub version: u32,
}

/// Audio configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// Selected output device name (None = system default)
    pub output_device: Option<String>,
    /// Master volume level (0.0 - 1.0)
    pub master_volume: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            audio: AudioSettings::default(),
            version: 1,
        }
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            output_device: None,
            master_volume: 1.0,
        }
    }
}

impl EditorSettings {
    /// Get the default path for the settings file
    pub fn default_path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("editor_settings.json")
    }

    /// Save settings to the default location
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::default_path(), json)?;
        info!("Saved editor settings");
        Ok(())
    }

    /// Load settings from the default location
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::default_path();
        if !path.exists() {
            info!("No settings file found, using defaults");
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        match serde_json::from_str::<Self>(&content) {
            Ok(settings) => {
                info!("Loaded editor settings from {:?}", path);
                Ok(settings)
            }
            Err(e) => {
                warn!("Failed to parse settings file: {}. Using defaults.", e);
                Ok(Self::default())
            }
        }
    }

    /// Save settings to a specific path
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        info!("Saved editor settings to {:?}", path.as_ref());
        Ok(())
    }

    /// Load settings from a specific path
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(&path)?;
        let settings = serde_json::from_str(&content)?;
        info!("Loaded editor settings from {:?}", path.as_ref());
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_settings() {
        let settings = EditorSettings::default();
        assert_eq!(settings.version, 1);
        assert_eq!(settings.audio.master_volume, 1.0);
        assert!(settings.audio.output_device.is_none());
    }

    #[test]
    fn test_save_load_settings() {
        let mut settings = EditorSettings::default();
        settings.audio.master_volume = 0.75;
        settings.audio.output_device = Some("Test Device".to_string());

        // Save to temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        settings
            .save_to(temp_path)
            .expect("Failed to save settings");

        // Load from file
        let loaded = EditorSettings::load_from(temp_path).expect("Failed to load settings");

        assert_eq!(loaded.audio.master_volume, 0.75);
        assert_eq!(loaded.audio.output_device, Some("Test Device".to_string()));
    }

    #[test]
    fn test_invalid_json_fallback() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Write invalid JSON
        std::fs::write(temp_path, "{ invalid json }").expect("Failed to write file");

        // Should return default settings on parse error
        let result = EditorSettings::load_from(temp_path);
        assert!(result.is_err());
    }
}
