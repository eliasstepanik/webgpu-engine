//! Audio resource management

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::debug;

/// Audio asset that can be loaded and played
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    /// Path to the audio file
    pub path: PathBuf,
    /// Whether this is a streaming asset (for large files)
    pub streaming: bool,
    /// Default volume for this asset
    pub default_volume: f32,
    /// Asset metadata
    pub metadata: AudioMetadata,
}

/// Audio asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    /// Asset name
    pub name: String,
    /// Asset category (e.g., "sfx", "music", "ambient")
    pub category: String,
    /// Tags for filtering
    pub tags: Vec<String>,
}

impl Default for AudioMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            category: "sfx".to_string(),
            tags: Vec::new(),
        }
    }
}

/// Audio asset manager
pub struct AudioAssetManager {
    /// Base path for audio assets
    pub base_path: PathBuf,
    /// Loaded assets
    assets: std::collections::HashMap<String, AudioAsset>,
}

impl AudioAssetManager {
    /// Create a new audio asset manager
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            assets: std::collections::HashMap::new(),
        }
    }

    /// Register an audio asset
    pub fn register_asset(&mut self, id: &str, asset: AudioAsset) {
        debug!("Registered audio asset: {}", id);
        self.assets.insert(id.to_string(), asset);
    }

    /// Get an audio asset by ID
    pub fn get_asset(&self, id: &str) -> Option<&AudioAsset> {
        self.assets.get(id)
    }

    /// Get the full path for an asset
    pub fn get_asset_path(&self, asset: &AudioAsset) -> PathBuf {
        if asset.path.is_absolute() {
            asset.path.clone()
        } else {
            self.base_path.join(&asset.path)
        }
    }

    /// Load audio asset manifest from JSON
    pub fn load_manifest(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let manifest: AudioManifest = serde_json::from_str(&content)?;

        for (id, asset) in manifest.assets {
            self.register_asset(&id, asset);
        }

        Ok(())
    }

    /// Find assets by category
    pub fn find_by_category(&self, category: &str) -> Vec<(&String, &AudioAsset)> {
        self.assets
            .iter()
            .filter(|(_, asset)| asset.metadata.category == category)
            .collect()
    }

    /// Find assets by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<(&String, &AudioAsset)> {
        self.assets
            .iter()
            .filter(|(_, asset)| asset.metadata.tags.contains(&tag.to_string()))
            .collect()
    }
}

/// Audio asset manifest for batch loading
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioManifest {
    /// Map of asset ID to asset data
    pub assets: std::collections::HashMap<String, AudioAsset>,
}

/// Common audio asset categories
pub mod categories {
    pub const SFX: &str = "sfx";
    pub const MUSIC: &str = "music";
    pub const AMBIENT: &str = "ambient";
    pub const UI: &str = "ui";
    pub const VOICE: &str = "voice";
}

/// Helper to create audio assets
pub struct AudioAssetBuilder {
    path: PathBuf,
    streaming: bool,
    default_volume: f32,
    metadata: AudioMetadata,
}

impl AudioAssetBuilder {
    /// Create a new audio asset builder
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            streaming: false,
            default_volume: 1.0,
            metadata: AudioMetadata::default(),
        }
    }

    /// Set whether this is a streaming asset
    pub fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    /// Set default volume
    pub fn default_volume(mut self, volume: f32) -> Self {
        self.default_volume = volume;
        self
    }

    /// Set asset name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.metadata.name = name.into();
        self
    }

    /// Set asset category
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.metadata.category = category.into();
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata
            .tags
            .extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Build the audio asset
    pub fn build(self) -> AudioAsset {
        AudioAsset {
            path: self.path,
            streaming: self.streaming,
            default_volume: self.default_volume,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_asset_builder() {
        let asset = AudioAssetBuilder::new("sounds/explosion.wav")
            .name("Explosion")
            .category(categories::SFX)
            .tag("combat")
            .tag("loud")
            .default_volume(0.8)
            .build();

        assert_eq!(asset.path.to_str().unwrap(), "sounds/explosion.wav");
        assert_eq!(asset.metadata.name, "Explosion");
        assert_eq!(asset.metadata.category, "sfx");
        assert_eq!(asset.metadata.tags, vec!["combat", "loud"]);
        assert_eq!(asset.default_volume, 0.8);
    }

    #[test]
    fn test_asset_manager() {
        let mut manager = AudioAssetManager::new("assets/audio");

        let asset = AudioAssetBuilder::new("sfx/jump.ogg")
            .category(categories::SFX)
            .tag("player")
            .build();

        manager.register_asset("jump", asset);

        assert!(manager.get_asset("jump").is_some());
        assert_eq!(manager.find_by_category(categories::SFX).len(), 1);
        assert_eq!(manager.find_by_tag("player").len(), 1);
    }
}
