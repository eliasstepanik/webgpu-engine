# Asset Browser with Drag-and-Drop Support

## Overview
Implement a functional asset browser panel that displays files from the game/assets directory and supports drag-and-drop operations to load files onto mesh and script components in the entity inspector.

## Success Criteria
- [ ] Asset browser displays files from game/assets directory in a tree structure
- [ ] Files can be dragged from the asset browser
- [ ] Mesh component input fields accept .obj file drops
- [ ] Script component input fields accept .rhai file drops
- [ ] Path validation prevents directory traversal attacks
- [ ] Visual feedback during drag operations

## Context and Research

### Current State
- Asset browser is an empty stub at `editor/src/panels/assets.rs`
- Inspector components use `input_text` fields for mesh and script values
- AssetConfig at `engine/src/config.rs` provides path validation patterns
- Using imgui 0.12 with drag_drop module support

### Documentation References
- imgui-rs drag_drop module: https://docs.rs/imgui/latest/imgui/drag_drop/index.html
- DragDropSource: https://docs.rs/imgui/latest/imgui/drag_drop/struct.DragDropSource.html
- DragDropTarget: https://docs.rs/imgui/latest/imgui/drag_drop/struct.DragDropTarget.html

### Key Code Patterns to Follow

**Panel Structure (from hierarchy.rs):**
```rust
pub fn render_assets_panel(
    ui: &imgui::Ui,
    shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
) {
    // Panel boilerplate...
    ui.window(&window_name)
        .size([800.0, 200.0], Condition::FirstUseEver)
        .build(|| {
            // Implementation here
        });
}
```

**Tree Display (from hierarchy.rs):**
```rust
let is_open = ui.tree_node_config(&name).flags(node_flags).push();
if is_open {
    // Children
    ui.tree_pop();
}
```

**Drag Source Pattern (from imgui-rs docs):**
```rust
ui.selectable(&file_name);
if ui.drag_drop_source_config("ASSET_FILE")
    .condition(Condition::Once)
    .begin()
    .is_some()
{
    // Store file path in state
    state.dragged_file = Some(file_path);
    ui.text(&format!("Moving: {}", file_name));
}
```

**Drop Target Pattern (to add to inspector.rs):**
```rust
if ui.input_text("Mesh ID", &mut mesh_name).build() {
    // existing code
}
// Add drop target
if let Some(target) = ui.drag_drop_target() {
    if target.accept_payload_empty("ASSET_FILE", DragDropFlags::empty()).is_some() {
        if let Some(file_path) = state.dragged_file.take() {
            if file_path.ends_with(".obj") {
                mesh_id.0 = file_path;
                shared_state.mark_scene_modified();
            }
        }
    }
    target.pop();
}
```

## Implementation Blueprint

### Phase 1: Asset Browser State
```rust
// In editor/src/panels/assets.rs

use std::path::{Path, PathBuf};
use std::collections::HashMap;

struct AssetBrowserState {
    asset_root: PathBuf,
    file_tree: FileNode,
    dragged_file: Option<String>,
    expanded_dirs: HashMap<PathBuf, bool>,
}

struct FileNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    children: Vec<FileNode>,
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
}

fn scan_directory(path: &Path) -> Result<FileNode, std::io::Error> {
    // Recursive directory scanning
    // Filter for .obj, .rhai, .json files
    // Sort directories first, then files alphabetically
}

// Global state (similar to inspector.rs pattern)
static mut ASSET_BROWSER_STATE: Option<AssetBrowserState> = None;
```

### Phase 2: File Display with Drag Sources
```rust
fn render_file_tree(ui: &imgui::Ui, node: &FileNode, state: &mut AssetBrowserState) {
    if node.is_dir {
        let is_expanded = state.expanded_dirs.get(&node.path).copied().unwrap_or(false);
        let flags = if is_expanded {
            TreeNodeFlags::DEFAULT_OPEN
        } else {
            TreeNodeFlags::empty()
        };
        
        let is_open = ui.tree_node_config(&node.name).flags(flags).push();
        if is_open {
            state.expanded_dirs.insert(node.path.clone(), true);
            for child in &node.children {
                render_file_tree(ui, child, state);
            }
            ui.tree_pop();
        } else {
            state.expanded_dirs.insert(node.path.clone(), false);
        }
    } else {
        // File node - make it draggable
        let file_icon = get_file_icon(&node.path);
        ui.text(file_icon);
        ui.same_line();
        
        ui.selectable(&node.name);
        
        // Make draggable if it's a supported file type
        if is_draggable_file(&node.path) {
            if ui.drag_drop_source_config("ASSET_FILE")
                .condition(Condition::Once)
                .begin()
                .is_some()
            {
                // Calculate relative path from asset root
                let relative_path = node.path.strip_prefix(&state.asset_root)
                    .unwrap_or(&node.path)
                    .to_string_lossy()
                    .to_string();
                
                state.dragged_file = Some(relative_path);
                ui.text(&format!("📄 {}", node.name));
            }
        }
    }
}

fn is_draggable_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("obj") | Some("rhai") | Some("json")
    )
}

fn get_file_icon(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("obj") => "🗿",
        Some("rhai") => "📜",
        Some("json") => "📋",
        _ => "📄",
    }
}
```

### Phase 3: Inspector Integration
```rust
// Modify inspector.rs mesh component section:
if has_mesh && ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
    shared_state.with_world_write(|world| {
        if let Ok(mut mesh_id) = world.inner_mut().remove_one::<MeshId>(entity) {
            let mut mesh_name = mesh_id.0.clone();
            let input_changed = ui.input_text("Mesh ID", &mut mesh_name)
                .hint("e.g. cube, sphere, or path/to/model.obj")
                .build();
                
            // Add drop target
            let mut drop_accepted = false;
            if let Some(target) = ui.drag_drop_target() {
                // Visual feedback when hovering
                if ui.is_item_hovered() {
                    ui.get_window_draw_list()
                        .add_rect(
                            ui.item_rect_min(),
                            ui.item_rect_max(),
                            [0.0, 1.0, 0.0, 0.5],
                        )
                        .build();
                }
                
                if target.accept_payload_empty("ASSET_FILE", DragDropFlags::empty()).is_some() {
                    // Get dragged file from asset browser state
                    if let Some(state) = unsafe { ASSET_BROWSER_STATE.as_mut() } {
                        if let Some(file_path) = state.dragged_file.take() {
                            if file_path.ends_with(".obj") {
                                mesh_name = format!("game/assets/{}", file_path);
                                drop_accepted = true;
                            }
                        }
                    }
                }
                target.pop();
            }
            
            if input_changed || drop_accepted {
                mesh_id.0 = mesh_name;
                shared_state.mark_scene_modified();
                debug!(entity = ?entity, mesh = %mesh_id.0, "Modified mesh");
            }
            let _ = world.insert_one(entity, mesh_id);
        }
    });
}

// Similar modification for Script component...
```

### Phase 4: Path Validation
```rust
fn validate_asset_path(path: &str) -> bool {
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
```

## Task List (In Order)

1. **Add file system scanning dependency**
   - No external crate needed, use std::fs

2. **Create asset browser state structure**
   - File: `editor/src/panels/assets.rs`
   - Add FileNode and AssetBrowserState structs
   - Implement directory scanning with filtering

3. **Implement file tree display**
   - Render directories as collapsible tree nodes
   - Render files with appropriate icons
   - Handle expand/collapse state

4. **Add drag source to files**
   - Make file items draggable
   - Store relative path in state
   - Show drag preview

5. **Modify mesh component in inspector**
   - File: `editor/src/panels/inspector.rs`
   - Add drop target to mesh input field
   - Handle .obj file drops
   - Update mesh_id on successful drop

6. **Modify script component in inspector**
   - Add drop target to script input field
   - Handle .rhai file drops
   - Update script name on successful drop

7. **Add path validation**
   - Validate dropped paths
   - Ensure paths are relative to asset root
   - Prevent directory traversal

8. **Add visual feedback**
   - Highlight drop targets when dragging
   - Show file type icons
   - Display tooltips

## Validation Gates

```bash
# Rust syntax and style checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
cargo test --workspace

# Build with editor feature
cargo build -p game --features editor

# Full preflight check (from justfile)
just preflight
```

## Error Handling Strategy

1. **File System Errors**
   - Log errors but don't crash
   - Show empty tree if scan fails
   - Fallback to manual path entry

2. **Invalid Paths**
   - Validate before accepting drops
   - Show error in UI for invalid paths
   - Keep previous value on failure

3. **Missing Files**
   - Check file existence before accepting
   - Show warning if file deleted after scan
   - Refresh tree on focus

## Potential Gotchas

1. **imgui-rs Drag-Drop Lifetime**
   - Use empty payload pattern with state storage
   - Clear dragged_file on drop or frame end

2. **Path Separators**
   - Normalize to forward slashes for consistency
   - Handle both Windows and Unix paths

3. **Large Directories**
   - Consider lazy loading for performance
   - Limit tree depth if needed

4. **Concurrent Modifications**
   - File system may change during runtime
   - Add refresh button or auto-refresh

## Testing Approach

1. Create test assets in game/assets:
   - models/test.obj
   - scripts/test.rhai
   - scenes/test.json

2. Manual testing:
   - Drag .obj to mesh component
   - Drag .rhai to script component
   - Verify path validation
   - Test cancel operations

3. Edge cases:
   - Empty directories
   - Files without extensions
   - Special characters in names

## Confidence Score: 8/10

High confidence due to:
- Clear existing patterns to follow
- Well-documented imgui-rs drag-drop API
- Isolated changes to specific files
- Good validation infrastructure

Minor uncertainties:
- Asset browser state management approach
- Performance with large asset directories