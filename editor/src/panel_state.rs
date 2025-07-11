//! Panel state management for editor windows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};

/// Unique identifier for a panel
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelId(pub String);

/// Serializable layout data for a panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayout {
    /// Unique identifier
    pub id: PanelId,
    /// Display title
    pub title: String,
    /// Position in main window
    pub position: (f32, f32),
    /// Size of the panel
    pub size: (f32, f32),
    /// Whether the panel is visible
    pub is_visible: bool,
}

/// State of an individual panel
#[derive(Debug, Clone)]
pub struct PanelState {
    /// Unique identifier
    pub id: PanelId,
    /// Display title
    pub title: String,
    /// Position in main window
    pub position: (f32, f32),
    /// Size of the panel
    pub size: (f32, f32),
    /// Whether the panel is visible
    pub is_visible: bool,
}

impl PanelState {
    /// Create a new panel state
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: PanelId(id.into()),
            title: title.into(),
            position: (100.0, 100.0),
            size: (400.0, 300.0),
            is_visible: true,
        }
    }

    /// Create from layout data
    pub fn from_layout(layout: PanelLayout) -> Self {
        // Validate and clamp position to reasonable bounds
        let position = (
            layout.position.0.clamp(0.0, 2000.0),
            layout.position.1.clamp(0.0, 2000.0),
        );

        // Validate size
        let size = (
            layout.size.0.clamp(100.0, 1920.0),
            layout.size.1.clamp(100.0, 1080.0),
        );

        Self {
            id: layout.id,
            title: layout.title,
            position,
            size,
            is_visible: layout.is_visible,
        }
    }

    /// Convert to layout data for serialization
    pub fn to_layout(&self) -> PanelLayout {
        PanelLayout {
            id: self.id.clone(),
            title: self.title.clone(),
            position: self.position,
            size: self.size,
            is_visible: self.is_visible,
        }
    }
}

/// Manages all editor panels
#[derive(Debug)]
pub struct PanelManager {
    /// All registered panels
    panels: HashMap<PanelId, PanelState>,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelManager {
    /// Create a new panel manager with default panels
    pub fn new() -> Self {
        let mut panels = HashMap::new();

        // Register default panels
        let default_panels = vec![
            PanelState::new("hierarchy", "Hierarchy"),
            PanelState::new("inspector", "Inspector"),
            PanelState::new("assets", "Assets"),
            PanelState::new("viewport", "Viewport"),
        ];

        for panel in default_panels {
            panels.insert(panel.id.clone(), panel);
        }

        Self { panels }
    }

    /// Create a new panel manager and try to load layout from file
    pub fn with_layout_file<P: AsRef<Path>>(layout_path: P) -> Self {
        let mut manager = Self::new();

        if let Err(e) = manager.load_layout(&layout_path) {
            warn!(
                "Failed to load layout from {:?}: {}",
                layout_path.as_ref(),
                e
            );
            info!("Using default panel layout");
        }

        manager
    }

    /// Register a new panel
    pub fn register_panel(&mut self, panel: PanelState) {
        self.panels.insert(panel.id.clone(), panel);
    }

    /// Get a panel by ID
    pub fn get_panel(&self, id: &PanelId) -> Option<&PanelState> {
        self.panels.get(id)
    }

    /// Get a mutable panel by ID
    pub fn get_panel_mut(&mut self, id: &PanelId) -> Option<&mut PanelState> {
        self.panels.get_mut(id)
    }

    /// Get all panels
    pub fn panels(&self) -> impl Iterator<Item = &PanelState> {
        self.panels.values()
    }

    /// Save current layout to JSON file
    pub fn save_layout<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let layouts: Vec<PanelLayout> = self
            .panels
            .values()
            .map(|panel| panel.to_layout())
            .collect();

        let json = serde_json::to_string_pretty(&layouts)?;
        std::fs::write(&path, json)?;

        info!("Saved panel layout to {:?}", path.as_ref());
        Ok(())
    }

    /// Load layout from JSON file
    pub fn load_layout<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(&path)?;
        let layouts: Vec<PanelLayout> = serde_json::from_str(&content)?;

        // Clear existing panels and load from file
        self.panels.clear();

        for layout in layouts {
            let panel = PanelState::from_layout(layout);
            self.panels.insert(panel.id.clone(), panel);
        }

        info!("Loaded panel layout from {:?}", path.as_ref());
        Ok(())
    }

    /// Get the default layout file path
    pub fn default_layout_path() -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("editor_layout.json")
    }

    /// Save layout to the default file path
    pub fn save_default_layout(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_layout(Self::default_layout_path())
    }

    /// Load layout from the default file path
    pub fn load_default_layout(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_layout(Self::default_layout_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_panel_layout_serialization() {
        let panel = PanelState {
            id: PanelId("test_panel".to_string()),
            title: "Test Panel".to_string(),
            position: (100.0, 200.0),
            size: (400.0, 300.0),
            is_visible: true,
        };

        let layout = panel.to_layout();

        assert_eq!(layout.id.0, "test_panel");
        assert_eq!(layout.title, "Test Panel");
        assert_eq!(layout.position, (100.0, 200.0));
        assert_eq!(layout.size, (400.0, 300.0));
        assert!(layout.is_visible);
    }

    #[test]
    fn test_panel_from_layout() {
        let layout = PanelLayout {
            id: PanelId("test_panel".to_string()),
            title: "Test Panel".to_string(),
            position: (150.0, 250.0),
            size: (500.0, 400.0),
            is_visible: false,
        };

        let panel = PanelState::from_layout(layout);

        assert_eq!(panel.id.0, "test_panel");
        assert_eq!(panel.title, "Test Panel");
        assert_eq!(panel.position, (150.0, 250.0));
        assert_eq!(panel.size, (500.0, 400.0));
        assert!(!panel.is_visible);
    }

    #[test]
    fn test_save_load_layout() {
        let mut manager = PanelManager::new();

        // Modify a panel
        if let Some(panel) = manager.get_panel_mut(&PanelId("hierarchy".to_string())) {
            panel.position = (50.0, 75.0);
            panel.size = (300.0, 450.0);
            panel.is_visible = false;
        }

        // Save to temporary file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        manager
            .save_layout(temp_path)
            .expect("Failed to save layout");

        // Create new manager and load
        let mut new_manager = PanelManager::new();
        new_manager
            .load_layout(temp_path)
            .expect("Failed to load layout");

        // Verify the loaded panel
        let loaded_panel = new_manager
            .get_panel(&PanelId("hierarchy".to_string()))
            .expect("Panel not found");

        assert_eq!(loaded_panel.position, (50.0, 75.0));
        assert_eq!(loaded_panel.size, (300.0, 450.0));
        assert!(!loaded_panel.is_visible);
    }

    #[test]
    fn test_json_format() {
        let layout = PanelLayout {
            id: PanelId("test".to_string()),
            title: "Test".to_string(),
            position: (10.0, 20.0),
            size: (100.0, 200.0),
            is_visible: true,
        };

        let json = serde_json::to_string(&layout).expect("Failed to serialize");
        let deserialized: PanelLayout = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(layout.id.0, deserialized.id.0);
        assert_eq!(layout.title, deserialized.title);
        assert_eq!(layout.position, deserialized.position);
        assert_eq!(layout.size, deserialized.size);
        assert_eq!(layout.is_visible, deserialized.is_visible);
    }
}
