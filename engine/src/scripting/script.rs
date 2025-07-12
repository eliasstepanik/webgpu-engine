//! Script loading and management

use crate::config::AssetConfig;

/// Represents a loaded script
#[derive(Debug, Clone)]
pub struct Script {
    /// Name of the script (without extension)
    pub name: String,
    /// Full path to the script file
    pub path: String,
}

impl Script {
    /// Create a new script reference
    pub fn new(name: String, path: String) -> Self {
        Self { name, path }
    }

    /// Load a script from the assets directory using default configuration
    pub fn from_name(name: &str) -> Result<Self, std::io::Error> {
        Self::from_name_with_config(name, &AssetConfig::default())
    }

    /// Load a script from the assets directory using custom configuration
    pub fn from_name_with_config(name: &str, config: &AssetConfig) -> Result<Self, std::io::Error> {
        let path = config.script_path(name);

        // Check if file exists
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Script file not found: {}", path.display()),
            ));
        }

        Ok(Self {
            name: name.to_string(),
            path: path.to_string_lossy().to_string(),
        })
    }

    /// Get the script name without extension
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the full path to the script file
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_creation() {
        let script = Script::new(
            "test_script".to_string(),
            "assets/scripts/test_script.rhai".to_string(),
        );
        assert_eq!(script.name(), "test_script");
        assert_eq!(script.path(), "assets/scripts/test_script.rhai");
    }
}
