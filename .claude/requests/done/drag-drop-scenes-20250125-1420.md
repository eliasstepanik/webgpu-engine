## FEATURE:

Implement drag-and-drop functionality for loading scenes into the viewport and adding models/scenes to entities in the inspector.

## EXAMPLES:

.claude/examples/asset-drag-drop.rs – Shows drag-drop pattern between panels
.claude/examples/scene-loading-with-dialog.rs – Demonstrates scene loading with save confirmation

## DOCUMENTATION:

https://docs.rs/imgui/latest/imgui/drag_drop/index.html
https://github.com/imgui-rs/imgui-rs/issues/drag-drop-viewports
https://docs.rs/imgui/latest/imgui/drag_drop/struct.DragDropTarget.html
https://github.com/ocornut/imgui/issues/5204

## OTHER CONSIDERATIONS:

- JSON files in game/assets/scenes/ are scene files that should be draggable to viewport
- Existing drag-drop implementation in assets.rs uses "ASSET_FILE" payload identifier
- Scene loading already exists via load_scene_from_file() in scene_operations.rs
- Must show unsaved changes dialog before loading new scene (show_unsaved_dialog flag exists)
- Viewport panel (viewport.rs) needs drag_drop_target() added to accept scene drops
- For entity inspector, dragging .json scene should instantiate entities as children
- Models (.obj) already work via MeshId component drag-drop in inspector.rs
- Need to differentiate between scene files and other JSON in assets panel
- AssetBrowserState::take_dragged_file() retrieves the dragged file path
- validate_asset_path() ensures dropped files are safe and within asset root
- SceneOperation enum exists for pending scene operations after dialogs
- Parent component exists for entity hierarchy relationships