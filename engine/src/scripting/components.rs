//! Script-related components

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::component_registry::ComponentRegistry;
use serde::{Deserialize, Serialize};

/// Reference to a script asset
///
/// This component can be attached to entities to give them scripted behavior.
/// The script is loaded from the `assets/scripts/` directory with a `.rhai` extension.
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Default,
    engine_derive::Component,
    engine_derive::EditorUI,
)]
#[component(name = "ScriptRef")]
pub struct ScriptRef {
    /// Script name without extension (e.g., "fly_camera")
    pub name: String,
}

impl ScriptRef {
    /// Create a new script reference
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Get the full path to the script file
    ///
    /// **Deprecated**: This method uses hardcoded paths. Use AssetConfig with ScriptEngine instead.
    #[deprecated(note = "Use AssetConfig with ScriptEngine for configurable paths")]
    pub fn path(&self) -> String {
        format!("assets/scripts/{}.rhai", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_ref_new() {
        let script_ref = ScriptRef::new("test_script");
        assert_eq!(script_ref.name, "test_script");
    }

    #[test]
    #[allow(deprecated)]
    fn test_script_ref_path() {
        let script_ref = ScriptRef::new("fly_camera");
        assert_eq!(script_ref.path(), "assets/scripts/fly_camera.rhai");
    }

    #[test]
    fn test_script_ref_serialization() {
        let script_ref = ScriptRef::new("test_script");
        let json = serde_json::to_string(&script_ref).unwrap();
        assert_eq!(json, r#"{"name":"test_script"}"#);

        let deserialized: ScriptRef = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, script_ref);
    }
}
