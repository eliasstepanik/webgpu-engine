## FEATURE:
Implement functional asset browser with drag-and-drop support to automatically load files onto mesh or script components in the entity inspector

## EXAMPLES:
.claude/examples/console-output-before.txt – shows initial script loading errors that demonstrate the need for better asset management
.claude/examples/console-output-after.txt – demonstrates successful script execution after fixes, showing the script system is ready for drag-drop integration

## DOCUMENTATION:
https://docs.rs/imgui/latest/imgui/drag_drop/index.html – imgui-rs drag drop module documentation
https://docs.rs/imgui/latest/imgui/drag_drop/struct.DragDropSource.html – DragDropSource struct for making UI elements draggable
https://docs.rs/imgui/latest/imgui/drag_drop/struct.DragDropTarget.html – DragDropTarget struct for accepting drops
https://github.com/imgui-rs/imgui-rs – imgui-rs repository with examples

## OTHER CONSIDERATIONS:
- Asset browser currently empty stub at editor/src/panels/assets.rs – needs full implementation
- Must scan game/assets directory recursively for .obj, .rhai, and other asset files
- Drag payload should contain file path relative to asset root
- Drop targets in inspector already exist for MeshId and ScriptRef text fields
- Need to handle file system operations safely to prevent path traversal attacks
- AssetConfig in engine/src/config.rs provides asset path validation patterns
- Renderer supports loading meshes from file paths ending in .obj
- Consider visual feedback during drag operations (highlight valid drop targets)
- Empty payload pattern recommended for imgui-rs drag-drop to avoid data lifetime issues