name: "Fix Editor on Windows - Cross-Platform Compatibility PRP"
description: |

## Purpose
Fix the editor functionality on Windows systems by addressing path handling issues, completing viewport integration, and ensuring cross-platform compatibility. This PRP provides comprehensive context for AI agents to implement Windows-specific fixes while maintaining functionality on other platforms.

## Core Principles
1. **Cross-Platform First**: All fixes must work on Windows, Linux, and macOS
2. **Minimal Platform-Specific Code**: Use platform-agnostic solutions where possible
3. **Test on Windows**: Use mcp__windows-ssh-mcp__exec for validation
4. **Follow CLAUDE.md**: Adhere to all project conventions and guidelines
5. **Preserve Existing Functionality**: Don't break what already works

---

## Goal
Fix the editor to work seamlessly on Windows, addressing:
- Path resolution issues preventing builds
- Incomplete viewport rendering
- Windows-specific window management quirks
- File watching and hot reload on Windows NTFS

## Why
- **Development Parity**: Windows developers need the same editor experience
- **Path Issues Block Development**: Current path mismatches prevent even building on Windows
- **Incomplete Features**: Viewport rendering TODO prevents full editor usage
- **Professional Tool**: A production-ready editor must work on all major platforms

## What
Fix critical path issues, complete viewport integration, and ensure Windows compatibility for:
- Building and running the project with editor feature
- Proper viewport rendering of game content
- Stable window management and DPI handling
- Functional hot reload and file watching
- Cross-platform path handling

### Success Criteria
- [ ] `just preflight` passes on Windows via SSH MCP
- [ ] Editor launches and displays properly on Windows
- [ ] Viewport shows rendered game content (not placeholder)
- [ ] File paths work correctly with Windows backslashes
- [ ] Hot reload detects changes on Windows NTFS
- [ ] High DPI displays work without scaling issues
- [ ] All existing functionality remains intact on Linux/macOS

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/std/latest/std/path/
  why: Rust path handling, canonicalize(), platform differences
  
- url: https://docs.rs/winit/latest/winit/dpi/index.html
  why: Windows DPI handling, scale factor events, physical vs logical sizes
  
- url: https://github.com/gfx-rs/wgpu/wiki/Windowing
  why: wgpu Windows-specific surface creation and compatibility
  
- url: https://docs.rs/imgui-wgpu/latest/imgui_wgpu/
  why: Texture registration for viewport, render() with custom textures
  
- url: https://docs.rs/notify/latest/notify/
  why: Windows file watching behavior, debouncing, NTFS considerations

- file: engine/src/graphics/render_target.rs
  why: Already implemented render target, needs integration with ImGui
  
- file: editor/src/editor_state.rs
  why: Contains TODOs for viewport, window sizing workarounds
  
- file: engine/Cargo.toml
  why: Incorrect example path causing Windows build failure
  
- file: .claude/prp/editor_system.md
  why: Original editor implementation plan, shows intended architecture
```

### Current Issues Found

1. **Critical Path Issue** (Blocks Everything):
   - `engine/Cargo.toml` references `../examples/scene_demo.rs`
   - Actual file location: `.claude/examples/scene_demo.rs`
   - Causes: `Error: file C:\...\examples\scene_demo.rs does not exist`

2. **Incomplete Viewport Rendering**:
   - Lines 289-292 in `editor_state.rs`: TODO comment
   - Render target texture not registered with ImGui
   - Viewport panel shows placeholder instead of game

3. **Window Initialization Issues**:
   - Frame skipping workaround (lines 187-194)
   - Display size validation checks (lines 207-232)
   - Suggests Windows timing/DPI issues

4. **No Platform-Specific Path Handling**:
   - No path normalization for Windows
   - Mixed forward/backslash usage
   - No canonicalization for cross-platform paths

### Known Gotchas
```rust
// CRITICAL: Windows paths use backslashes, must normalize
// Example: C:\Users\... vs /mnt/c/Users/...
// Example: Cargo expects forward slashes even on Windows

// CRITICAL: ImGui texture registration requires specific format
// Texture must have TEXTURE_BINDING usage flag (already set)
// Must register with imgui_renderer.textures.insert()

// CRITICAL: Windows DPI scaling affects window sizes
// Physical size != Logical size on high DPI displays
// winit provides both, must use correct one

// CRITICAL: notify crate on Windows
// Rapid file changes need debouncing (already implemented)
// Some editors create temp files that trigger false events
```

## Implementation Blueprint

### Fix Structure
```
engine/
  Cargo.toml (FIX PATH)
  src/
    graphics/
      render_target.rs (ALREADY GOOD)
    utils/
      paths.rs (CREATE - platform path utils)

editor/
  src/
    editor_state.rs (COMPLETE VIEWPORT)
    
.claude/
  examples/
    scene_demo.rs (KEEP HERE - this is correct location)
```

### List of Tasks

```yaml
Task 1 - Fix Critical Path Issue:
MODIFY engine/Cargo.toml:
  - FIND: path = "../examples/scene_demo.rs"
  - REPLACE: path = "../.claude/examples/scene_demo.rs"
  - This fixes Windows build error immediately

Task 2 - Add Path Utilities:
CREATE engine/src/utils/paths.rs:
  - Platform-agnostic path normalization
  - Convert backslashes to forward for consistency
  - Handle Windows drive letters and UNC paths
  - Export from engine/src/utils/mod.rs

Task 3 - Complete Viewport Rendering:
MODIFY editor/src/editor_state.rs:
  - Register render target texture with ImGui
  - Update render_viewport() to pass texture to viewport panel
  - Remove TODO comment once working
  - Handle texture recreation on resize

Task 4 - Fix Window Initialization:
MODIFY editor/src/editor_state.rs:
  - Improve frame skipping logic for Windows
  - Add explicit DPI awareness setup
  - Better size validation with logging

Task 5 - Update Viewport Panel:
MODIFY editor/src/panels/viewport.rs:
  - Accept texture ID parameter
  - Display game render target instead of placeholder
  - Handle aspect ratio correctly

Task 6 - Test Via Windows SSH:
USE mcp__windows-ssh-mcp__exec:
  - Run just preflight
  - Test editor launch
  - Verify viewport rendering
  - Check hot reload
```

### Task 1: Fix Path Issue
```toml
# engine/Cargo.toml
[[example]]
name = "scene_demo"
path = "../.claude/examples/scene_demo.rs"  # Fixed path
```

### Task 2: Path Utilities
```rust
// engine/src/utils/paths.rs
use std::path::{Path, PathBuf};

/// Normalize path for cross-platform compatibility
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    
    // Convert to string and replace backslashes
    let path_str = path.to_string_lossy().replace('\\', "/");
    
    // Handle Windows absolute paths (C:/ etc)
    if cfg!(windows) && path_str.len() > 2 {
        if path_str.chars().nth(1) == Some(':') {
            // Already absolute Windows path
            return PathBuf::from(path_str);
        }
    }
    
    PathBuf::from(path_str)
}

/// Get canonical path with fallback
pub fn canonical_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref()
        .canonicalize()
        .unwrap_or_else(|_| normalize_path(path))
}
```

### Task 3: Complete Viewport
```rust
// editor/src/editor_state.rs - in new() method
impl EditorState {
    pub fn new(render_context: &Arc<RenderContext>, window: &Window) -> Self {
        // ... existing code ...
        
        // Register render target texture with ImGui
        let texture_id = imgui_renderer.textures.insert(
            (render_target.texture.as_ref(), &render_target.view)
        );
        
        // Store texture_id in EditorState
        // ... rest of initialization ...
    }
    
    pub fn render_viewport(&mut self, renderer: &mut Renderer, world: &World) {
        // Render game to our render target
        if let Err(e) = renderer.render_to_target(world, &self.render_target) {
            error!("Failed to render to viewport: {:?}", e);
        }
    }
}

// In render_ui method
pub fn render_ui(&mut self, world: &mut World, window: &Window) {
    // ... existing panels ...
    
    // Pass texture ID to viewport panel
    viewport::render_viewport_panel(&ui, self.texture_id, &mut self.render_target);
}
```

### Task 4: Window Init Improvements
```rust
// editor/src/editor_state.rs
pub fn handle_event(&mut self, window: &Window, event: &Event<()>) -> bool {
    // Add explicit DPI handling
    let scale_factor = window.scale_factor();
    
    // ... existing platform update ...
    
    // Better size validation
    let window_size = window.inner_size();
    let physical_size = (window_size.width, window_size.height);
    let logical_size = (
        (window_size.width as f64 / scale_factor) as f32,
        (window_size.height as f64 / scale_factor) as f32,
    );
    
    debug!(
        physical_size = ?physical_size,
        logical_size = ?logical_size,
        scale_factor = scale_factor,
        "Window metrics"
    );
}
```

### Task 5: Viewport Panel Update
```rust
// editor/src/panels/viewport.rs
pub fn render_viewport_panel(
    ui: &imgui::Ui,
    texture_id: imgui::TextureId,
    render_target: &mut RenderTarget,
) {
    ui.window("Viewport")
        .resizable(true)
        .build(|| {
            let available_size = ui.content_region_avail();
            
            // Resize render target if needed
            let new_size = (available_size[0] as u32, available_size[1] as u32);
            if new_size != render_target.size && new_size.0 > 0 && new_size.1 > 0 {
                debug!("Viewport resize needed: {:?} -> {:?}", render_target.size, new_size);
                // Trigger render target recreation in editor_state
            }
            
            // Display the game render target
            ui.image(texture_id, available_size);
        });
}
```

## Validation Loop

### Level 1: Build & Format
```bash
# On Windows via SSH MCP
mcp__windows-ssh-mcp__exec "cd C:\Users\elias\RustroverProjects\webgpu-template && cargo fmt --all"
mcp__windows-ssh-mcp__exec "cd C:\Users\elias\RustroverProjects\webgpu-template && cargo clippy --workspace --all-features -- -D warnings"
```

### Level 2: Preflight Check
```bash
# This must pass - it's currently failing
mcp__windows-ssh-mcp__exec "cd C:\Users\elias\RustroverProjects\webgpu-template && just preflight"
```

### Level 3: Editor Launch Test
```bash
# Test editor feature
mcp__windows-ssh-mcp__exec "cd C:\Users\elias\RustroverProjects\webgpu-template && cargo build --features editor"
mcp__windows-ssh-mcp__exec "cd C:\Users\elias\RustroverProjects\webgpu-template && cargo run --features editor"
```

### Level 4: Feature Tests
```powershell
# Test viewport rendering
# 1. Launch editor
# 2. Check viewport panel shows game content (not gray)
# 3. Resize viewport - should update correctly
# 4. Move viewport panel - rendering should continue

# Test hot reload
# 1. Launch editor  
# 2. Modify a scene file
# 3. Verify reload triggers within 1-2 seconds

# Test path handling
# 1. Save a scene via editor
# 2. Load scene via editor
# 3. Verify paths work with Windows format
```

## Final Validation Checklist
- [ ] Build passes on Windows: `just preflight` via SSH MCP
- [ ] Editor launches without errors on Windows
- [ ] Viewport displays game content (not placeholder)
- [ ] Window resizing works without glitches
- [ ] High DPI displays scale correctly
- [ ] File paths handle Windows format (C:\...)
- [ ] Hot reload detects file changes on NTFS
- [ ] All tests pass on Linux/macOS (no regression)
- [ ] No new `println!` - only `tracing` macros used
- [ ] Documentation updated for Windows users

---

## Anti-Patterns to Avoid
- ❌ Don't use `#[cfg(windows)]` unless absolutely necessary
- ❌ Don't hardcode path separators - use `std::path`
- ❌ Don't skip Windows testing "because it should work"
- ❌ Don't ignore DPI/scaling - test on high DPI display
- ❌ Don't use blocking file operations in render loop
- ❌ Don't panic on Windows-specific errors - handle gracefully
- ❌ Don't mix path styles - normalize everything

## Critical Implementation Notes

### Path Handling
1. **Always Normalize**: Convert paths at entry points
2. **Forward Slashes**: Cargo.toml needs forward slashes even on Windows
3. **Canonical Paths**: Use with fallback for non-existent files
4. **Display Paths**: Show native format to users, use normalized internally

### Viewport Integration  
1. **Texture Format**: Must match surface format (Rgba8UnormSrgb)
2. **Usage Flags**: Already set correctly in RenderTarget
3. **Resize Handling**: Recreate texture and re-register with ImGui
4. **Render Order**: Game renders first, then ImGui overlay

### Windows-Specific
1. **DPI Awareness**: Use winit's scale_factor for all calculations
2. **Event Timing**: Windows may send events in different order
3. **File Watching**: Some events may be duplicated on NTFS
4. **Path Length**: Windows has 260 char limit unless long paths enabled

### Testing Strategy
1. **Use SSH MCP**: Test directly on Windows environment
2. **Both Debug/Release**: Editor only in debug, but test both
3. **Different DPI**: Test at 100%, 150%, 200% scaling
4. **Path Edge Cases**: Spaces, special chars, long paths

## Success Metrics
- Zero build errors on Windows
- Viewport rendering functional within 2 seconds of launch  
- No visual glitches at any DPI setting
- File operations work with native Windows paths
- Performance parity with Linux/macOS
- Clean `just preflight` output

## Confidence Score: 8/10
High confidence due to:
- Clear identification of root cause (path issue)
- Existing render target implementation
- Well-structured editor codebase
- Comprehensive testing via SSH MCP

Minor uncertainty in:
- Exact ImGui texture registration API
- Windows-specific timing quirks
- High DPI edge cases

The implementation path is clear with immediate wins (path fix) and systematic completion of remaining TODOs.