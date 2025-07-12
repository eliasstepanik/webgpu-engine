# Scene Management Implementation PRP

## Feature Overview
Implement complete scene management functionality including loading, unloading, and saving scenes through the editor interface with file dialogs, keyboard shortcuts, and proper state tracking.

## Critical Context

### Existing Scene IO Infrastructure
The engine already has robust scene serialization in `/engine/src/io/scene.rs`:
```rust
// Scene structure
pub struct Scene {
    pub entities: Vec<SerializedEntity>,
}

// Key methods already available:
Scene::save_to_file(path: &Path) -> Result<(), SceneError>
Scene::load_from_file(path: &Path) -> Result<Scene, SceneError>
Scene::from_world(world: &World) -> Self
Scene::instantiate(world: &mut World, renderer: &mut Renderer)
```

World integration in `/engine/src/core/entity/world.rs`:
```rust
World::save_scene(&self, path: &Path) -> Result<(), SceneError>
World::load_scene(&mut self, path: &Path, renderer: &mut Renderer) -> Result<(), SceneError>
World::clear() // Removes all entities
```

### Editor State Pattern
Current editor state in `/editor/src/editor_state.rs`:
- Menu bar with placeholder scene operations (lines 389-403)
- Keyboard event handling pattern (lines 175-266)
- Status bar implementation (lines 422-447)
- Uses imgui-rs 0.12 for UI

### File Dialog Library
Use `rfd` (Rust File Dialog) version 0.14:
- Documentation: https://docs.rs/rfd/latest/rfd/
- Synchronous API recommended due to winit event loop constraints
- Native file dialogs on all platforms

### Keyboard Handling in winit 0.30
Modifier tracking pattern:
```rust
// Track modifiers state
WindowEvent::ModifiersChanged(new_modifiers) => {
    self.current_modifiers = new_modifiers.state();
}

// Check for shortcuts
if key_event.state == ElementState::Pressed {
    let ctrl_pressed = self.current_modifiers.control_key();
    if ctrl_pressed {
        match key_event.physical_key {
            PhysicalKey::Code(KeyCode::KeyS) => { /* Save */ }
            // etc.
        }
    }
}
```

### ImGui Modal Dialog Pattern
From imgui-rs 0.12 documentation:
```rust
// Confirmation dialog
if self.show_unsaved_dialog {
    ui.open_popup("unsaved_changes");
}

ui.modal_popup("unsaved_changes", || {
    ui.text("Save changes to current scene?");
    if ui.button("Save") {
        // Save and proceed
        ui.close_current_popup();
    }
    ui.same_line();
    if ui.button("Don't Save") {
        // Discard and proceed
        ui.close_current_popup();
    }
    ui.same_line();
    if ui.button("Cancel") {
        // Cancel operation
        ui.close_current_popup();
    }
});
```

## Implementation Blueprint

### Phase 1: Core State Management
1. Update `editor/Cargo.toml`:
```toml
[dependencies]
rfd = "0.14"
```

2. Extend `EditorState` struct:
```rust
pub struct EditorState {
    // ... existing fields ...
    
    // Scene management state
    pub current_scene_path: Option<PathBuf>,
    pub scene_modified: bool,
    pub current_modifiers: winit::event::Modifiers,
    
    // Dialog state
    pub show_unsaved_dialog: bool,
    pub pending_action: Option<PendingAction>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    NewScene,
    LoadScene,
    Exit,
}
```

### Phase 2: Keyboard Shortcuts
Add to `handle_event` method:
```rust
// Track modifier changes
if let Event::WindowEvent {
    event: WindowEvent::ModifiersChanged(new_modifiers),
    ..
} = event
{
    self.current_modifiers = new_modifiers.state();
}

// Check for shortcuts (before Tab key handling)
if let Event::WindowEvent {
    event: WindowEvent::KeyboardInput { event: key_event, .. },
    ..
} = event
{
    if key_event.state == ElementState::Pressed && self.ui_mode {
        let ctrl = self.current_modifiers.control_key();
        let shift = self.current_modifiers.shift_key();
        
        if ctrl {
            match key_event.physical_key {
                PhysicalKey::Code(KeyCode::KeyN) => {
                    self.new_scene_action();
                    return true;
                }
                PhysicalKey::Code(KeyCode::KeyO) => {
                    self.load_scene_action();
                    return true;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    if shift {
                        self.save_scene_as_action();
                    } else {
                        self.save_scene_action();
                    }
                    return true;
                }
                _ => {}
            }
        }
    }
}
```

### Phase 3: Scene Operations
Implement scene operation methods:
```rust
impl EditorState {
    pub fn new_scene_action(&mut self) {
        if self.scene_modified {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::NewScene);
        } else {
            self.perform_new_scene();
        }
    }
    
    pub fn perform_new_scene(&mut self) {
        // Implementation will clear world and reset state
    }
    
    pub fn save_scene_action(&mut self) -> bool {
        if let Some(ref path) = self.current_scene_path {
            self.save_scene_to_path(path)
        } else {
            self.save_scene_as_action()
        }
    }
    
    pub fn save_scene_as_action(&mut self) -> bool {
        if let Some(path) = self.show_save_dialog() {
            self.save_scene_to_path(&path)
        } else {
            false
        }
    }
    
    fn show_save_dialog(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Save Scene")
            .add_filter("Scene files", &["json"])
            .add_filter("All files", &["*"])
            .set_file_name("untitled.json")
            .save_file()
    }
}
```

### Phase 4: UI Integration
Update `render_ui_and_draw` method:

1. Update menu items:
```rust
ui.menu("File", || {
    if ui.menu_item_with_shortcut("New Scene", "Ctrl+N") {
        self.new_scene_action();
    }
    if ui.menu_item_with_shortcut("Load Scene...", "Ctrl+O") {
        self.load_scene_action();
    }
    if ui.menu_item_with_shortcut("Save Scene", "Ctrl+S") {
        self.save_scene_action();
    }
    if ui.menu_item_with_shortcut("Save Scene As...", "Ctrl+Shift+S") {
        self.save_scene_as_action();
    }
    // ... rest of menu
});
```

2. Add unsaved changes dialog:
```rust
// Handle unsaved changes dialog
if self.show_unsaved_dialog {
    ui.open_popup("unsaved_changes");
}

ui.modal_popup("unsaved_changes", || {
    ui.text("Save changes to current scene?");
    ui.spacing();
    
    let mut action_taken = false;
    
    if ui.button("Save") {
        if self.save_scene_action() {
            action_taken = true;
            if let Some(ref action) = self.pending_action {
                match action {
                    PendingAction::NewScene => self.perform_new_scene(),
                    PendingAction::LoadScene => self.perform_load_scene(),
                    PendingAction::Exit => std::process::exit(0),
                }
            }
        }
        ui.close_current_popup();
    }
    
    ui.same_line();
    if ui.button("Don't Save") {
        action_taken = true;
        if let Some(ref action) = self.pending_action {
            match action {
                PendingAction::NewScene => self.perform_new_scene(),
                PendingAction::LoadScene => self.perform_load_scene(),
                PendingAction::Exit => std::process::exit(0),
            }
        }
        ui.close_current_popup();
    }
    
    ui.same_line();
    if ui.button("Cancel") {
        ui.close_current_popup();
    }
    
    if action_taken {
        self.show_unsaved_dialog = false;
        self.pending_action = None;
    }
});
```

3. Update status bar:
```rust
// In status bar window
let scene_name = self.current_scene_path
    .as_ref()
    .and_then(|p| p.file_name())
    .and_then(|n| n.to_str())
    .unwrap_or("Untitled");

ui.text(format!("Scene: {}{}", 
    scene_name,
    if self.scene_modified { "*" } else { "" }
));
```

### Phase 5: Change Tracking
Track modifications in relevant methods:
```rust
// When entities are created/destroyed
pub fn mark_scene_modified(&mut self) {
    self.scene_modified = true;
}

// Hook into entity operations
// - After entity spawn
// - After entity despawn  
// - After component changes
// - After transform updates
```

## Error Handling Strategy

1. **File Operations**: Wrap in Result types and display errors in modal
2. **Invalid Scene Files**: Show error with details, keep current scene
3. **Missing Files**: Show file not found error
4. **Save Failures**: Show error but keep trying

```rust
// Error display modal
if let Some(ref error) = self.error_message {
    ui.open_popup("error");
}

ui.modal_popup("error", || {
    ui.text("Error");
    ui.separator();
    ui.text_wrapped(&self.error_message.as_ref().unwrap());
    if ui.button("OK") {
        self.error_message = None;
        ui.close_current_popup();
    }
});
```

## Implementation Tasks

1. **Add rfd dependency** to editor/Cargo.toml
2. **Extend EditorState** with scene management fields
3. **Implement keyboard shortcut handling** with modifier tracking
4. **Create scene operation methods** (new, load, save, save as)
5. **Add file dialogs** using rfd
6. **Implement unsaved changes dialog**
7. **Update menu items** with shortcuts
8. **Update status bar** with scene info
9. **Add error handling modals**
10. **Implement change tracking** throughout editor
11. **Create default scene** with camera and lighting
12. **Test all file operations** and edge cases

## Validation Gates

```bash
# Check compilation
cargo build --package editor --features editor

# Check formatting and lints
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests (when implemented)
cargo test --package editor --features editor

# Manual testing checklist:
# - [ ] Ctrl+N creates new scene
# - [ ] Ctrl+O opens file dialog and loads scene
# - [ ] Ctrl+S saves current scene
# - [ ] Ctrl+Shift+S opens save as dialog
# - [ ] Unsaved changes prompt appears when needed
# - [ ] Status bar shows scene name and modified indicator
# - [ ] Error messages display in modal dialogs
# - [ ] File operations handle all error cases gracefully
```

## External Resources

- rfd documentation: https://docs.rs/rfd/latest/rfd/
- imgui-rs modal examples: https://github.com/imgui-rs/imgui-rs/blob/main/imgui-examples/examples/modals.rs
- winit keyboard handling: https://docs.rs/winit/latest/winit/event/enum.WindowEvent.html

## Confidence Score: 8/10

The implementation path is clear with existing infrastructure for scene IO. Main complexity is in UI integration and state management. The synchronous file dialog approach avoids async complications. Error handling patterns are straightforward. Minor deduction for potential platform-specific dialog issues and change tracking complexity.