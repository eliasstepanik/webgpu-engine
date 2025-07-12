## FEATURE:

Implement complete scene management functionality including loading, unloading, and saving scenes through the editor interface.

## EXAMPLES:

The scene management should work similar to Unity or Godot:
- **File Menu Integration**:
  - "New Scene" - Creates a new empty scene (clearing current entities)
  - "Load Scene..." - Opens file dialog to load a .scene file
  - "Save Scene..." - Opens file dialog to save current scene
  - "Save Scene As..." - Save with a new filename
- **Scene State Management**:
  - Track if scene has unsaved changes (dirty flag)
  - Prompt user to save before loading new scene or closing
  - Show current scene name in editor title/status bar
- **File Format**: Use the existing `.scene` JSON format from the IO module

## DOCUMENTATION:

Key existing files to leverage:
- `/engine/src/io/scene.rs` - Scene serialization/deserialization
- `/engine/src/io/component_registry.rs` - Component serialization
- `/engine/src/core/entity/world.rs` - World save_scene/load_scene methods
- `/editor/src/editor_state.rs` - Editor state management
- `/editor/src/panels/mod.rs` - UI panel implementations

Relevant methods already available:
- `World::save_scene()` - Serializes world to Scene struct
- `World::load_scene()` - Loads Scene into world (additive)
- `World::clear()` - Removes all entities
- `Scene::save_to_file()` - Writes scene to disk
- `Scene::load_from_file()` - Reads scene from disk

## OTHER CONSIDERATIONS:

1. **File Dialog Integration**:
   - Need native file dialog for Load/Save operations
   - Consider using `rfd` (Rust File Dialog) crate or similar
   - Filter for .scene files
   - Remember last used directory

2. **Scene State Tracking**:
   - Add `current_scene_path: Option<PathBuf>` to EditorState
   - Add `scene_dirty: bool` flag to track unsaved changes
   - Mark dirty on any entity/component modification

3. **UI Feedback**:
   - Show current scene name in status bar or title
   - Add asterisk (*) to indicate unsaved changes
   - Success/error messages for load/save operations

4. **Error Handling**:
   - Handle missing files gracefully
   - Handle corrupted scene files
   - Show clear error messages to user
   - Maintain current scene if load fails

5. **Entity ID Considerations**:
   - Clear world before loading (non-additive load)
   - Handle entity ID remapping properly
   - Preserve parent-child relationships

6. **Default Scene**:
   - Create sensible default scene with camera and light
   - Option to reset to default scene

7. **Hotkeys**:
   - Ctrl+N - New Scene
   - Ctrl+O - Open Scene
   - Ctrl+S - Save Scene
   - Ctrl+Shift+S - Save Scene As

8. **Future Enhancements**:
   - Recent files menu
   - Auto-save functionality
   - Scene templates
   - Prefab system integration