name: "Fix Editor Scissor Rect Validation Error - Complete Solution"
description: |

## Purpose
Fix the scissor rect validation error in the editor that prevents it from running on Windows. This PRP provides comprehensive context and implementation steps to resolve the issue where ImGui attempts to use a scissor rect of 1920x1080 while the render target is only 1280x720.

## Core Principles
1. **Surface Size Consistency**: Always use wgpu surface configuration size, not window inner size
2. **Defensive Programming**: Multiple validation layers to catch size mismatches
3. **Display Scaling Awareness**: Properly handle high-DPI displays and fractional scaling
4. **Graceful Degradation**: Skip frames rather than crash when sizes don't match
5. **Follow CLAUDE.md**: Adhere to all project conventions and guidelines

---

## Goal
Ensure the editor runs without scissor rect validation errors on all platforms and display configurations by:
- Fixing the size mismatch between ImGui display size and render target
- Properly handling display scaling factors
- Implementing robust initialization sequencing
- Adding defensive validation to prevent future issues

## Why
- **User Impact**: Editor crashes immediately on launch, blocking all development
- **Platform Parity**: Windows users cannot use the editor at all
- **Professional Quality**: A production editor must handle all display configurations
- **Developer Experience**: The error is cryptic and hard to debug without proper handling

## What
Fix the validation error: "Scissor Rect { x: 0, y: 0, w: 1920, h: 1080 } is not contained in the render target (1280, 720, 1)"

### Success Criteria
- [ ] Editor launches without crashes on Windows
- [ ] No scissor rect validation errors in any display configuration
- [ ] Proper viewport rendering with correct sizing
- [ ] Works with display scaling at 100%, 125%, 150%, 200%
- [ ] Handles window resizing without errors
- [ ] Clear error messages if issues occur

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://github.com/ocornut/imgui/issues/8628
  why: Display scaling issue with GLFW/wgpu - exact same 1920x1080 vs actual size issue
  critical: Shows the issue is related to display scaling percentages
  
- url: https://github.com/Yatekii/imgui-wgpu-rs/issues/77
  why: Crashing with Invalid ScissorRect Parameters - common imgui-wgpu issue
  critical: Explains that scissor rect must be contained within render target bounds
  
- url: https://github.com/emilk/egui/issues/2038
  why: wgpu crash if scissor rectangle is outside of the window
  critical: Shows solution of clamping scissor rects before setting them
  
- url: https://docs.rs/imgui-wgpu/0.25.0/imgui_wgpu/
  why: Current version API documentation
  
- url: https://docs.rs/winit/0.30/winit/dpi/index.html
  why: Understanding physical vs logical sizes and scale factors

- file: editor/src/editor_state.rs
  why: Current implementation with multiple workarounds already attempted
  
- file: engine/src/graphics/render_target.rs
  why: RenderTarget implementation that needs to match ImGui display size
  
- file: game/src/main.rs
  why: Window creation with 1280x720 size vs ImGui's 1920x1080 default
```

### Current Codebase Structure
```
editor/
  src/
    editor_state.rs    # Main editor state with ImGui integration
    panels/
      viewport.rs      # Viewport panel that displays render target
engine/
  src/
    graphics/
      context.rs       # RenderContext with surface configuration
      render_target.rs # RenderTarget for off-screen rendering
game/
  src/
    main.rs           # Window creation and event loop
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: ImGui defaults to 1920x1080 if not explicitly set before first frame
// This happens in imgui-winit-support when attaching window

// CRITICAL: Display scaling causes scissor rect to be proportionally larger
// Example: 120% scaling = scissor rect 20% larger than render target

// CRITICAL: Window inner_size() may not match surface configuration size
// Always use surface_config.width/height for render calculations

// CRITICAL: Some window managers (i3) cause 1-pixel discrepancies
// Scissor rect might be exactly 1 pixel wider than render target

// CRITICAL: Scale factor events can arrive after window creation
// Must handle ScaleFactorChanged events properly

// CRITICAL: Frame timing - first few frames may have incorrect sizes
// Current code skips first 5 frames but this may not be enough
```

## Implementation Blueprint

### Core Issue
ImGui is initialized with a default size (1920x1080) that doesn't match the actual window/surface size (1280x720). When ImGui tries to render, it sets scissor rects based on its display size, causing validation errors.

### Solution Architecture
1. Always use surface configuration size for all calculations
2. Force ImGui display size updates at multiple points
3. Clamp scissor rects to render target bounds
4. Add comprehensive validation before rendering

### List of Tasks

```yaml
Task 1 - Refactor Size Handling:
MODIFY editor/src/editor_state.rs:
  - FIND all instances of window.inner_size()
  - REPLACE with surface configuration size
  - ENSURE consistency throughout initialization and rendering
  
Task 2 - Fix Initialization Sequence:
MODIFY editor/src/editor_state.rs::new():
  - MOVE ImGui display size setting after renderer creation
  - FORCE display size from surface config immediately
  - ADD validation that sizes match before continuing
  
Task 3 - Add Scissor Rect Clamping:
MODIFY editor/src/editor_state.rs::render_ui_and_draw():
  - ADD scissor rect validation before imgui_renderer.render()
  - CLAMP any scissor rects that exceed render target bounds
  - LOG warnings when clamping occurs
  
Task 4 - Improve Display Scaling Handling:
MODIFY editor/src/editor_state.rs::handle_event():
  - ENHANCE ScaleFactorChanged event handling
  - ROUND sizes to avoid fractional pixels
  - UPDATE both display size and framebuffer scale
  
Task 5 - Add Robust Validation:
MODIFY editor/src/editor_state.rs::render_ui_and_draw():
  - VALIDATE all sizes before creating render pass
  - SKIP frame if any size mismatch detected
  - ADD detailed error logging for debugging
```

### Task 1: Refactor Size Handling
```rust
// BEFORE: Using window inner size (incorrect)
let size = window.inner_size();
io.display_size = [size.width as f32, size.height as f32];

// AFTER: Using surface configuration size (correct)
let surface_size = {
    let surface_config = render_context.surface_config.lock().unwrap();
    (surface_config.width, surface_config.height)
};
io.display_size = [surface_size.0 as f32, surface_size.1 as f32];
```

### Task 2: Fix Initialization
```rust
impl EditorState {
    pub fn new(render_context: &Arc<RenderContext>, window: &Window) -> Self {
        // ... create ImGui context ...
        
        // Get actual surface size from render context
        let surface_size = {
            let surface_config = render_context.surface_config.lock().unwrap();
            (surface_config.width, surface_config.height)
        };
        
        // Attach window first
        imgui_platform.attach_window(imgui_context.io_mut(), window, HiDpiMode::Default);
        
        // CRITICAL: Force correct size immediately after attachment
        let io = imgui_context.io_mut();
        io.display_size = [surface_size.0 as f32, surface_size.1 as f32];
        io.display_framebuffer_scale = [scale_factor, scale_factor];
        
        // Validate sizes match
        debug!(
            "ImGui initialized with surface size: {}x{} (window reports {}x{})",
            surface_size.0, surface_size.1, window.inner_size().width, window.inner_size().height
        );
        
        // ... rest of initialization ...
    }
}
```

### Task 3: Scissor Rect Clamping
```rust
// Add before imgui_renderer.render() call
fn clamp_draw_data(draw_data: &mut imgui::DrawData, max_width: f32, max_height: f32) {
    for draw_list in draw_data.draw_lists() {
        for cmd in draw_list.commands() {
            if let imgui::DrawCmd::Elements { clip_rect, .. } = cmd {
                // Check if clip rect exceeds bounds
                if clip_rect[2] > max_width || clip_rect[3] > max_height {
                    tracing::warn!(
                        "Clamping clip rect from [{}, {}, {}, {}] to fit in {}x{}",
                        clip_rect[0], clip_rect[1], clip_rect[2], clip_rect[3],
                        max_width, max_height
                    );
                    // Note: This is read-only in current imgui-rs API
                    // May need to use a different approach
                }
            }
        }
    }
}
```

### Task 4: Display Scaling
```rust
// In handle_event for ScaleFactorChanged
WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
    let surface_size = {
        let surface_config = render_context.surface_config.lock().unwrap();
        (surface_config.width, surface_config.height)
    };
    
    // Calculate logical size with proper rounding
    let logical_width = (surface_size.0 as f64 / scale_factor).round();
    let logical_height = (surface_size.1 as f64 / scale_factor).round();
    
    // Update ImGui with exact surface size
    let io = self.imgui_context.io_mut();
    io.display_size = [surface_size.0 as f32, surface_size.1 as f32];
    io.display_framebuffer_scale = [*scale_factor as f32, *scale_factor as f32];
    
    debug!(
        "Scale factor changed to {}: surface={}x{}, logical={}x{}",
        scale_factor, surface_size.0, surface_size.1, logical_width, logical_height
    );
}
```

### Task 5: Robust Validation
```rust
pub fn render_ui_and_draw(&mut self, /* params */) {
    // Get all relevant sizes
    let surface_size = {
        let surface_config = render_context.surface_config.lock().unwrap();
        (surface_config.width, surface_config.height)
    };
    let window_size = window.inner_size();
    let imgui_size = self.imgui_context.io().display_size;
    
    // Comprehensive validation
    let sizes_match = 
        surface_size.0 == window_size.width &&
        surface_size.1 == window_size.height &&
        surface_size.0 as f32 == imgui_size[0] &&
        surface_size.1 as f32 == imgui_size[1];
    
    if !sizes_match {
        tracing::error!(
            "Size mismatch detected! surface={}x{}, window={}x{}, imgui={:?}",
            surface_size.0, surface_size.1,
            window_size.width, window_size.height,
            imgui_size
        );
        
        // Force correct size and skip this frame
        self.imgui_context.io_mut().display_size = [surface_size.0 as f32, surface_size.1 as f32];
        return;
    }
    
    // ... continue with rendering ...
}
```

## Validation Loop

### Level 1: Build & Format
```bash
# Format and build with editor feature
cargo fmt --all
cargo clippy --workspace --all-features -- -D warnings
cargo build --features editor

# Expected: Clean build with no warnings
```

### Level 2: Basic Launch Test
```bash
# Run with debug logging to see size information
RUST_LOG=debug,wgpu_core=warn,wgpu_hal=warn cargo run --features editor

# Expected: Editor launches without scissor rect errors
# Look for: "ImGui initialized with surface size: 1280x720"
# No errors containing "Scissor Rect"
```

### Level 3: Display Scaling Test
```powershell
# On Windows, test with different display scaling
# 1. Set display scaling to 100% in Windows Settings
cargo run --features editor

# 2. Set display scaling to 125%
cargo run --features editor

# 3. Set display scaling to 150%  
cargo run --features editor

# Expected: No scissor rect errors at any scaling level
```

### Level 4: Window Resize Test
```bash
# Launch editor and resize window multiple times
cargo run --features editor

# Actions:
# 1. Drag window corners to resize
# 2. Maximize and restore window
# 3. Move window between monitors (if available)

# Expected: No crashes or validation errors during any resize
```

### Level 5: Stress Test
```bash
# Run with minimal frame skipping to stress test initialization
# Temporarily change SKIP_FRAMES from 5 to 1 in editor_state.rs
cargo run --features editor

# Expected: May see warnings but no crashes
```

## Final Validation Checklist
- [ ] Editor launches successfully on Windows
- [ ] No scissor rect validation errors in logs
- [ ] Viewport displays game content correctly
- [ ] Window resizing works without errors
- [ ] Display scaling (100%, 125%, 150%) all work
- [ ] No `println!` statements - only tracing macros
- [ ] All existing tests pass
- [ ] Documentation updated with any new caveats

---

## Anti-Patterns to Avoid
- ❌ Don't use window.inner_size() for render calculations
- ❌ Don't assume ImGui display size is correct without verification
- ❌ Don't skip validation "because it should work"
- ❌ Don't ignore scale factor changes
- ❌ Don't hardcode any sizes or assume 1:1 pixel ratios
- ❌ Don't panic on size mismatches - skip frame instead

## Critical Implementation Notes

### Size Hierarchy
1. **Surface Configuration**: The source of truth for render target size
2. **Window Inner Size**: May differ due to window decorations or OS quirks  
3. **ImGui Display Size**: Must be forced to match surface configuration
4. **Scissor Rects**: Must fit within surface configuration bounds

### Timing Considerations
- Window creation and surface initialization may happen asynchronously
- Scale factor changes can arrive at any time
- First few frames may have incorrect sizes - defensive programming required
- Some window managers report sizes differently

### Platform Differences
- Windows: Display scaling commonly causes issues (125%, 150%, etc.)
- Linux/i3: Window manager may cause 1-pixel boundary issues
- macOS: Retina displays have 2x scale factor by default

## Success Metrics
- Zero scissor rect validation errors across all platforms
- Consistent 60 FPS with no frame skips after initialization
- Clear debug logging showing size synchronization
- Graceful handling of edge cases without crashes

## Confidence Score: 9/10
High confidence because:
- Issue is well-documented in multiple projects
- Clear root cause identified (size mismatch)
- Multiple defensive strategies provided
- Comprehensive validation steps
- Similar issues resolved in other imgui-wgpu projects

Minor uncertainty (-1) due to:
- Exact timing of winit events on Windows
- Potential platform-specific quirks not yet discovered