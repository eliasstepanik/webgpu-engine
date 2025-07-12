# ImGui-based Scene Editor with Viewport Integration PRP

## Objective
Implement a complete ImGui-based scene editor as described in PLANNING.md. Create a separate `editor` library crate that's only compiled in dev builds via feature flags. When enabled, the game renders into a movable, dockable ImGui viewport window instead of directly to the main window surface. Provide comprehensive scene editing capabilities including hierarchy management, component inspection, and asset browsing.

## Codebase Research Findings

### Current Workspace Structure
- **Root Workspace**: Contains `game` and `engine` crates (no `editor` crate yet)
- **Dependency Patterns**: Both crates share `winit = "0.30"` and `tracing = "0.1"`
- **No Feature Flags**: No custom features defined yet, only `#[cfg(debug_assertions)]`
- **Build Commands**: `justfile` with `preflight`, `build`, `run` commands
- **File**: `/mnt/c/Users/elias/RustroverProjects/webgpu-template/Cargo.toml`

### Renderer and Graphics Systems Analysis
- **RenderContext**: Manages WebGPU device, queue, surface (`engine/src/graphics/context.rs`)
- **Renderer**: Orchestrates rendering with `render()` and `render_world()` methods (`engine/src/graphics/renderer.rs`)
- **Current Limitation**: Hardcoded to render to surface texture only
- **Render Target Pattern**: `DepthTexture` shows texture creation pattern with proper usage flags
- **Integration Point**: Need to abstract render target to support render-to-texture

### ECS Patterns and Component Systems
- **World Wrapper**: `hecs::World` wrapper with helper methods (`engine/src/core/entity/world.rs`)
- **Component Registry**: Dynamic component deserialization system (`engine/src/io/component_registry.rs`)
- **Query Patterns**: Comprehensive query system with mutable/immutable access
- **Transform Hierarchy**: Breadth-first update system with cycle detection (`engine/src/core/entity/hierarchy.rs`)
- **Testing Patterns**: Colocated tests in `#[cfg(test)]` modules

### Scene Loading and IO Systems
- **Scene Serialization**: JSON-based with component registry (`engine/src/io/scene.rs`)
- **Asset Management**: Validation and fallback systems (`engine/src/graphics/asset_manager.rs`)
- **Hot Reload**: File watching with debouncing (`engine/src/io/hot_reload.rs`)
- **World Helper Methods**: `save_scene()`, `load_scene()`, `assign_default_meshes()`
- **Validation**: Comprehensive asset validation with detailed error reports

### Component Definitions
- **Transform**: Local position, rotation, scale with serialization support
- **GlobalTransform**: World-space matrix calculated by hierarchy system
- **Camera**: Perspective/orthographic projection with aspect ratio handling
- **Material**: Surface properties (currently color only)
- **MeshId**: String-based mesh reference with asset management integration

## External Documentation

### ImGui Integration Libraries
- **imgui-rs**: https://docs.rs/imgui/latest/imgui/ - Core ImGui API with builder patterns
- **imgui-wgpu**: https://docs.rs/imgui-wgpu/latest/imgui_wgpu/ - WebGPU render backend with texture support
- **imgui-winit-support**: https://docs.rs/imgui-winit-support/latest/imgui_winit_support/ - Window event integration

### ImGui Docking System
- **Docking Wiki**: https://github.com/ocornut/imgui/wiki/Docking
- **Configuration**: Enable with `ImGuiConfigFlags_DockingEnable`
- **DockSpace**: Use for flexible UI layouts with dockable panels

### Cargo Features
- **Features Guide**: https://doc.rust-lang.org/cargo/reference/features.html
- **Best Practice**: Features should be additive, use `optional = true` for dependencies
- **Dev Features**: `#[cfg(feature = "editor")]` for conditional compilation

## Implementation Blueprint

### 1. Workspace and Crate Setup
```toml
# Root Cargo.toml
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

### 2. Render-to-Texture Implementation
```rust
// engine/src/graphics/render_target.rs
pub struct RenderTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub size: (u32, u32),
}

impl RenderTarget {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Editor Viewport Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Self { texture, view, format, size: (width, height) }
    }
}

// Modify Renderer to support custom render targets
impl<'window> Renderer<'window> {
    pub fn render_to_target(&mut self, world: &World, render_target: &RenderTarget) -> Result<(), wgpu::SurfaceError> {
        // Similar to render() but use render_target.view instead of surface texture
    }
}
```

### 3. Editor Crate Structure
```rust
// editor/src/lib.rs
pub mod editor_state;
pub mod panels;
pub mod integration;

pub use editor_state::EditorState;

// editor/src/editor_state.rs
pub struct EditorState {
    imgui_context: imgui::Context,
    imgui_platform: imgui_winit_support::WinitPlatform,
    imgui_renderer: imgui_wgpu::Renderer,
    render_target: RenderTarget,
    selected_entity: Option<hecs::Entity>,
    ui_mode: bool, // true = editor UI, false = game input
}

impl EditorState {
    pub fn new(render_context: &RenderContext, window: &winit::window::Window) -> Self {
        // Initialize ImGui context and platform
        // Create render target for viewport
        // Setup docking configuration
    }
    
    pub fn handle_event(&mut self, event: &winit::event::Event<()>) -> bool {
        // Return true if event was consumed by editor
        // Handle Tab key for input mode toggle
    }
    
    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        // Prepare ImGui frame
    }
    
    pub fn render_viewport(&mut self, renderer: &mut Renderer, world: &World) {
        // Render game to render target texture
    }
    
    pub fn render_ui(&mut self, world: &mut World) {
        // Render all editor panels
    }
    
    pub fn end_frame(&mut self, render_context: &RenderContext) {
        // Submit ImGui render commands
    }
}
```

### 4. Editor Panels Implementation
```rust
// editor/src/panels/hierarchy.rs
pub fn render_hierarchy_panel(ui: &imgui::Ui, world: &World, selected_entity: &mut Option<hecs::Entity>) {
    ui.window("Scene Hierarchy")
        .resizable(true)
        .build(|| {
            // Query all entities, show hierarchical tree
            // Handle entity selection clicks
            // Show entity names (from Name component or entity ID)
        });
}

// editor/src/panels/inspector.rs
pub fn render_inspector_panel(ui: &imgui::Ui, world: &mut World, selected_entity: Option<hecs::Entity>) {
    ui.window("Inspector")
        .resizable(true)
        .build(|| {
            if let Some(entity) = selected_entity {
                // Use component registry to discover available components
                // Render component editors dynamically
                render_transform_inspector(ui, world, entity);
                render_camera_inspector(ui, world, entity);
                render_material_inspector(ui, world, entity);
            }
        });
}

// editor/src/panels/viewport.rs
pub fn render_viewport_panel(ui: &imgui::Ui, texture_id: imgui::TextureId) {
    ui.window("Viewport")
        .resizable(true)
        .build(|| {
            let size = ui.content_region_avail();
            ui.image(texture_id, size);
        });
}

// editor/src/panels/assets.rs
pub fn render_assets_panel(ui: &imgui::Ui, world: &mut World) {
    ui.window("Assets")
        .resizable(true)
        .build(|| {
            // Show available scenes, meshes, materials
            // Implement drag-and-drop or double-click to instantiate
            // Show scene save/load buttons
        });
}
```

### 5. Main.rs Integration
```rust
// game/src/main.rs
#[cfg(feature = "editor")]
use editor::EditorState;

fn main() {
    // ... existing setup ...
    
    #[cfg(feature = "editor")]
    let mut editor = EditorState::new(&render_context, &window);
    
    let _ = event_loop.run(move |event, elwt| {
        #[cfg(feature = "editor")]
        if editor.handle_event(&event) {
            return; // Editor consumed the event
        }
        
        match event {
            WindowEvent::RedrawRequested => {
                #[cfg(feature = "editor")]
                {
                    editor.begin_frame(&window);
                    editor.render_viewport(&mut renderer, &world);
                    editor.render_ui(&mut world);
                    editor.end_frame(&render_context);
                }
                #[cfg(not(feature = "editor"))]
                {
                    // Direct rendering for release builds
                    match renderer.render(&world) {
                        // ... existing error handling
                    }
                }
            }
            // ... other events
        }
    });
}
```

## Implementation Tasks (In Order)

1. **Setup Workspace Structure**
   - Create `editor` crate directory and `Cargo.toml`
   - Update root workspace `Cargo.toml` to include editor member
   - Add feature flags to `game/Cargo.toml` with optional editor dependency

2. **Implement Render-to-Texture**
   - Create `RenderTarget` struct in `engine/src/graphics/render_target.rs`
   - Modify `Renderer` to support custom render targets via `render_to_target()` method
   - Update graphics module exports to include new types

3. **Create Editor Foundation**
   - Initialize editor crate with ImGui integration dependencies
   - Implement `EditorState` with ImGui context, platform, and renderer
   - Add render target creation for viewport texture
   - Setup docking configuration (`ImGuiConfigFlags_DockingEnable`)

4. **Implement Core Editor Panels**
   - Create hierarchy panel with entity tree view and selection
   - Implement inspector panel with dynamic component editing
   - Add viewport panel for displaying game render target
   - Create assets panel for scene and mesh management

5. **Add Component Inspectors**
   - Transform inspector with position/rotation/scale editing
   - Camera inspector for projection parameters
   - Material inspector for color and properties
   - MeshId selector using asset manager

6. **Integrate Scene Management**
   - Scene save/load functionality using existing IO systems
   - New scene creation with default entities
   - Asset validation and error reporting in UI

7. **Add Input Mode Handling**
   - Tab key toggle between editor UI and game input modes
   - Event routing based on current mode
   - Mouse capture/release for different modes

8. **Update Main Integration**
   - Feature-gated editor initialization in `main.rs`
   - Event handling with editor priority
   - Conditional rendering based on editor feature

9. **Add Build Configuration**
   - Update `justfile` to support editor feature
   - Ensure release builds exclude editor by default
   - Add development convenience commands

10. **Write Comprehensive Tests**
    - Editor state initialization and cleanup
    - Render target creation and management
    - Component inspector functionality
    - Scene save/load with editor integration

11. **Documentation and Polish**
    - Document all public APIs with examples
    - Add usage examples for editor integration
    - Performance profiling and optimization

## Validation Gates

```bash
# Format check
cargo fmt --all -- --check

# Lint check with all warnings as errors (with editor feature)
cargo clippy --workspace --all-targets --features editor -- -D warnings

# Lint check without editor feature
cargo clippy --workspace --all-targets -- -D warnings

# Run all tests with editor feature
cargo test --workspace --features editor

# Run all tests without editor feature
cargo test --workspace

# Build with editor feature (dev mode)
cargo build --features editor

# Build without editor feature (release mode)
cargo build --release

# Build documentation
cargo doc --workspace --features editor --no-deps --document-private-items

# Full preflight check
just preflight

# Test editor integration
cargo run --features editor
```

## Critical Implementation Notes

### Feature Flag Integration
1. **Conditional Compilation**: Use `#[cfg(feature = "editor")]` for all editor-specific code
2. **Optional Dependencies**: Mark editor crate as `optional = true` in game dependencies
3. **Default Features**: Include editor in default features for development convenience
4. **Release Builds**: Ensure `cargo build --release` excludes editor automatically

### Render-to-Texture Implementation
1. **Texture Format**: Use `wgpu::TextureFormat::Rgba8UnormSrgb` for consistency with surface
2. **Usage Flags**: Include both `RENDER_ATTACHMENT` and `TEXTURE_BINDING` for ImGui display
3. **Size Management**: Handle viewport resize by recreating render target texture
4. **Pipeline Compatibility**: Ensure render target format matches pipeline configuration

### ImGui Integration Gotchas
1. **Context Lifetime**: ImGui context must live for entire application lifetime
2. **Event Handling Order**: Process ImGui events before game input events
3. **Docking Setup**: Enable docking flag during context creation, not after
4. **Texture Registration**: Register render target texture with ImGui renderer for display

### ECS Integration
1. **Entity Selection**: Store selected entity in editor state, validate existence each frame
2. **Component Editing**: Use existing component registry for dynamic editing
3. **Query Safety**: Collect entities before modification to avoid borrow conflicts
4. **Validation**: Validate component edits before applying to ensure scene integrity

### Performance Considerations
1. **Feature Flag Overhead**: Zero-cost abstraction when editor disabled via feature flags
2. **Render Target Size**: Only create render target texture when editor viewport is visible
3. **UI Updates**: Only rebuild UI when necessary, cache static elements
4. **Asset Loading**: Use existing asset management for efficient resource handling

### Error Handling
1. **Editor Isolation**: Editor errors should never crash the main application
2. **Graceful Degradation**: Fall back to direct rendering if editor initialization fails
3. **User Feedback**: Display error messages in editor UI rather than console
4. **Asset Validation**: Use existing validation systems for scene integrity

### Integration with Existing Systems
1. **Scene Loading**: Leverage existing `Scene::load_from_file()` and `World::load_scene()`
2. **Asset Management**: Use `AssetManager` for validation and `MeshLibrary` for defaults
3. **Component Registry**: Extend existing registry for dynamic component discovery
4. **Hot Reload**: Integrate with existing `SceneWatcher` for asset hot-reloading

## Success Criteria
- Editor crate builds successfully with feature flags
- Game renders to ImGui viewport window (movable/dockable)
- Scene hierarchy displays all entities with selection capability
- Component inspector shows and edits Transform, Camera, Material properties
- Scene save/load works through editor UI
- Tab key toggles between editor UI and game input modes
- Release builds exclude editor entirely with no overhead
- All tests pass with and without editor feature
- Documentation builds successfully
- No performance impact when editor disabled

## Confidence Score: 8/10
Very high confidence for one-pass implementation. All integration patterns are well-researched, existing codebase provides excellent foundations, and external libraries are mature and well-documented. The only uncertainty is in fine-tuning ImGui integration details and ensuring seamless event handling between systems. The modular architecture and comprehensive research provide a clear implementation path with minimal risk.