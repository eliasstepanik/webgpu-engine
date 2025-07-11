//! Panel state management for editor windows
//!
//! Since imgui-rs 0.12 doesn't support viewports, we implement our own
//! panel detachment system using multiple OS windows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};
use winit::window::WindowId;

/// Unique identifier for a panel
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelId(pub String);

/// Serializable layout data for a panel (excludes runtime state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayout {
    /// Unique identifier
    pub id: PanelId,
    /// Display title
    pub title: String,
    /// Position in main window (when attached)
    pub position: (f32, f32),
    /// Size of the panel
    pub size: (f32, f32),
    /// Whether this panel was detached when saved
    pub was_detached: bool,
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
    /// Position in main window (when attached)
    pub position: (f32, f32),
    /// Size of the panel
    pub size: (f32, f32),
    /// Whether this panel is currently detached
    pub is_detached: bool,
    /// Window ID if detached (runtime only, not serialized)
    pub window_id: Option<WindowId>,
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
            is_detached: false,
            window_id: None,
            is_visible: true,
        }
    }

    /// Create from layout data
    /// Note: All panels start attached due to imgui-rs 0.12 limitations
    pub fn from_layout(layout: PanelLayout) -> Self {
        Self {
            id: layout.id,
            title: layout.title,
            position: layout.position,
            size: layout.size,
            is_detached: false, // Always start attached due to imgui limitations
            window_id: None,    // Runtime state
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
            was_detached: self.is_detached,
            is_visible: self.is_visible,
        }
    }

    /// Mark panel as detached
    pub fn detach(&mut self, window_id: WindowId) {
        self.is_detached = true;
        self.window_id = Some(window_id);
    }

    /// Mark panel as attached
    pub fn attach(&mut self) {
        self.is_detached = false;
        self.window_id = None;
    }
}

/// Manages all editor panels
#[derive(Debug)]
pub struct PanelManager {
    /// All registered panels
    panels: HashMap<PanelId, PanelState>,
    /// Panels that want to be detached (set by UI)
    pending_detach: Vec<PanelId>,
    /// Panels that want to be reattached
    pending_attach: Vec<PanelId>,
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

        Self {
            panels,
            pending_detach: Vec::new(),
            pending_attach: Vec::new(),
        }
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

    /// Get all visible panels for a specific window
    pub fn panels_for_window(&self, window_id: Option<WindowId>) -> Vec<&PanelState> {
        self.panels
            .values()
            .filter(|panel| {
                panel.is_visible
                    && match (panel.is_detached, panel.window_id, window_id) {
                        (false, _, None) => true, // Attached panels go to main window
                        (true, Some(pid), Some(wid)) => pid == wid, // Detached panels go to their window
                        _ => false,
                    }
            })
            .collect()
    }

    /// Request panel detachment
    pub fn request_detach(&mut self, panel_id: PanelId) {
        if !self.pending_detach.contains(&panel_id) {
            self.pending_detach.push(panel_id);
        }
    }

    /// Request panel attachment
    pub fn request_attach(&mut self, panel_id: PanelId) {
        if !self.pending_attach.contains(&panel_id) {
            self.pending_attach.push(panel_id);
        }
    }

    /// Get and clear pending detach requests
    pub fn take_pending_detach(&mut self) -> Vec<PanelId> {
        std::mem::take(&mut self.pending_detach)
    }

    /// Get and clear pending attach requests
    pub fn take_pending_attach(&mut self) -> Vec<PanelId> {
        std::mem::take(&mut self.pending_attach)
    }

    /// Handle window close - reattach any panels in that window
    pub fn handle_window_close(&mut self, window_id: WindowId) {
        for panel in self.panels.values_mut() {
            if panel.window_id == Some(window_id) {
                panel.attach();
            }
        }
    }

    /// Check if there are any pending detach or attach requests
    pub fn has_pending_requests(&self) -> bool {
        !self.pending_detach.is_empty() || !self.pending_attach.is_empty()
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
            is_detached: true,
            window_id: None,
            is_visible: true,
        };

        let layout = panel.to_layout();

        assert_eq!(layout.id.0, "test_panel");
        assert_eq!(layout.title, "Test Panel");
        assert_eq!(layout.position, (100.0, 200.0));
        assert_eq!(layout.size, (400.0, 300.0));
        assert!(layout.was_detached);
        assert!(layout.is_visible);
    }

    #[test]
    fn test_panel_from_layout() {
        let layout = PanelLayout {
            id: PanelId("test_panel".to_string()),
            title: "Test Panel".to_string(),
            position: (150.0, 250.0),
            size: (500.0, 400.0),
            was_detached: true,
            is_visible: false,
        };

        let panel = PanelState::from_layout(layout);

        assert_eq!(panel.id.0, "test_panel");
        assert_eq!(panel.title, "Test Panel");
        assert_eq!(panel.position, (150.0, 250.0));
        assert_eq!(panel.size, (500.0, 400.0));
        assert!(!panel.is_detached); // Always starts attached
        assert_eq!(panel.window_id, None); // Runtime state
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
        assert!(!loaded_panel.is_detached); // Always starts attached
    }

    #[test]
    fn test_json_format() {
        let layout = PanelLayout {
            id: PanelId("test".to_string()),
            title: "Test".to_string(),
            position: (10.0, 20.0),
            size: (100.0, 200.0),
            was_detached: false,
            is_visible: true,
        };

        let json = serde_json::to_string(&layout).expect("Failed to serialize");
        let deserialized: PanelLayout = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(layout.id.0, deserialized.id.0);
        assert_eq!(layout.title, deserialized.title);
        assert_eq!(layout.position, deserialized.position);
        assert_eq!(layout.size, deserialized.size);
        assert_eq!(layout.was_detached, deserialized.was_detached);
        assert_eq!(layout.is_visible, deserialized.is_visible);
    }
}
