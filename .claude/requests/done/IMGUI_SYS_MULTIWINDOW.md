# Request: Multi-Window Editor Support using imgui-sys

## FEATURE:

Implement true multi-window/multi-viewport support for the editor by migrating from imgui-rs to imgui-sys with manual Dear ImGui integration. This will overcome the current imgui-rs 0.12 single-context limitation that prevents panel detachment to separate OS windows.

### Core Requirements:

1. **Multiple ImGui Contexts**: Create separate Dear ImGui contexts for each detached window using imgui-sys raw bindings
2. **Cross-Context State Sharing**: Implement safe data sharing between contexts using Arc<Mutex<>> for editor state
3. **Independent Window Rendering**: Each detached window renders its ImGui UI independently with proper event handling
4. **Seamless Panel Migration**: Allow panels to be detached/reattached between main window and separate OS windows
5. **Layout Persistence**: Maintain current JSON layout persistence with full detached window support
6. **Performance Optimization**: Ensure minimal overhead for multi-context management

### Technical Approach:

- **Replace imgui-rs dependency** with imgui-sys for direct Dear ImGui C API access
- **Custom Rust wrapper layer** for type-safe ImGui operations while maintaining flexibility
- **Per-window context management** with proper initialization/cleanup
- **Shared font atlas** across contexts to reduce memory usage
- **Event routing system** to handle input events for appropriate windows
- **WGPU integration** maintaining current render context architecture

## EXAMPLES:

Reference the existing multi-window architecture in these files:

### Current Implementation (To Be Enhanced):
- `editor/src/detached_window.rs` - Window wrapper (currently simplified due to limitations)
- `editor/src/detached_window_manager.rs` - Multi-window coordinator (currently disabled)
- `editor/src/panel_state.rs` - Panel state management with persistence
- `examples/dual_monitor_layout.json` - Example layout with detached panels
- `examples/detached_workflow.json` - Multi-window workflow configuration

### Architecture Pattern:
```rust
// Target architecture using imgui-sys
struct ImGuiContext {
    raw_context: *mut imgui_sys::ImGuiContext,
    io: ImGuiIO,
    style: ImGuiStyle,
}

struct DetachedWindow {
    window: Arc<Window>,
    imgui_context: ImGuiContext,
    renderer: ImGuiWgpuRenderer,
    platform: WinitPlatform,
}
```

### Integration Points:
- `editor/src/editor_state.rs` - Main editor context (lines 95-100 for ImGui setup)
- `game/src/main.rs` - Window event handling (lines 421-437 for editor events)
- `editor/src/panels/` - All panel rendering code (to be made context-agnostic)

## DOCUMENTATION:

### Primary References:

1. **Dear ImGui Multi-Viewport Documentation**:
   - https://github.com/ocornut/imgui/blob/master/docs/CHANGELOG.txt (Viewport API)
   - https://github.com/ocornut/imgui/wiki/Multi-Viewports (Official multi-viewport guide)
   - https://github.com/ocornut/imgui/blob/master/imgui.h (Context management APIs)

2. **imgui-sys Rust Bindings**:
   - https://docs.rs/imgui-sys/ (Raw bindings documentation)
   - https://github.com/imgui-rs/imgui-rs/tree/main/imgui-sys (Source and examples)
   - https://docs.rs/imgui-sys/latest/imgui_sys/ (API reference)

3. **Multi-Context Pattern References**:
   - Dear ImGui's `imgui_impl_glfw.cpp` - Multi-context window management
   - Dear ImGui's `imgui_impl_opengl3.cpp` - Context switching patterns
   - https://github.com/ocornut/imgui/issues/1542 (Multi-context discussion)

4. **WGPU Integration Patterns**:
   - Current `imgui-wgpu` source for render integration patterns
   - https://github.com/gfx-rs/wgpu/tree/master/examples (WGPU multi-window examples)
   - `engine/src/graphics/context.rs` - Current render context architecture

5. **Rust FFI Safety Patterns**:
   - https://doc.rust-lang.org/nomicon/ffi.html (Rust FFI guidelines)
   - https://github.com/rust-lang/rfcs/blob/master/text/2945-c-unwind.md (C ABI safety)

## OTHER CONSIDERATIONS:

### Critical Implementation Details:

1. **Memory Safety with Raw Pointers**:
   - Dear ImGui contexts are C pointers requiring careful Rust lifetime management
   - Must ensure contexts are properly destroyed in correct order
   - Font atlas sharing requires special attention to prevent use-after-free

2. **Event Handling Complexity**:
   - Current imgui-winit-support won't work with multiple contexts
   - Need custom event routing to appropriate window contexts
   - Mouse capture and keyboard focus management across windows

3. **Font Atlas Management**:
   - Share font atlas between contexts to reduce memory usage
   - Ensure atlas is built once and shared safely
   - Handle font atlas rebuilding when adding/removing contexts

4. **Context Switching Overhead**:
   - Dear ImGui has current context concept - need to switch contexts per window
   - Minimize context switches by batching operations
   - Consider performance impact on rendering loop

5. **State Synchronization Challenges**:
   - Editor state (selected entities, etc.) must be shared between contexts
   - Panel data needs to be accessible from any context
   - Avoid race conditions with concurrent access from multiple windows

### Common Gotchas for AI Assistants:

1. **Don't assume imgui-rs patterns work with imgui-sys** - The APIs are fundamentally different
2. **Context lifetime management is critical** - Dear ImGui contexts must be destroyed in reverse creation order
3. **Font atlas is shared resource** - Cannot be safely accessed from multiple threads simultaneously
4. **Window event routing is complex** - Events must go to correct context based on window focus
5. **WGPU integration requires custom renderer** - Cannot reuse existing imgui-wgpu with multiple contexts
6. **Performance testing essential** - Multiple contexts have overhead that must be measured
7. **Fallback strategy needed** - Must gracefully handle context creation failures

### Integration with Existing Codebase:

1. **Preserve Current Architecture**:
   - Keep existing `EditorState` and `PanelManager` structure
   - Maintain `SharedState` pattern for cross-context data sharing
   - Preserve all current panel rendering logic

2. **Backward Compatibility**:
   - Single-window mode should work exactly as before
   - Layout persistence must continue working seamlessly
   - All existing keyboard shortcuts and workflows preserved

3. **Cargo Feature Flags**:
   - Add `multi-window` feature flag to enable imgui-sys path
   - Default to current imgui-rs implementation for stability
   - Allow users to opt into multi-window functionality

4. **Error Handling Strategy**:
   - Graceful fallback to single-window mode on context creation failure
   - Clear error messages for common setup issues (GPU memory, driver limits)
   - Automatic recovery from window management errors

### Performance and Resource Considerations:

1. **GPU Memory Usage**:
   - Each context may require separate GPU resources
   - Monitor texture atlas memory usage
   - Implement context pooling for frequently created/destroyed windows

2. **CPU Overhead**:
   - Multiple render loops impact CPU usage
   - Consider frame rate limiting for detached windows
   - Profile context switching overhead

3. **Platform-Specific Limitations**:
   - Windows: DPI awareness across multiple monitors
   - macOS: Retina display handling and window server limits
   - Linux: X11 vs Wayland differences in multi-window support

### Success Criteria:

1. **Functional Requirements**:
   - ✅ Panels can be detached to separate OS windows
   - ✅ Multiple windows render independently at full frame rate
   - ✅ Window close/minimize/maximize work correctly
   - ✅ Cross-window drag and drop for panel management
   - ✅ Layout persistence includes detached window positions

2. **Performance Requirements**:
   - ✅ <10% CPU overhead for dual window setup vs single window
   - ✅ <50MB additional GPU memory per detached window
   - ✅ 60fps maintained on mid-range hardware with 2-3 detached windows

3. **Stability Requirements**:
   - ✅ No crashes when rapidly creating/destroying windows
   - ✅ Graceful handling of GPU context loss
   - ✅ Proper cleanup on application exit
   - ✅ Memory leak testing passes with Valgrind/AddressSanitizer

This request represents a significant architectural enhancement that will restore and improve upon the original multi-window vision while working within the constraints of available Rust ImGui bindings.