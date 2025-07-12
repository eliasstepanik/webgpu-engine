# Viewport System Debugging Guide

## Common Issues and Solutions

### 1. Scissor Rect Validation Errors

**Problem**: "Scissor Rect is not contained in the render target"

**Root Causes**:
- Viewport size doesn't match render target size
- DPI scaling mismatch
- Coordinate system confusion (logical vs physical pixels)
- Window resize not properly handled

**Solution**:
Use the `safe_scissor` module to automatically clamp scissor rectangles:

```rust
use engine::graphics::safe_scissor::{safe_set_scissor_rect, RenderTargetInfo};

// Instead of:
// render_pass.set_scissor_rect(x, y, w, h);

// Use:
safe_set_scissor_rect(
    &mut render_pass,
    x, y, w, h,
    RenderTargetInfo {
        width: surface_texture.texture.width(),
        height: surface_texture.texture.height(),
    }
);
```

### 2. Main Window Freezes When Clicking Subwindows

**Problem**: Main window becomes unresponsive when subwindows gain focus

**Root Causes**:
- Event loop blocking on single window
- ImGui context not properly shared/updated
- Platform backend not handling focus changes

**Debug Steps**:
1. Add logging to window event handling:
```rust
info!("Window {:?} event: {:?}", window_id, event);
```

2. Check if events are reaching all windows:
```rust
// In handle_event
if window_id == main_window_id {
    debug!("Main window event");
} else {
    debug!("Subwindow {:?} event", window_id);
}
```

3. Verify ImGui platform updates for each window:
```rust
// Each window needs its own platform event handling
imgui_platform.handle_event(imgui_context.io_mut(), window, event);
```

### 3. Subwindow UI Not Responsive

**Problem**: UI elements in detached panels don't respond to clicks/input

**Root Causes**:
- Mouse position not correctly transformed
- Wrong window context for input
- ImGui draw data not properly isolated per viewport

**Debug Steps**:
1. Log mouse positions in each window:
```rust
debug!("Window {:?} mouse pos: {:?}", window_id, mouse_pos);
```

2. Verify viewport IDs match:
```rust
debug!("Viewport {:?} -> Window {:?}", viewport.id, window_id);
```

3. Check draw data is correct for each viewport:
```rust
if let Some(draw_data) = viewport.draw_data() {
    debug!("Draw data for viewport {:?}: display_size={:?}", 
           viewport.id, draw_data.display_size);
}
```

## Debugging Checklist

### Before Rendering Each Frame:
- [ ] Log window sizes (physical and logical)
- [ ] Log DPI scale factors
- [ ] Verify surface configuration matches window size
- [ ] Check viewport-to-window mappings

### During Rendering:
- [ ] Log scissor rect before setting
- [ ] Log render target dimensions
- [ ] Verify draw data dimensions match expectations
- [ ] Check for any size mismatches

### Event Handling:
- [ ] Log all window events with window IDs
- [ ] Verify events reach correct windows
- [ ] Check focus state changes
- [ ] Monitor input event routing

## Key Files to Instrument

1. **viewport_backend.rs**:
   - Add logging to all PlatformViewportBackend trait methods
   - Track window creation/destruction
   - Monitor position/size updates

2. **viewport_renderer_backend.rs**:
   - Log before each render_window call
   - Track surface texture dimensions
   - Monitor draw data

3. **window_manager.rs**:
   - Log window operations (create, resize, close)
   - Track surface reconfigurations
   - Monitor event dispatch

4. **main.rs event loop**:
   - Log event routing decisions
   - Track which windows receive events
   - Monitor render timing

## Performance Considerations

When adding debug logging:
- Use `trace!` for high-frequency events
- Use `debug!` for per-frame logging
- Use `info!` for state changes
- Consider a debug flag to enable/disable viewport debugging

## Testing Scenarios

1. **Single Window Test**: Ensure everything works with just main window
2. **Single Detached Panel**: Test with one panel detached
3. **Multiple Panels**: Test with all panels detached
4. **Rapid Focus Switch**: Click between windows rapidly
5. **Resize Storm**: Resize all windows simultaneously
6. **High DPI**: Test on high-DPI display with scaling

## Emergency Fixes

If the system is completely broken:

1. **Disable Viewports Temporarily**:
```rust
// In EditorState::new
// Comment out: self.init_viewport_backend(...)
```

2. **Force Single Window Mode**:
```rust
// In render_ui_and_draw
// Always use: self.render_single_window(...)
```

3. **Add Scissor Rect Safety**:
```rust
// Wrap ALL set_scissor_rect calls with safe_set_scissor_rect
```