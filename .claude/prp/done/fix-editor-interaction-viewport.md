# Fix Editor Interaction and Viewport Issues PRP

## Executive Summary
Fix three critical issues in the editor:
1. Editor input handling stops working after switching to editor mode
2. Viewport window is not visible
3. Window resizing before first mode switch causes crash

## Context and Problem Analysis

### Issue 1: Input Handling Failure
**Symptom**: After pressing Tab to switch to editor mode, input works briefly then stops.
**Root Cause**: The imgui event handling state is not properly maintained between frames.

### Issue 2: Missing Viewport Window
**Symptom**: The viewport window that should show the game render is not visible.
**Root Cause**: `render_viewport()` is called before `begin_frame()`, violating imgui's frame lifecycle.

### Issue 3: Resize Crash
**Symptom**: Resizing window before first mode switch causes crash when switching modes later.
**Error**: "Surface outdated, reconfiguring" followed by crash.
**Root Cause**: Texture ID management issue when recreating imgui renderer - trying to remove texture from wrong renderer instance.

## Implementation Blueprint

### Phase 1: Fix the Resize Crash (CRITICAL)

The crash occurs because `editor_state.resize()` recreates the imgui renderer but tries to remove the old texture ID from the OLD renderer before creating the new one. This causes invalid state.

**File**: `/editor/src/editor_state.rs`

1. Fix the resize method to handle texture lifecycle correctly:

```rust
pub fn resize(&mut self, render_context: &RenderContext, new_size: winit::dpi::PhysicalSize<u32>) {
    // ... existing size calculations ...
    
    // Store the old texture before recreating renderer
    let old_texture_exists = self.imgui_renderer.textures.get(self.texture_id).is_some();
    
    // Only remove if it exists in current renderer
    if old_texture_exists {
        self.imgui_renderer.textures.remove(self.texture_id);
    }
    
    // Create new renderer
    self.imgui_renderer = Renderer::new(...);
    
    // Create new render target
    self.render_target = RenderTarget::new(...);
    
    // Register new texture with new renderer
    let imgui_texture = imgui_wgpu::Texture::from_raw_parts(...);
    self.texture_id = self.imgui_renderer.textures.insert(imgui_texture);
}
```

### Phase 2: Fix Render Order for Viewport

The viewport rendering happens before imgui frame begins, which is incorrect.

**File**: `/game/src/main.rs`

1. Move viewport rendering to after begin_frame:

```rust
#[cfg(feature = "editor")]
{
    // Begin editor frame FIRST
    editor_state.begin_frame(&window, &render_context);
    
    // Get surface texture for final rendering
    let surface_texture = match render_context.surface.lock().unwrap().get_current_texture() {
        // ... existing error handling ...
    };
    
    // Now render game to viewport texture (after imgui frame started)
    editor_state.render_viewport(&mut renderer, &world);
    
    // ... rest of rendering ...
}
```

### Phase 3: Fix Input Handling

The input handling fails because imgui's want_capture state isn't properly checked.

**File**: `/editor/src/editor_state.rs`

1. Update handle_event to ensure proper event consumption:

```rust
pub fn handle_event(&mut self, window: &winit::window::Window, event: &Event<()>) -> bool {
    // ... existing Tab key handling ...
    
    // If in UI mode, let imgui handle ALL events
    if self.ui_mode {
        // Process the event
        self.imgui_platform.handle_event(self.imgui_context.io_mut(), window, event);
        
        // Always consume events in UI mode
        // Don't rely on want_capture flags as they may not be set correctly yet
        return true;
    }
    
    false
}
```

### Phase 4: Ensure Proper Initial State

Add initialization to match state after first mode switch.

**File**: `/editor/src/editor_state.rs`

1. In the `new()` method, add after creating platform:

```rust
// Force initial event processing to ensure imgui is properly initialized
self.imgui_platform.prepare_frame(self.imgui_context.io_mut(), window)
    .expect("Initial frame preparation failed");
```

### Phase 5: Add Comprehensive Logging

Add debug logging throughout to track state changes:

```rust
// In handle_event
debug!("Editor handle_event: ui_mode={}, event={:?}", self.ui_mode, event);

// In begin_frame
debug!("Begin frame: surface_size={:?}, display_size={:?}", surface_size, logical_size);

// In resize
debug!("Editor resize: old_texture_exists={}, new_size={:?}", old_texture_exists, new_size);
```

## Validation Gates

```bash
# Build with editor feature
cargo build --release --features editor

# Run preflight checks
just preflight

# Manual testing checklist:
# 1. Start application
# 2. Resize window BEFORE switching modes - should not crash
# 3. Press Tab to switch to editor mode
# 4. Verify viewport window is visible
# 5. Click on UI elements - should respond
# 6. Type in UI - should accept input
# 7. Press Tab to switch back to game mode
# 8. Verify game receives input
# 9. Resize window at any point - should work without black areas
```

## Error Handling Strategy

1. All texture operations wrapped in existence checks
2. Surface outdated errors handled gracefully with proper state updates
3. Imgui frame operations validated before use
4. Clear error messages for debugging

## Known Gotchas

1. imgui-rs requires specific event/frame lifecycle order
2. Texture IDs are tied to specific renderer instances
3. Surface reconfiguration must update all dependent state
4. Event consumption in imgui is not always reflected in want_capture flags immediately

## References

- https://github.com/imgui-rs/imgui-winit-support - Event handling documentation
- https://docs.rs/imgui-winit-support/latest/imgui_winit_support/ - Platform integration
- https://github.com/ocornut/imgui/issues/7873 - WebGPU resize crash issues
- https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/ - WGPU surface handling

## Task List (in order)

1. [ ] Fix resize crash by updating texture ID management in `editor_state.rs`
2. [ ] Fix render order by moving viewport rendering after begin_frame in `main.rs`  
3. [ ] Fix input handling by ensuring events are consumed in UI mode
4. [ ] Add initialization in new() to match post-mode-switch state
5. [ ] Add comprehensive debug logging
6. [ ] Run validation gates
7. [ ] Test all scenarios in manual testing checklist

## Success Criteria

- No crashes when resizing at any time
- Viewport window visible and showing game scene
- Input handling works reliably in both modes
- No black areas when resizing
- All preflight checks pass

## Confidence Score: 8/10

The issues are well-understood with clear root causes identified. The implementation plan addresses each issue systematically. The main risk is potential additional edge cases in imgui-wgpu integration, but the comprehensive logging should help identify any remaining issues.