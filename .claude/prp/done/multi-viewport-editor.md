# PRP: Multi-Viewport Editor Implementation

## Overview
Enable multi-viewport support in the editor, allowing users to drag editor panels outside the main window to create detached, independent windows. This provides flexible workspace arrangements across multiple monitors.

## Current State Analysis

### Editor Architecture (editor/src/)
- **Single window** with ImGui panels (hierarchy, inspector, assets, viewport)
- **ImGui 0.12** - no safe viewport/docking API
- **EditorState** manages all panels in `editor/src/editor_state.rs`
- **Modal input system** - Tab toggles between editor/game input

### Window Management (game/src/main.rs)
- **Single Arc<Window>** created at startup
- **Event loop** uses deprecated closure API
- **No window ID tracking** - assumes single window

### Rendering (engine/src/graphics/)
- **RenderContext** assumes single window/surface (`context.rs`)
- **Surface wrapped in Mutex** for thread safety
- **Shared GPU resources** (device, queue) via Arc

## Technical Approach

### Phase 1: ImGui Viewport Support

**Option A: Use imgui-sys directly**
```toml
# editor/Cargo.toml
[dependencies]
imgui = "0.12"
imgui-sys = "0.12"  # Access viewport C API
```

**Option B: Community fork**
```toml
# Replace imgui with fork
imgui = { git = "https://github.com/Ax9D/imgui-docking-rs", branch = "docking" }
```

Research shows imgui-sys provides access to viewport functions via FFI. We'll use Option A to maintain compatibility.

### Phase 2: Window Management Infrastructure

Create `engine/src/windowing/mod.rs`:
```rust
use std::collections::HashMap;
use winit::window::{Window, WindowId};
use wgpu::Surface;

pub struct WindowData {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
}

pub struct WindowManager {
    windows: HashMap<WindowId, WindowData>,
    main_window_id: WindowId,
}
```

### Phase 3: Multi-Window Event Loop

Refactor `game/src/main.rs` to use ApplicationHandler pattern:
```rust
struct App {
    window_manager: WindowManager,
    render_context: RenderContext,
    editor_state: Option<EditorState>,
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        // Route events to correct window
    }
}
```

### Phase 4: Editor Panel Detachment

Modify `editor/src/editor_state.rs`:
```rust
pub struct PanelState {
    pub window_id: Option<WindowId>,  // None = docked
    pub position: [f32; 2],
    pub size: [f32; 2],
}

pub struct EditorState {
    panels: HashMap<PanelId, PanelState>,
    // ... existing fields
}
```

## Implementation Blueprint

### Task Order (CRITICAL - must be done in sequence)

1. **Add viewport feature flag**
   ```toml
   # editor/Cargo.toml
   [features]
   viewport = ["imgui-sys"]
   ```

2. **Create WindowManager module**
   - Path: `engine/src/windowing/window_manager.rs`
   - Update `engine/src/lib.rs` to export module
   - Pattern: Follow `engine/src/graphics/context.rs` structure

3. **Implement viewport platform backend**
   - Path: `editor/src/viewport_backend.rs`
   - Reference: https://github.com/ocornut/imgui/blob/docking/backends/imgui_impl_win32.cpp
   - Implement viewport creation/destruction callbacks

4. **Refactor main event loop**
   - Convert to ApplicationHandler pattern
   - Track windows in HashMap by WindowId
   - Route events based on window_id

5. **Update RenderContext for multi-surface**
   ```rust
   // engine/src/graphics/context.rs
   pub struct RenderContext {
       surfaces: HashMap<WindowId, Mutex<wgpu::Surface<'static>>>,
       // Keep shared resources
   }
   ```

6. **Implement drag detection**
   - Check `imgui.is_mouse_dragging()` on panel title bars
   - Track drag distance threshold
   - Create window on drag-out

7. **Window creation on detach**
   ```rust
   fn create_detached_window(&mut self, panel_id: PanelId, position: [i32; 2]) {
       let window = Arc::new(event_loop.create_window(
           WindowAttributes::default()
               .with_position(PhysicalPosition::new(position[0], position[1]))
               .with_inner_size(PhysicalSize::new(400, 300))
       )?);
   }
   ```

8. **State synchronization**
   - Share EditorState between all windows
   - Use channels for cross-window communication
   - Update selected entity across all inspectors

9. **Window cleanup**
   - Handle WindowEvent::CloseRequested
   - Return panel to main window or close
   - Clean up GPU resources

10. **Persistence**
    - Save window layouts to JSON
    - Restore on startup
    - Path: `editor/layouts/default.json`

## Validation Gates

```bash
# After each phase, run:
just preflight

# Phase 1: ImGui upgrade
cargo check --features editor,viewport

# Phase 2: Window manager
cargo test --package engine --lib windowing::window_manager

# Phase 3: Multi-window rendering
cargo run --features editor -- --test-multi-window

# Phase 4: Full integration
# Manual test: 
# 1. Run editor
# 2. Drag hierarchy panel out
# 3. Verify rendering in both windows
# 4. Close detached window
# 5. Verify panel returns to main
```

## Critical Implementation Details

### Platform-Specific Gotchas

**Windows**: 
- DPI awareness - use `window.scale_factor()` per window
- Multiple monitors may have different DPI

**Linux**:
- X11 vs Wayland - test both
- Window decorations vary by desktop environment

**macOS**:
- Respect spaces/fullscreen behavior
- Window restoration after app restart

### GPU Resource Management

```rust
// Surfaces are per-window but device/queue are shared
let surface = instance.create_surface(Arc::clone(&window))?;

// Configure surface with shared device
surface.configure(&device, &surface_config);
```

### Event Routing Pattern

```rust
match event {
    WindowEvent::RedrawRequested => {
        if let Some(window_data) = window_manager.get_mut(window_id) {
            // Render to specific window
        }
    }
}
```

### ImGui Context Handling

```rust
// Single ImGui context, multiple viewports
let mut context = imgui::Context::create();
unsafe {
    imgui_sys::igGetIO().ConfigFlags |= imgui_sys::ImGuiConfigFlags_ViewportsEnable;
}
```

## Error Handling Strategy

1. **Window creation failure**: Log error, keep panel docked
2. **GPU resource exhaustion**: Limit max windows to 4
3. **Platform limitations**: Graceful fallback to single window
4. **Monitor disconnection**: Move windows to primary monitor

## Performance Considerations

- **Render only visible windows** - check minimized state
- **Share textures** between windows when possible
- **Batch GPU commands** across windows
- **Profile with 4 windows** - target 60 FPS

## Testing Checklist

- [ ] Create/destroy windows repeatedly (no leaks)
- [ ] Drag panels between monitors with different DPI
- [ ] Minimize/maximize/fullscreen behavior
- [ ] Alt-tab between detached windows
- [ ] Save/restore layout persistence
- [ ] Platform testing (Windows, Linux, macOS)

## References

- **ImGui Viewport Documentation**: https://github.com/ocornut/imgui/wiki/Multi-Viewports
- **imgui-sys viewport API**: Check `imgui-sys/src/lib.rs` for viewport functions
- **winit multi-window example**: https://github.com/rust-windowing/winit/blob/master/examples/multithreaded.rs
- **wgpu multi-surface**: https://github.com/gfx-rs/wgpu/discussions/6284
- **Working example**: https://github.com/erer1243/wgpu-0.20-winit-0.30-web-example

## Success Metrics

- 4 detached windows at 60 FPS
- < 100ms window creation time
- Zero memory leaks over 100 create/destroy cycles
- Cross-platform compatibility verified

---

**Confidence Score: 7/10**

Primary risks:
- ImGui viewport API complexity
- Platform-specific window behavior
- State synchronization edge cases

Mitigation: Extensive error handling and platform testing phases built into implementation plan.