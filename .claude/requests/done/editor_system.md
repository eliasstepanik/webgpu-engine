## FEATURE:

**ImGui-based Scene Editor with Viewport Integration**

Implement a complete ImGui-based scene editor as described in PLANNING.md. The editor should be a separate `editor` library crate that's only compiled in dev builds via feature flags. When the editor is enabled, the game renders into a movable, dockable ImGui viewport window instead of directly to the main window surface.

Key requirements:
* Create `editor` library crate with ImGui integration
* Render game content into an ImGui viewport window (movable/dockable)
* Provide scene hierarchy panel for entity inspection/editing
* Add component inspector for modifying entity properties
* Include asset browser for loading/managing scenes and meshes
* Support Tab key to toggle between editor UI and game input modes
* Feature-flag the editor so release builds exclude it entirely
* Integrate with existing ECS, renderer, and scene loading systems

## EXAMPLES:

```rust
// Editor integration in main.rs when editor feature is enabled
#[cfg(feature = "editor")]
use editor::EditorState;

fn main() {
    // ... existing window/renderer setup ...

    #[cfg(feature = "editor")]
    let mut editor = EditorState::new(&context, &window);

    let _ = event_loop.run(move |event, elwt| {
        #[cfg(feature = "editor")]
        if editor.handle_event(&event) {
            return; // Editor consumed the event
        }

        match event {
            // ... existing window events ...
            WindowEvent::RedrawRequested => {
                #[cfg(feature = "editor")]
                {
                    // Render to editor viewport instead of direct surface
                    editor.begin_frame();
                    editor.render_viewport(&mut renderer, &world);
                    editor.render_ui(&mut world);
                    editor.end_frame();
                }
                #[cfg(not(feature = "editor"))]
                {
                    // Direct rendering for release builds
                    renderer.render(&world);
                }
            }
        }
    });
}
```

```rust
// Editor UI panels
impl EditorState {
    fn render_ui(&mut self, world: &mut World) {
        // Main menu bar
        ui.main_menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("New Scene") { self.new_scene(world); }
                if ui.menu_item("Load Scene...") { self.load_scene_dialog(); }
                if ui.menu_item("Save Scene...") { self.save_scene_dialog(world); }
            });
        });

        // Dockspace for all panels
        ui.dockspace_over_main_viewport();

        // Scene hierarchy panel
        ui.window("Scene Hierarchy")
            .resizable(true)
            .build(|| {
                self.render_entity_tree(world);
            });

        // Properties inspector
        ui.window("Inspector")
            .resizable(true)
            .build(|| {
                if let Some(entity) = self.selected_entity {
                    self.render_component_inspector(world, entity);
                }
            });

        // Viewport window (movable/dockable)
        ui.window("Viewport")
            .resizable(true)
            .build(|| {
                let size = ui.content_region_avail();
                if let Some(texture_id) = self.viewport_texture {
                    ui.image(texture_id, size);
                }
            });

        // Asset browser
        ui.window("Assets")
            .resizable(true)
            .build(|| {
                self.render_asset_browser();
            });
    }
}
```

```toml
# Cargo.toml workspace configuration
[workspace]
members = ["engine", "editor", "game"]

# game/Cargo.toml
[features]
default = ["editor"]
editor = ["dep:editor"]

[dependencies]
engine = { path = "../engine" }
editor = { path = "../editor", optional = true }

# editor/Cargo.toml
[dependencies]
engine = { path = "../engine" }
imgui = "0.12"
imgui-wgpu = "0.24"
imgui-winit-support = "0.12"
```

```rust
// Example component inspector
fn render_transform_inspector(ui: &Ui, transform: &mut Transform) {
    ui.text("Transform");
    ui.separator();
    
    let mut pos = [transform.position.x, transform.position.y, transform.position.z];
    if ui.input_float3("Position", &mut pos) {
        transform.position = Vec3::from_array(pos);
    }
    
    let mut euler = transform.rotation.to_euler(EulerRot::XYZ);
    let mut euler_deg = [
        euler.0.to_degrees(),
        euler.1.to_degrees(), 
        euler.2.to_degrees()
    ];
    if ui.input_float3("Rotation", &mut euler_deg) {
        let euler_rad = [
            euler_deg[0].to_radians(),
            euler_deg[1].to_radians(),
            euler_deg[2].to_radians()
        ];
        transform.rotation = Quat::from_euler(EulerRot::XYZ, euler_rad[0], euler_rad[1], euler_rad[2]);
    }
    
    let mut scale = [transform.scale.x, transform.scale.y, transform.scale.z];
    if ui.input_float3("Scale", &mut scale) {
        transform.scale = Vec3::from_array(scale);
    }
}
```

## DOCUMENTATION:

* imgui-rs documentation: https://docs.rs/imgui/latest/imgui/
* imgui-wgpu integration: https://docs.rs/imgui-wgpu/latest/imgui_wgpu/
* imgui-winit support: https://docs.rs/imgui-winit-support/latest/imgui_winit_support/
* ImGui docking: https://github.com/ocornut/imgui/wiki/Docking
* Dear ImGui demo: https://github.com/ocornut/imgui/blob/master/imgui_demo.cpp
* Cargo features guide: https://doc.rust-lang.org/cargo/reference/features.html
* WebGPU render to texture: https://sotrh.github.io/learn-wgpu/intermediate/tutorial12-camera/

## OTHER CONSIDERATIONS:

* **Architecture**: Follow PLANNING.md structure - create separate `editor` crate as library
* **Feature Flags**: Editor must be feature-gated so release builds exclude it entirely
* **Build Integration**: `just run` should enable editor by default, `cargo build --release` should exclude it
* **Viewport Rendering**: Game must render to a texture that's displayed in ImGui viewport window
* **Input Handling**: Tab key toggles between editor UI mode and game input mode (as per PLANNING.md)
* **Docking**: All ImGui windows should be dockable - use `dockspace_over_main_viewport()`
* **Entity Selection**: Clicking on entities in hierarchy should select them for inspection
* **Component Editing**: Support editing Transform, Camera, Material, and MeshId components
* **Scene Management**: Integrate with existing scene loading/saving system in `engine/src/io/`
* **Asset Integration**: Use existing mesh library and asset management systems
* **Hot Reload**: Consider integrating with existing hot-reload system for assets
* **State Management**: Editor state should be persistent across frames but not saved to disk
* **Error Handling**: Editor errors should not crash the application - log and recover gracefully
* **Performance**: Editor should not significantly impact game performance when disabled
* **UI Layout**: Use sensible default panel layout that's user-configurable
* **Component Registry**: Leverage existing component registry for dynamic component editing
* **Memory Management**: Properly manage ImGui textures and GPU resources
* **Thread Safety**: Ensure editor integration works with existing single-threaded architecture
* **Validation**: Validate component edits before applying to ensure scene integrity
* **Undo/Redo**: Consider basic undo/redo for component modifications (future enhancement)
* **Gizmos**: Consider 3D transform gizmos for visual editing (future enhancement)
* **Debug Info**: Show useful debug information like FPS, entity counts, GPU stats