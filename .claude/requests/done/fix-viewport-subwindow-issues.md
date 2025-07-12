## FEATURE:

Fix critical viewport/subwindow rendering issues in the ImGui-based editor. The current multi-viewport implementation has severe usability problems:

1. **Main window freezes when clicking other windows** - The main window becomes unresponsive when any subwindow gains focus
2. **Subwindow UI is unusable** - The UI in detached panels doesn't respond properly to input
3. **Persistent scissor rect errors** - "Scissor Rect is not contained in the render target" errors continue to occur
4. **Need a robust scissor rect validation system** - A permanent fix that prevents these errors from ever occurring

## CURRENT ISSUES IN DETAIL:

### 1. Window Focus/Event Handling Issues
- Main window stops processing events when subwindows are focused
- Input events may not be properly routed to the correct window
- Window manager might not be handling multi-window event loops correctly

### 2. Rendering Pipeline Issues
- Scissor rectangles are being set incorrectly for viewport rendering
- DPI scaling might still be causing coordinate mismatches
- Surface sizes and viewport sizes may be out of sync

### 3. ImGui Multi-Viewport Integration
- The viewport backend implementation might have incorrect window management
- Platform/renderer backend coordination could be broken
- Draw data might not be properly isolated per viewport

## EXAMPLES:

### Current Error Pattern:
```
wgpu error: Validation Error
Caused by:
    In a RenderPass
      note: encoder = `<CommandBuffer-(2, 139, Metal)>`
    In a set_scissor_rect command
    Scissor Rect { x: 450, y: 0, w: 450, h: 600 } is not contained in the render target (450, 600, 1)
```

### Expected Behavior:
- All windows should remain responsive regardless of which has focus
- UI in subwindows should be fully interactive
- No scissor rect validation errors should ever occur
- Smooth, seamless multi-window experience

## DOCUMENTATION:

### Key References:
1. **ImGui Multi-Viewport Documentation**: https://github.com/ocornut/imgui/wiki/Multi-Viewports
2. **wgpu Scissor Rect**: https://docs.rs/wgpu/latest/wgpu/struct.RenderPass.html#method.set_scissor_rect
3. **winit Multi-Window**: https://docs.rs/winit/latest/winit/event_loop/index.html
4. **Our ImGui Fork**: Check the specific viewport API implementation in our imgui-rs fork

### Relevant Files:
- `/editor/src/viewport_backend.rs` - Platform viewport backend
- `/editor/src/viewport_renderer_backend.rs` - Renderer viewport backend
- `/editor/src/enhanced_viewport_renderer.rs` - Main viewport renderer
- `/engine/src/windowing/window_manager.rs` - Window management
- `/game/src/main.rs` - Main event loop

## OTHER CONSIDERATIONS:

### Critical Requirements:

1. **Robust Scissor Rect Validation System**:
   - Create a wrapper around `set_scissor_rect` that ALWAYS validates bounds
   - Clamp scissor rectangles to render target bounds before setting
   - Add debug logging to track when/why scissor rects are invalid
   - Never allow a scissor rect error to reach wgpu

2. **Event Loop Architecture**:
   - Ensure winit event loop properly handles multiple windows
   - Each window needs its own event handling context
   - Main window should never block on subwindow events

3. **DPI and Coordinate Systems**:
   - Carefully track logical vs physical coordinates throughout
   - Ensure all viewport sizes match their actual render targets
   - Handle DPI scaling consistently across all windows

4. **Debugging Tools**:
   - Add comprehensive debug logging for viewport operations
   - Create visual debug overlays showing viewport boundaries
   - Log all coordinate transformations and size calculations

### Implementation Strategy:

1. **First Priority - Fix Scissor Rect Errors**:
   ```rust
   // Create a safe wrapper
   pub fn safe_set_scissor_rect(
       pass: &mut RenderPass,
       x: u32, y: u32, w: u32, h: u32,
       render_target_width: u32,
       render_target_height: u32
   ) {
       // Clamp to valid bounds
       let x = x.min(render_target_width);
       let y = y.min(render_target_height);
       let w = w.min(render_target_width.saturating_sub(x));
       let h = h.min(render_target_height.saturating_sub(y));
       
       // Only set if non-zero
       if w > 0 && h > 0 {
           pass.set_scissor_rect(x, y, w, h);
       }
   }
   ```

2. **Second Priority - Fix Window Event Handling**:
   - Review how we're processing events in the main loop
   - Ensure each window gets its own imgui context updates
   - Properly handle focus changes between windows

3. **Third Priority - Fix Rendering Pipeline**:
   - Ensure each viewport has correct size information
   - Validate all coordinate transformations
   - Add defensive checks at every rendering step

### Testing Requirements:
- Test with multiple panels detached
- Test rapid focus switching between windows
- Test resizing all windows simultaneously
- Test on high-DPI displays
- Verify no scissor rect errors in any scenario

### Success Criteria:
- Zero scissor rect validation errors
- All windows remain responsive at all times
- UI in subwindows is fully functional
- Smooth performance with multiple windows
- Clean debug output with no warnings