## FEATURE:

Complete removal of the ImGui multi-window/panel detachment feature (NOT the viewport panel that displays the game), migration back to the official imgui-rs crate, and comprehensive cleanup of the editor codebase. The multi-window detachment feature has proven to be too complex, unstable, and "wonky" - causing more problems than it solves. The editor should be simplified to use only docked panels within a single window.

**IMPORTANT**: The "3D Viewport" panel that displays the rendered game MUST be preserved. We are only removing the ability to detach panels into separate OS windows.

### Scope of Work:

1. **Migrate back to official imgui-rs**:
   - Update Cargo.toml to use official imgui-rs crates instead of the fork
   - Remove dependency on imgui-rs fork from github.com/eliasstepanik
   - Update to latest stable versions of:
     - imgui = "0.12" (or latest)
     - imgui-wgpu = "0.25" (or latest)
     - imgui-winit-support = "0.12" (or latest)
   - Resolve any API differences between fork and official version

2. **Remove all multi-window/detachment-related code**:
   - Remove viewport_backend module (for multi-window support)
   - Remove viewport_renderer, viewport_renderer_backend modules
   - Remove enhanced_viewport_renderer module
   - Remove all multi-window creation and management code
   - Remove viewport_mode_enabled flags and related state
   - Remove all #[cfg(feature = "viewport")] conditional compilation blocks
   - Remove the viewport feature from Cargo.toml
   - KEEP the viewport panel rendering code (render_viewport_panel function)

3. **Simplify editor_state.rs**:
   - Remove all multi-window initialization code
   - Remove check_viewport_toggle, set_viewport_mode methods
   - Remove render_with_viewports method - use only render_single_window
   - Remove multi-window related fields from EditorState struct
   - Simplify render pipeline to single window only
   - KEEP render_viewport() method that renders the game to texture
   - KEEP the viewport render target and texture handling

4. **Clean up panel system**:
   - Remove viewport_mode_enabled parameters from all panel rendering functions
   - Remove detachment/attachment logic from panels
   - Simplify panel state to only support docked mode
   - Ensure all panels (Hierarchy, Inspector, Assets, and 3D Viewport) work within docked interface
   - KEEP the 3D Viewport panel fully functional

5. **Remove unnecessary files**:
   - Delete viewport_backend.rs, viewport_renderer.rs, viewport_renderer_backend.rs
   - Delete enhanced_viewport_renderer.rs
   - Delete detached_window_manager.rs
   - Delete any test files related to multi-window functionality
   - Remove multi-window related documentation files
   - DO NOT delete files related to the 3D viewport panel

6. **Simplify main loop**:
   - Remove process_viewport_requests calls
   - Remove viewport-specific event handling
   - Simplify window management to single main window only

7. **Default window resolution**:
   - Set the main window to fullscreen resolution by default
   - Query the primary monitor's resolution at startup
   - Only override if WINDOW_WIDTH and WINDOW_HEIGHT env variables are set
   - Example: If screen is 1920x1080, start with that size unless env vars specify otherwise

## EXAMPLES:

Before (complex viewport code):
```rust
#[cfg(feature = "viewport")]
{
    if self.viewport_mode_enabled && self.viewport_renderer.is_some() {
        self.render_with_viewports(render_context, encoder, view, window, window_manager);
        return;
    }
}
self.render_single_window(render_context, encoder, view, window);
```

After (simplified):
```rust
self.render(render_context, encoder, view, window);
```

Before (panel with viewport support):
```rust
pub fn render_hierarchy_panel(
    ui: &Ui, 
    shared_state: &EditorSharedState, 
    panel_manager: &mut PanelManager,
    viewport_mode_enabled: bool
) {
    // Complex logic for viewport mode
}
```

After (simplified panel):
```rust
pub fn render_hierarchy_panel(
    ui: &Ui, 
    shared_state: &EditorSharedState, 
    panel_manager: &mut PanelManager
) {
    // Simple docked panel only
}
```

Window resolution example:
```rust
// Before: Fixed size
let window_width = 1600;
let window_height = 900;

// After: Screen resolution by default
let (window_width, window_height) = if let (Ok(width), Ok(height)) = 
    (env::var("WINDOW_WIDTH"), env::var("WINDOW_HEIGHT")) {
    // Use env vars if set
    (width.parse().unwrap_or(1600), height.parse().unwrap_or(900))
} else {
    // Use primary monitor resolution
    event_loop.primary_monitor()
        .map(|monitor| {
            let size = monitor.size();
            (size.width, size.height)
        })
        .unwrap_or((1920, 1080))
};
```

## DOCUMENTATION:

- Official imgui-rs repository: https://github.com/imgui-rs/imgui-rs
- ImGui docking documentation: https://github.com/ocornut/imgui/wiki/Docking
- imgui-rs examples for single window applications
- Migration guide for imgui-rs versions
- Keep the editor focused on being a reliable, single-window docked interface

## OTHER CONSIDERATIONS:

1. **Migration considerations**:
   - The official imgui-rs doesn't support viewport/multi-window features
   - This is actually a benefit - it enforces the single-window design
   - May need to adjust some API calls if they've changed between fork and official
   - Ensure docking still works properly with official version

2. **Preserve working functionality**:
   - Ensure all panels continue to work, especially the 3D Viewport panel
   - The 3D Viewport panel MUST continue to display the rendered game
   - Maintain the docking system - panels should be dockable and rearrangeable
   - Keep the layout save/load functionality
   - Preserve all scene management features
   - Keep the render_viewport() functionality that renders game to texture
   - Window starts at screen resolution for better user experience

3. **Improve code quality**:
   - Remove all dead code related to multi-window support
   - Simplify the rendering pipeline (but keep game-to-texture rendering)
   - Reduce complexity in editor_state.rs (currently over 1700 lines)
   - Make the code more maintainable and easier to understand

4. **Testing after cleanup**:
   - Verify all panels render correctly, especially the 3D Viewport
   - Test docking/undocking panels within the main window
   - Ensure layout persistence works
   - Verify no multi-window menu items remain (remove "Enable Viewport Mode")
   - Confirm the game renders correctly in the viewport panel

5. **Benefits of removal**:
   - Eliminate complex multi-window synchronization issues
   - Remove scissor rect coordinate transformation problems
   - Avoid ImGui assertion errors related to platform windows
   - Reduce maintenance burden
   - Improve editor stability and reliability
   - Simplify to a single-window editor while keeping full functionality

6. **Keep it simple**:
   - The editor should focus on being a solid, single-window tool
   - Docking provides enough flexibility for panel arrangement
   - Multi-window support adds complexity without significant benefit
   - Follow the principle: "Do one thing well"