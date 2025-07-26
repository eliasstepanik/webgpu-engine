//! Asset browser panel
//!
//! Displays available assets like scenes, meshes, and materials with drag-and-drop support.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use imgui::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// File node representing a file or directory in the asset tree
#[derive(Debug, Clone)]
struct FileNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
}

impl Default for FileNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: PathBuf::new(),
            is_dir: false,
            children: Vec::new(),
        }
    }
}

/// Asset browser state
pub struct AssetBrowserState {
    asset_root: PathBuf,
    file_tree: FileNode,
    dragged_file: Option<String>,
    expanded_dirs: HashMap<PathBuf, bool>,
}

impl AssetBrowserState {
    fn new(asset_root: PathBuf) -> Self {
        let file_tree = scan_directory(&asset_root).unwrap_or_default();
        Self {
            asset_root,
            file_tree,
            dragged_file: None,
            expanded_dirs: HashMap::new(),
        }
    }

    /// Get the global asset browser state, creating it if necessary
    #[allow(static_mut_refs)]
    pub fn get_dragged_file() -> Option<String> {
        unsafe {
            ASSET_BROWSER_STATE
                .as_ref()
                .and_then(|s| s.dragged_file.clone())
        }
    }

    /// Take the dragged file (consumes it)
    #[allow(static_mut_refs)]
    pub fn take_dragged_file() -> Option<String> {
        unsafe {
            ASSET_BROWSER_STATE
                .as_mut()
                .and_then(|s| s.dragged_file.take())
        }
    }
}

// Global state (similar to inspector.rs pattern)
static mut ASSET_BROWSER_STATE: Option<AssetBrowserState> = None;

/// Get the asset browser state, creating it if necessary
#[allow(static_mut_refs)]
fn get_asset_browser_state() -> &'static mut AssetBrowserState {
    unsafe {
        if ASSET_BROWSER_STATE.is_none() {
            let asset_root = PathBuf::from("game/assets");
            ASSET_BROWSER_STATE = Some(AssetBrowserState::new(asset_root));
        }
        ASSET_BROWSER_STATE.as_mut().unwrap()
    }
}

/// Scan a directory recursively and build a file tree
fn scan_directory(path: &Path) -> Result<FileNode, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    if metadata.is_dir() {
        let mut children = Vec::new();
        let entries = std::fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let child_path = entry.path();

            // Skip hidden files and directories
            if let Some(name) = child_path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }

            match scan_directory(&child_path) {
                Ok(child_node) => children.push(child_node),
                Err(e) => {
                    warn!("Failed to scan {}: {}", child_path.display(), e);
                }
            }
        }

        // Sort: directories first, then files, alphabetically
        children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(FileNode {
            name,
            path: path.to_path_buf(),
            is_dir: true,
            children,
        })
    } else {
        // Only include files with supported extensions
        let include = matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("obj") | Some("rhai") | Some("json")
        );

        if include {
            Ok(FileNode {
                name,
                path: path.to_path_buf(),
                is_dir: false,
                children: Vec::new(),
            })
        } else {
            Err(std::io::Error::other("Unsupported file type"))
        }
    }
}

/// Check if a file is a scene file (JSON in scenes directory)
pub fn is_scene_file(path: &Path) -> bool {
    // Check if it's a JSON file
    if let Some(ext) = path.extension() {
        if ext == "json" {
            // Check if path contains "scenes/" directory
            let path_str = path.to_string_lossy();
            return path_str.contains("scenes/") || path_str.contains("scenes\\");
        }
    }
    false
}

/// Check if a file is draggable
fn is_draggable_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("obj") | Some("rhai") => true,
        Some("json") => is_scene_file(path), // Only drag scene JSONs
        _ => false,
    }
}

/// Get the appropriate icon for a file type
fn get_file_icon(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("obj") => "🗿",
        Some("rhai") => "📜",
        Some("json") => {
            if is_scene_file(path) {
                "🎬" // Scene icon
            } else {
                "📋" // Regular JSON
            }
        }
        _ => "📄",
    }
}

/// Render a file tree node recursively
fn render_file_tree(ui: &imgui::Ui, node: &FileNode, state: &mut AssetBrowserState) {
    if node.is_dir {
        // Skip empty directories
        if node.children.is_empty() {
            return;
        }

        let is_expanded = state
            .expanded_dirs
            .get(&node.path)
            .copied()
            .unwrap_or(false);
        let flags = if is_expanded {
            TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::OPEN_ON_ARROW
        } else {
            TreeNodeFlags::OPEN_ON_ARROW
        };

        let _id_token = ui.push_id(node.path.to_string_lossy());
        if let Some(_token) = ui
            .tree_node_config(&format!("📁 {}", node.name))
            .flags(flags)
            .push()
        {
            state.expanded_dirs.insert(node.path.clone(), true);
            for child in &node.children {
                render_file_tree(ui, child, state);
            }
        } else {
            state.expanded_dirs.insert(node.path.clone(), false);
        }
    } else {
        // File node - make it draggable
        let file_icon = get_file_icon(&node.path);

        let _id_token = ui.push_id(node.path.to_string_lossy());
        let label = format!("{} {}", file_icon, node.name);
        ui.selectable(&label);

        // Add tooltip with full path
        if ui.is_item_hovered() {
            ui.tooltip(|| {
                ui.text(format!("Path: {}", node.path.display()));
            });
        }

        // Make draggable if it's a supported file type
        if is_draggable_file(&node.path) {
            if let Some(_source) = ui
                .drag_drop_source_config("ASSET_FILE")
                .condition(Condition::Once)
                .begin()
            {
                // Calculate relative path from asset root
                let relative_path = node
                    .path
                    .strip_prefix(&state.asset_root)
                    .unwrap_or(&node.path)
                    .to_string_lossy()
                    .to_string()
                    .replace('\\', "/"); // Normalize path separators

                state.dragged_file = Some(relative_path.clone());
                ui.text(format!("📄 {}", node.name));
                debug!("Started dragging file: {}", relative_path);

                _source.end();
            }
        }
    }
}

/// Validate an asset path to ensure it's safe and within the asset root
pub fn validate_asset_path(path: &str) -> bool {
    // No parent directory references
    if path.contains("..") {
        return false;
    }

    // Must be within asset root
    let full_path = PathBuf::from(path);
    if full_path.is_absolute() {
        return false;
    }

    // Check file exists
    let asset_path = PathBuf::from("game/assets").join(path);
    asset_path.exists() && asset_path.is_file()
}

/// Render the assets panel
pub fn render_assets_panel(
    ui: &imgui::Ui,
    _shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
) {
    let panel_id = PanelId("assets".to_string());

    // Get panel info
    let (panel_title, is_visible) = {
        match panel_manager.get_panel(&panel_id) {
            Some(panel) => (panel.title.clone(), panel.is_visible),
            None => return,
        }
    };

    if !is_visible {
        return;
    }

    let window_name = format!("{}##{}", panel_title, panel_id.0);

    ui.window(&window_name)
        .size([800.0, 200.0], Condition::FirstUseEver)
        .position([100.0, 500.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
            let state = get_asset_browser_state();

            // Header
            ui.text("Assets");
            ui.separator();

            // Refresh button
            if ui.button("🔄 Refresh") {
                debug!("Refreshing asset browser");
                state.file_tree = scan_directory(&state.asset_root).unwrap_or_default();
            }

            ui.same_line();
            ui.text(format!("Root: {}", state.asset_root.display()));

            ui.separator();

            // File tree
            ui.child_window("asset_tree").build(|| {
                if state.file_tree.children.is_empty() {
                    ui.text("No assets found");
                    ui.text("Place .obj, .rhai, or .json files in:");
                    ui.text(format!("{}", state.asset_root.display()));
                } else {
                    // Render each top-level child
                    let children = state.file_tree.children.clone();
                    for child in children.iter() {
                        render_file_tree(ui, child, state);
                    }
                }
            });
        });
}
