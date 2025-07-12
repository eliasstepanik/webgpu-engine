## FEATURE:

Fix the editor interaction system and viewport rendering issues:

1. **Editor Input Handling**: The editor mode (toggled with Tab) only works for a very brief moment after switching to it, then stops responding to mouse/keyboard input. The ImGui UI should remain interactive when in editor mode.

2. **Viewport Window Missing**: The viewport window that should display the game render is not visible at all. It might be rendering behind other elements or not rendering at all. The viewport should be a visible ImGui window showing the game scene.

3. **Window Resizing Issue**: Window resizing only works correctly after switching modes (Tab key) at least once after application start. Before the first mode switch, resizing shows black areas in the newly exposed regions. **Critical**: If you resize before switching modes and then try to switch modes, the application logs "Surface outdated, reconfiguring" and then crashes.

## EXAMPLES:

The editor should work similar to game engines like Unity or Godot where:
- The viewport window shows the game scene
- UI panels (Hierarchy, Inspector, Assets) are dockable windows
- Mouse/keyboard input is properly captured by ImGui when in editor mode
- Tab key toggles between editor UI mode and game input mode

## DOCUMENTATION:

Key files to reference:
- `/editor/src/editor_state.rs` - Main editor state and input handling
- `/editor/src/panels/viewport.rs` - Viewport panel implementation
- `/game/src/main.rs` - Main game loop with editor integration (lines 63-74 for event handling, 134-217 for rendering)
- `imgui-rs` documentation for proper event handling
- `imgui-winit-support` documentation for platform integration

## OTHER CONSIDERATIONS:

1. **Event Handling Order**: The current implementation has `editor_state.handle_event()` which should consume events when in UI mode, preventing them from reaching the game.

2. **Render Order**: Currently rendering happens in this order:
   - Render game to viewport texture
   - Begin ImGui frame
   - Render ImGui UI
   - Present to surface

3. **Known Issues**:
   - Window resize causes "Outdated" surface errors (partially fixed)
   - Black areas appear when resizing until mode is switched once
   - **CRASH**: Resizing before first mode switch + then switching modes = crash after "Surface outdated, reconfiguring"
   - ImGui display size might not be updating correctly on initial setup
   - Initial state configuration might be incomplete
   - Editor state resize might be called with inconsistent state

4. **Debug Info**: When fixing, add debug logging to track:
   - When input mode switches
   - Which events ImGui is capturing
   - Viewport texture creation and rendering
   - Window/surface size mismatches

5. **Initialization Sequence**: The issue with resizing only working after a mode switch suggests that:
   - The initial editor state setup might be missing some configuration
   - Event handling or render state might not be fully initialized until the first mode switch
   - The `handle_event` method might be doing initialization work that should happen in `new()`

6. **Testing**: After fixes, verify:
   - Tab key reliably toggles between modes
   - In editor mode: mouse clicks on UI elements work, keyboard input goes to focused UI elements
   - In game mode: mouse/keyboard input goes to the game
   - Viewport window is visible and shows the game scene
   - Resizing the window works correctly from application start (without needing to switch modes first)
   - No black areas appear when resizing at any point