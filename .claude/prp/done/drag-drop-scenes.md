name: "Drag-Drop Scene Loading Implementation"
description: |

## Purpose
Implement drag-and-drop functionality to load scenes into the viewport with save confirmation, and to add models/scenes to entities in the inspector panel.

## Core Principles
1. **Follow Existing Patterns**: Use established drag-drop patterns from assets.rs and inspector.rs
2. **Safety First**: Validate all dropped paths and handle unsaved changes properly
3. **Progressive Implementation**: Viewport first, then inspector additions
4. **Preserve Functionality**: Don't break existing model drag-drop in inspector

---

## Goal
Enable users to drag scene files from the assets panel to:
1. The viewport to load that scene (with unsaved changes confirmation)
2. An entity in the inspector to instantiate the scene's entities as children

## Why
- **Workflow efficiency**: Faster scene switching without menu navigation
- **Intuitive interaction**: Drag-drop is standard in game editors
- **Entity composition**: Easy way to build complex entities from scene templates

## What
Users can drag JSON scene files to the viewport to load them. If there are unsaved changes, a dialog asks whether to save first. Users can also drag scenes or models to the entity inspector to add them to the selected entity.

### Success Criteria
- [x] Scene files can be dragged from assets panel
- [x] Dropping on viewport loads the scene
- [x] Unsaved changes dialog appears when needed
- [x] Dropping scenes on inspector adds as children
- [x] Existing model drag-drop continues working
- [x] All paths are validated for security

## All Needed Context

### Documentation & References
```yaml
- url: https://docs.rs/imgui/latest/imgui/drag_drop/index.html
  why: Core drag-drop API documentation
  
- file: editor/src/panels/assets.rs
  why: Existing drag source implementation pattern (lines 218-237)
  
- file: editor/src/panels/inspector.rs  
  why: Existing drag target pattern for models (lines 160-185)
  
- file: editor/src/panels/viewport.rs
  why: Where to add viewport drag target (after line 138)
  
- file: engine/src/core/entity/world.rs
  why: load_scene() and load_scene_additive() methods
  
- file: editor/src/editor_state.rs
  why: Dialog system and pending actions (lines 20-54, 89-97)
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: AssetBrowserState::take_dragged_file() CONSUMES the dragged file
// It returns Option<String> - None if no file or already consumed

// CRITICAL: Scene files are JSON files specifically in "scenes/" subdirectory
// Not all JSON files are scenes - must check path contains "scenes/"

// CRITICAL: validate_asset_path() must be called on all dropped paths
// It prevents directory traversal attacks and ensures file exists

// CRITICAL: Use EditorSharedState for scene_modified, not EditorState
// shared_state.mark_scene_modified() updates across all windows

// CRITICAL: Parent component requires entity remapping after scene load
// load_scene_additive() returns EntityMapper with new entity IDs

// CRITICAL: Must call update_hierarchy_system after setting Parent
// This ensures GlobalTransform is calculated for new children
```

## Implementation Blueprint

### Helper Functions

```rust
// In assets.rs - identify scene files
fn is_scene_file(path: &Path) -> bool {
    // Check if it's a JSON file in a scenes directory
    if let Some(ext) = path.extension() {
        if ext == "json" {
            // Check if path contains "scenes/" directory
            let path_str = path.to_string_lossy();
            return path_str.contains("scenes/") || path_str.contains("scenes\\");
        }
    }
    false
}
```

### List of tasks to be completed in order

```yaml
Task 1 - Make scene files explicitly draggable:
MODIFY editor/src/panels/assets.rs:
  - FIND function: is_draggable_file (around line 153)
  - ADD scene detection: check if JSON is in scenes/ directory
  - UPDATE get_file_icon to return scene-specific icon for scene files
  
Task 2 - Add viewport drag target:
MODIFY editor/src/panels/viewport.rs:
  - FIND line after: imgui::Image::new(texture_id, available_size).build(ui);
  - ADD drag_drop_target with visual feedback
  - CHECK if dropped file is scene using helper function
  - SET show_unsaved_dialog if scene_modified
  - SET pending_scene_operation with LoadScene(path)
  
Task 3 - Enhance inspector for scene drops:
MODIFY editor/src/panels/inspector.rs:
  - FIND section after MeshId component handling (around line 208)
  - ADD new collapsing header for "Add Child Scene"
  - CREATE drag_drop_target that accepts scenes
  - USE load_scene_additive to instantiate scene
  - SET Parent component on all loaded entities
  
Task 4 - Update shared state handling:
VERIFY editor/src/shared_state.rs usage:
  - ENSURE mark_scene_modified() is called appropriately
  - CHECK scene_modified flag before loading
```

### Task 1: Scene File Detection
```rust
// In is_draggable_file function
fn is_draggable_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("obj") | Some("rhai") => true,
        Some("json") => is_scene_file(path), // Only drag scene JSONs
        _ => false,
    }
}

// In get_file_icon function  
fn get_file_icon(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("obj") => "🗿",
        Some("rhai") => "📜", 
        Some("json") => {
            if is_scene_file(path) {
                "🎬"  // Scene icon
            } else {
                "📋"  // Regular JSON
            }
        }
        _ => "📄",
    }
}
```

### Task 2: Viewport Drag Target
```rust
// After imgui::Image in render_viewport_panel
imgui::Image::new(texture_id, available_size).build(ui);

// Add drag-drop target for scenes
if let Some(target) = ui.drag_drop_target() {
    // Visual feedback on hover
    if ui.is_item_hovered() {
        ui.get_window_draw_list()
            .add_rect(
                [available_size[0] * 0.1, available_size[1] * 0.1],
                [available_size[0] * 0.9, available_size[1] * 0.9],
                [0.0, 1.0, 0.0, 0.3], // Semi-transparent green
            )
            .thickness(3.0)
            .build();
    }
    
    if target.accept_payload_empty("ASSET_FILE", DragDropFlags::empty()).is_some() {
        if let Some(file_path) = crate::panels::assets::AssetBrowserState::take_dragged_file() {
            // Validate and check if it's a scene
            if crate::panels::assets::validate_asset_path(&file_path) {
                let full_path = PathBuf::from("game/assets").join(&file_path);
                if crate::panels::assets::is_scene_file(&full_path) {
                    // Return scene load request to main loop
                    // Main loop will handle unsaved dialog if needed
                    viewport_action = Some(ViewportAction::LoadScene(full_path));
                    debug!("Scene drop accepted: {}", file_path);
                }
            }
        }
    }
    target.pop();
}
```

### Task 3: Inspector Scene Drops
```rust
// After MeshId component section in inspector
if ui.collapsing_header("Add Components", TreeNodeFlags::DEFAULT_OPEN) {
    ui.text("Drop assets here to add:");
    
    // Create a drop zone
    let drop_zone_size = [ui.content_region_avail()[0], 40.0];
    ui.invisible_button("##drop_zone", drop_zone_size);
    
    if let Some(target) = ui.drag_drop_target() {
        // Visual feedback
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
            if let Some(file_path) = crate::panels::assets::AssetBrowserState::take_dragged_file() {
                if crate::panels::assets::validate_asset_path(&file_path) {
                    let full_path = PathBuf::from("game/assets").join(&file_path);
                    
                    // Handle based on file type
                    if file_path.ends_with(".obj") {
                        // Add MeshId component
                        shared_state.with_world_write(|world| {
                            let _ = world.insert_one(entity, MeshId(full_path.to_string_lossy().to_string()));
                        });
                        shared_state.mark_scene_modified();
                    } else if crate::panels::assets::is_scene_file(&full_path) {
                        // Load scene as children
                        shared_state.with_world_write(|world| {
                            match world.load_scene_additive(&full_path) {
                                Ok(mapper) => {
                                    // Set parent for all loaded entities
                                    for (_old_id, new_entity) in mapper.iter() {
                                        let _ = world.insert_one(*new_entity, Parent(entity));
                                    }
                                    // Update hierarchy to calculate transforms
                                    engine::core::entity::hierarchy::update_hierarchy_system(world);
                                    info!("Added {} entities as children", mapper.len());
                                }
                                Err(e) => {
                                    error!("Failed to load scene: {}", e);
                                }
                            }
                        });
                        shared_state.mark_scene_modified();
                    }
                }
            }
        }
        target.pop();
    }
}
```

### Integration Points
```yaml
VIEWPORT:
  - Return type: Change render_viewport_panel to return Option<ViewportAction>
  - Main loop: Handle ViewportAction::LoadScene with dialog check
  
SHARED_STATE:
  - Access pattern: Use shared_state parameter, not direct modification
  - Thread safety: All modifications through provided methods
  
HIERARCHY:
  - After adding Parent: Call update_hierarchy_system
  - Frame advancement: Handled by main render loop
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Format and lint
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Build
```bash
# Full build
cargo build --workspace

# Expected: Successful compilation
```

### Level 3: Manual Testing

1. **Test Scene Drop to Viewport**:
   - Run: `just run`
   - Open assets panel, navigate to game/assets/scenes/
   - Drag simple_test.json to viewport
   - Verify: Scene loads immediately (no unsaved changes)

2. **Test Unsaved Dialog**:
   - Make any change (move an entity)
   - Drag another scene to viewport
   - Verify: Dialog appears with Save/Don't Save/Cancel
   - Test all three options

3. **Test Scene Drop to Inspector**:
   - Select an entity in hierarchy
   - Drag simple_test.json to inspector drop zone
   - Verify: New entities appear as children
   - Check hierarchy panel shows parent-child relationship

4. **Test Model Drop to Inspector**:
   - Drag any .obj file to inspector
   - Verify: MeshId component is added (existing functionality preserved)

## Final Validation Checklist
- [ ] All existing tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets`
- [ ] Scene files show different icon in assets panel
- [ ] Viewport accepts only scene drops
- [ ] Unsaved dialog works correctly
- [ ] Inspector accepts both scenes and models
- [ ] Parent-child relationships are correct
- [ ] No panics or crashes during drag-drop

---

## Anti-Patterns to Avoid
- ❌ Don't accept non-scene JSON files in viewport
- ❌ Don't skip validate_asset_path() - security critical
- ❌ Don't modify scene without setting scene_modified flag
- ❌ Don't forget to call hierarchy update after adding parents
- ❌ Don't create new drag-drop payload types - use "ASSET_FILE"
- ❌ Don't handle dialogs in panels - return to main loop