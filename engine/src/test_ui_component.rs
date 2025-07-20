//! Test component demonstrating UI annotations

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::component_registry::ComponentRegistry;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Test component demonstrating various UI annotations
#[derive(
    Debug, Clone, Serialize, Deserialize, Default, engine_derive::Component, engine_derive::EditorUI,
)]
#[component(name = "UITestComponent")]
pub struct UITestComponent {
    /// Speed value with range constraint
    #[ui(range = 0.0..10.0, step = 0.1, tooltip = "Movement speed")]
    pub speed: f32,

    /// Enabled flag
    #[ui(tooltip = "Enable or disable the component")]
    pub enabled: bool,

    /// Component name
    #[ui(tooltip = "Name of the component")]
    pub name: String,

    /// Position in 3D space
    #[ui(tooltip = "World position")]
    pub position: Vec3,

    /// Rotation quaternion
    #[ui(tooltip = "World rotation")]
    pub rotation: Quat,

    /// Tint color
    #[ui(color_mode = "rgba", tooltip = "Tint color with alpha")]
    pub tint_color: [f32; 4],

    /// Debug color (RGB only)
    #[ui(tooltip = "Debug visualization color")]
    pub debug_color: [f32; 3],

    /// Description text
    #[ui(multiline = 3, tooltip = "Multi-line description")]
    pub description: String,

    /// Internal state (hidden from UI)
    #[ui(hidden)]
    pub internal_state: u32,

    /// Read-only value
    #[ui(readonly, tooltip = "This value cannot be edited")]
    pub computed_value: f32,

    /// Integer value with range
    #[ui(range = 0.0..100.0, tooltip = "Health points")]
    pub health: i32,
}

impl UITestComponent {
    /// Create the UI metadata for this component
    pub fn create_metadata() -> crate::component_system::ui_metadata::ComponentUIMetadata {
        Self::__build_ui_metadata()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_system::ui_metadata::UIWidgetType;

    #[test]
    fn test_ui_metadata_generation() {
        let metadata = UITestComponent::create_metadata();

        // Print field names for debugging
        println!("Field names:");
        for field in &metadata.fields {
            println!("  - {}", field.name);
        }

        // Check that we have the right number of fields (excluding hidden ones)
        assert_eq!(metadata.fields.len(), 10); // All fields except internal_state

        // Find specific fields and check their properties
        let speed_field = metadata
            .fields
            .iter()
            .find(|f| f.name == "speed")
            .expect("speed field should exist");

        assert_eq!(speed_field.label, Some("speed".to_string()));
        assert_eq!(speed_field.tooltip, Some("Movement speed".to_string()));
        assert!(!speed_field.readonly);

        match &speed_field.widget {
            UIWidgetType::DragFloat {
                min,
                max,
                speed,
                format,
            } => {
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 10.0);
                assert_eq!(*speed, 0.1); // From step attribute
                assert_eq!(format, "%.3f");
            }
            _ => panic!("Expected DragFloat widget for speed field"),
        }

        // Check bool field
        let enabled_field = metadata
            .fields
            .iter()
            .find(|f| f.name == "enabled")
            .expect("enabled field should exist");

        assert_eq!(
            enabled_field.tooltip,
            Some("Enable or disable the component".to_string())
        );
        assert!(matches!(enabled_field.widget, UIWidgetType::Checkbox));

        // Check readonly field
        let computed_field = metadata
            .fields
            .iter()
            .find(|f| f.name == "computed_value")
            .expect("computed_value field should exist");

        assert!(computed_field.readonly);

        // Check that hidden field is not included
        assert!(metadata.fields.iter().all(|f| f.name != "internal_state"));

        // Check Vec3 field
        let position_field = metadata
            .fields
            .iter()
            .find(|f| f.name == "position")
            .expect("position field should exist");

        assert!(matches!(
            position_field.widget,
            UIWidgetType::Vec3Input { .. }
        ));

        // Check color field
        let tint_field = metadata
            .fields
            .iter()
            .find(|f| f.name == "tint_color")
            .expect("tint_color field should exist");

        match &tint_field.widget {
            UIWidgetType::ColorEdit { alpha } => {
                assert!(*alpha); // Should be true because of color_mode = "rgba"
            }
            _ => panic!("Expected ColorEdit widget for tint_color field"),
        }
    }
}
