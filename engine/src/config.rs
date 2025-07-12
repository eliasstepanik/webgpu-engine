//! Configuration types for the engine

use std::path::PathBuf;
use tracing::debug;

/// Configuration for asset paths
#[derive(Debug, Clone)]
pub struct AssetConfig {
    /// Root directory for all assets
    pub asset_root: PathBuf,
    /// Directory name for scripts (relative to asset_root)
    pub scripts_dir: String,
    /// Directory name for scenes (relative to asset_root)
    pub scenes_dir: String,
}

impl AssetConfig {
    /// Create a new AssetConfig with custom paths
    pub fn new(asset_root: PathBuf, scripts_dir: String, scenes_dir: String) -> Self {
        debug!(
            asset_root = ?asset_root,
            scripts_dir = scripts_dir,
            scenes_dir = scenes_dir,
            "Creating new AssetConfig"
        );
        Self {
            asset_root,
            scripts_dir,
            scenes_dir,
        }
    }

    /// Get the full path to a script file
    pub fn script_path(&self, name: &str) -> PathBuf {
        // Validate name to prevent path traversal attacks
        if name.contains("..") || name.contains("/") || name.contains("\\") {
            panic!("Invalid script name: {name}");
        }
        let path = self
            .asset_root
            .join(&self.scripts_dir)
            .join(format!("{name}.rhai"));
        debug!(name = name, path = ?path, "Generated script path");
        path
    }

    /// Get the full path to a scene file
    pub fn scene_path(&self, name: &str) -> PathBuf {
        // Validate name to prevent path traversal attacks
        if name.contains("..") || name.contains("/") || name.contains("\\") {
            panic!("Invalid scene name: {name}");
        }
        let path = self
            .asset_root
            .join(&self.scenes_dir)
            .join(format!("{name}.json"));
        debug!(name = name, path = ?path, "Generated scene path");
        path
    }

    /// Check if the asset directories exist
    pub fn validate(&self) -> Result<(), std::io::Error> {
        let scripts_path = self.asset_root.join(&self.scripts_dir);
        let scenes_path = self.asset_root.join(&self.scenes_dir);

        if !self.asset_root.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Asset root directory not found: {:?}", self.asset_root),
            ));
        }

        if !scripts_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Scripts directory not found: {scripts_path:?}"),
            ));
        }

        if !scenes_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Scenes directory not found: {scenes_path:?}"),
            ));
        }

        Ok(())
    }
}

impl Default for AssetConfig {
    /// Default configuration that matches the current project structure
    fn default() -> Self {
        Self {
            asset_root: PathBuf::from("assets"),
            scripts_dir: "scripts".to_string(),
            scenes_dir: "scenes".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_config_script_path() {
        let config = AssetConfig {
            asset_root: PathBuf::from("game/assets"),
            scripts_dir: "scripts".to_string(),
            scenes_dir: "scenes".to_string(),
        };

        let path = config.script_path("test_script");
        assert_eq!(path, PathBuf::from("game/assets/scripts/test_script.rhai"));
    }

    #[test]
    fn test_asset_config_scene_path() {
        let config = AssetConfig {
            asset_root: PathBuf::from("game/assets"),
            scripts_dir: "scripts".to_string(),
            scenes_dir: "scenes".to_string(),
        };

        let path = config.scene_path("test_scene");
        assert_eq!(path, PathBuf::from("game/assets/scenes/test_scene.json"));
    }

    #[test]
    #[should_panic(expected = "Invalid script name: ../evil")]
    fn test_asset_config_rejects_path_traversal_parent() {
        let config = AssetConfig::default();
        config.script_path("../evil");
    }

    #[test]
    #[should_panic(expected = "Invalid script name: some/path/evil")]
    fn test_asset_config_rejects_path_traversal_slash() {
        let config = AssetConfig::default();
        config.script_path("some/path/evil");
    }

    #[test]
    #[should_panic(expected = "Invalid scene name: some\\path\\evil")]
    fn test_asset_config_rejects_path_traversal_backslash() {
        let config = AssetConfig::default();
        config.scene_path("some\\path\\evil");
    }

    #[test]
    fn test_default_config() {
        let config = AssetConfig::default();
        assert_eq!(config.asset_root, PathBuf::from("assets"));
        assert_eq!(config.scripts_dir, "scripts");
        assert_eq!(config.scenes_dir, "scenes");
    }
}
