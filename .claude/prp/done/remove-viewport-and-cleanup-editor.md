name: "Remove Multi-Window Feature and Clean Up Editor"
description: |

## Purpose
Complete removal of the ImGui multi-window/panel detachment feature while preserving the 3D viewport panel, migrating back to official imgui-rs, and comprehensive editor cleanup.

## Core Principles
1. **Preserve Working Features**: The 3D viewport panel that displays the game MUST continue working
2. **Clean Architecture**: Remove all multi-window complexity while maintaining single-window functionality
3. **Official Dependencies**: Migrate from fork to stable official imgui-rs crates
4. **Follow Conventions**: Match existing patterns from CLAUDE.md and codebase
5. **Default to Screen Resolution**: Window should start at monitor resolution unless overridden

---

## Goal
Remove the complex and unstable multi-window detachment feature that allows panels to float as separate OS windows, while keeping all editor panels functional within a single docked interface. The 3D viewport panel that shows the game render MUST be preserved.

## Why
- **Stability**: Multi-window feature causes scissor rect errors, focus issues, and assertion failures
- **Maintainability**: Current implementation is overly complex with many workarounds
- **Dependencies**: Official imgui-rs is more stable than custom forks
- **User Experience**: Single window with docking provides sufficient flexibility

## What
- Remove all code that enables panels to detach into separate OS windows
- Keep all panels (Hierarchy, Inspector, Assets, 3D Viewport) working in docked mode
- Migrate to official imgui-rs crates
- Default window size to primary monitor resolution
- Clean up editor_state.rs (currently 1700+ lines)

### Success Criteria
- [ ] Editor compiles and runs without viewport feature
- [ ] All panels render correctly in single window
- [ ] Docking system works for panel arrangement
- [ ] 3D viewport panel displays game render
- [ ] Window starts at screen resolution by default
- [ ] No multi-window menu items remain
- [ ] Official imgui-rs crates are used

## All Needed Context

### Documentation & References
```yaml
- url: https://github.com/imgui-rs/imgui-rs
  why: Official imgui-rs repository for migration reference
  
- url: https://crates.io/crates/imgui
  why: Latest version is 0.12.0, need to check API compatibility
  
- file: CLAUDE.md
  why: Contains mandatory development guidelines and patterns to follow

- file: editor/src/panels/viewport.rs
  why: This is the 3D viewport panel to PRESERVE (NOT the multi-window feature)
  
- file: game/src/main.rs
  why: Window creation pattern and feature initialization to modify

- file: editor/src/editor_state.rs
  why: Main file to clean up, contains viewport mode logic
```

### Current Codebase Structure (Multi-Window Related)
```bash
editor/src/
├── viewport_backend.rs              # DELETE - Multi-window platform backend
├── viewport_renderer_backend.rs     # DELETE - Multi-window renderer backend
├── enhanced_viewport_renderer.rs    # DELETE - Enhanced multi-window renderer
├── viewport_sys_integration.rs      # DELETE - Direct imgui-sys integration
├── viewport_render_fix.rs          # DELETE - Viewport rendering fixes
├── viewport_scissor_fix.rs         # DELETE - Scissor rect fixes
├── viewport_surface_fix.rs         # DELETE - Surface management fixes
├── viewport_workarounds.rs         # DELETE - Debug utilities
├── detached_window.rs              # DELETE - Individual detached windows
├── detached_window_manager.rs      # DELETE - Manages all detached windows
├── panels/
│   ├── detachable.rs              # MODIFY - Remove detach/attach UI
│   └── viewport.rs                # KEEP - 3D game viewport panel
├── panel_state.rs                 # MODIFY - Remove detachment logic
├── editor_state.rs                # MODIFY - Major cleanup needed
└── safe_imgui_renderer.rs         # KEEP - Scissor rect safety wrapper
```

### Desired Codebase Structure
```bash
editor/src/
├── panels/
│   ├── mod.rs                     # Simple panel exports
│   ├── hierarchy.rs               # Scene hierarchy panel
│   ├── inspector.rs               # Entity inspector panel
│   ├── assets.rs                  # Asset browser panel
│   └── viewport.rs                # 3D game viewport panel (PRESERVED)
├── panel_state.rs                 # Simplified panel state (docked only)
├── editor_state.rs                # Cleaned up, single-window focused
├── safe_imgui_renderer.rs         # Scissor rect safety wrapper
└── shared_state.rs                # Shared editor state
```

### Known Gotchas & Patterns
```rust
// CRITICAL: Official imgui-rs 0.12.0 does NOT support viewports/multi-window
// This is actually beneficial - enforces single-window design

// PATTERN: Feature removal uses conditional compilation
#[cfg(feature = "viewport")]  // Remove these blocks
#[cfg(not(feature = "viewport"))]  // Keep/promote these blocks

// PATTERN: Window creation (game/src/main.rs:64)
let window_attributes = WindowAttributes::default()
    .with_title("WebGPU Game Engine Demo")
    .with_inner_size(winit::dpi::PhysicalSize::new(width, height));

// GOTCHA: The 3D viewport panel is NOT related to multi-window
// panels/viewport.rs renders the game - MUST BE PRESERVED

// GOTCHA: Panel rendering functions have viewport_mode_enabled parameter
// This parameter needs to be removed from all signatures
```

## Implementation Blueprint

### Task List

```yaml
Task 1 - Update Dependencies:
MODIFY editor/Cargo.toml:
  - REPLACE imgui fork with official: imgui = "0.12"
  - ADD: imgui-wgpu = "0.25"
  - ADD: imgui-winit-support = "0.12"
  - REMOVE: viewport feature definition
  - REMOVE: imgui-sys dependency

MODIFY game/Cargo.toml:
  - REMOVE: viewport from default features
  - REMOVE: viewport feature definition

Task 2 - Clean Up Module Exports:
MODIFY editor/src/lib.rs:
  - REMOVE: All #[cfg(feature = "viewport")] module declarations
  - REMOVE: viewport_backend, viewport_renderer_backend, etc.
  - KEEP: panels, panel_state, safe_imgui_renderer

Task 3 - Simplify Panel Rendering:
MODIFY editor/src/panels/mod.rs:
  - REMOVE: viewport_mode_enabled parameter from all functions
  - UPDATE: All panel rendering function signatures

MODIFY editor/src/panels/detachable.rs:
  - REMOVE: This file entirely OR
  - SIMPLIFY: Remove detach/attach button logic

Task 4 - Clean Up EditorState:
MODIFY editor/src/editor_state.rs:
  - REMOVE: viewport_mode_enabled field
  - REMOVE: pending_viewport_toggle field
  - REMOVE: viewport_backend, viewport_renderer fields
  - REMOVE: check_viewport_toggle method
  - REMOVE: set_viewport_mode method
  - REMOVE: render_with_viewports method
  - REMOVE: All #[cfg(feature = "viewport")] blocks
  - SIMPLIFY: render_ui_and_draw to always use single window
  - KEEP: render_viewport method (renders game to texture)
  - KEEP: viewport render target and texture handling

Task 5 - Update Panel State:
MODIFY editor/src/panel_state.rs:
  - REMOVE: is_detached field from PanelState
  - REMOVE: detach/attach methods
  - REMOVE: pending detach/attach request handling
  - SIMPLIFY: Panel state to only track visibility and docked position

Task 6 - Update Main Loop:
MODIFY game/src/main.rs:
  - REMOVE: process_viewport_requests calls
  - REMOVE: viewport backend initialization
  - REMOVE: #[cfg(feature = "viewport")] blocks
  - ADD: Screen resolution detection for window size

Task 7 - Delete Viewport Files:
DELETE files:
  - editor/src/viewport_backend.rs
  - editor/src/viewport_renderer_backend.rs
  - editor/src/enhanced_viewport_renderer.rs
  - editor/src/viewport_sys_integration.rs
  - editor/src/viewport_render_fix.rs
  - editor/src/viewport_scissor_fix.rs
  - editor/src/viewport_surface_fix.rs
  - editor/src/viewport_workarounds.rs
  - editor/src/detached_window.rs
  - editor/src/detached_window_manager.rs

Task 8 - Update Window Creation:
MODIFY game/src/main.rs window creation:
  - ADD: Monitor resolution detection
  - ADD: Environment variable override support
  - PATTERN: See pseudocode below
```

### Window Resolution Pseudocode
```rust
// Task 8 - In game/src/main.rs around line 64
use std::env;

// Get window dimensions
let (window_width, window_height) = if let (Ok(width), Ok(height)) = 
    (env::var("WINDOW_WIDTH"), env::var("WINDOW_HEIGHT")) {
    // Use environment variables if set
    (
        width.parse().unwrap_or(1280),
        height.parse().unwrap_or(720)
    )
} else {
    // Use primary monitor resolution
    event_loop.primary_monitor()
        .map(|monitor| {
            let size = monitor.size();
            (size.width, size.height)
        })
        .unwrap_or((1920, 1080)) // Fallback to 1080p
};

let window_attributes = WindowAttributes::default()
    .with_title("WebGPU Game Engine Demo")
    .with_inner_size(winit::dpi::PhysicalSize::new(window_width, window_height));
```

### Integration Points
```yaml
BUILD:
  - just commands should work without viewport feature
  - Remove viewport from default features in justfile if present
  
IMPORTS:
  - Update all panel render calls to remove viewport_mode_enabled
  - Fix any compilation errors from removed types
  
UI:
  - Remove "Enable Viewport Mode" menu item from View menu
  - Ensure all panels have docking enabled by default
```

## Validation Loop

### Level 1: Clean Removal
```bash
# Remove all viewport files first
rm editor/src/viewport_*.rs
rm editor/src/detached_window*.rs

# Check that no viewport references remain
rg "viewport_mode_enabled|viewport_backend|viewport_renderer" editor/src/

# Expected: No matches except in panels/viewport.rs (the 3D panel)
```

### Level 2: Compilation
```bash
# Build without viewport feature
cargo build --no-default-features --features editor

# Fix any compilation errors by:
# 1. Removing viewport-related code
# 2. Updating function signatures
# 3. Removing unused imports
```

### Level 3: Runtime Testing
```bash
# Run the editor
cargo run --release

# Test checklist:
# - [ ] Window opens at screen resolution
# - [ ] All panels visible (Hierarchy, Inspector, Assets, 3D Viewport)
# - [ ] 3D viewport shows rendered game
# - [ ] Panels can be docked/rearranged
# - [ ] No "Enable Viewport Mode" in View menu
# - [ ] Layout save/load works
```

### Level 4: Validation Commands
```bash
# Ensure code quality (from CLAUDE.md)
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
cargo test --workspace

# Build documentation
cargo doc --workspace --no-deps
```

## Final Validation Checklist
- [ ] No viewport/multi-window code remains
- [ ] Official imgui-rs 0.12.0 is used
- [ ] All 4 panels render correctly
- [ ] 3D viewport panel shows game render
- [ ] Docking system works properly
- [ ] Window starts at screen resolution
- [ ] Environment variables override window size
- [ ] No compilation warnings
- [ ] All tests pass
- [ ] Code follows CLAUDE.md guidelines

---

## Anti-Patterns to Avoid
- ❌ Don't delete panels/viewport.rs (it's the game view, not multi-window)
- ❌ Don't break existing panel functionality
- ❌ Don't leave #[cfg(feature = "viewport")] blocks
- ❌ Don't forget to test docking after removal
- ❌ Don't hardcode window size without env var override
- ❌ Don't skip running `just preflight` validation

## Confidence Score: 9/10

The removal is straightforward due to:
- Clear separation between multi-window code and core functionality
- Established patterns for feature removal via conditional compilation
- Official imgui-rs doesn't support viewports, enforcing the design
- Comprehensive file list and validation steps provided

The only minor uncertainty is potential API differences between the fork and official imgui-rs 0.12.0, but these should be minor and easily resolved during compilation.