name: "Window Docking Implementation for Editor Panels"
description: |

## Purpose
Implement window docking functionality that allows editor panels to snap and attach to main window borders, creating proper layouts instead of floating panels.

## Core Principles
1. **Preserve Existing Functionality**: Panels must still be draggable and floatable
2. **Edge Snapping**: Panels should snap to window edges when dragged near borders
3. **Resize Aware**: Docked panels must maintain their docked positions during window resize
4. **State Persistence**: Docked states must be saved/loaded with layouts
5. **Global rules**: Follow all rules in CLAUDE.md including tracing for logging

---

## Goal
Transform the current floating panel system into a dockable panel system where panels can snap to the main window borders, creating professional-looking layouts similar to modern IDEs while preserving the ability to float panels when needed.

## Why
- **Professional UI**: Floating panels look unprofessional and disorganized
- **Space Efficiency**: Docked panels use screen space more efficiently
- **User Experience**: Standard behavior in modern editors (VS Code, Unity, Unreal)
- **Layout Consistency**: Docked panels maintain positions relative to window edges

## What
Implement a docking system that:
- Detects when panels are dragged near window borders (10-20px threshold)
- Snaps panels to edges with visual feedback
- Maintains docked positions during window resize
- Allows undocking by dragging panels away from edges
- Saves/loads docked states in layout JSON

### Success Criteria
- [ ] Panels snap to all four window edges when dragged within threshold
- [ ] Docked panels resize appropriately with window
- [ ] Panels can be undocked by dragging away from edges
- [ ] Docked state persists in layout saves/loads
- [ ] No visual glitches or panel overlap
- [ ] Performance remains smooth during dragging

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://github.com/ocornut/imgui/issues/2109
  why: Discusses window docking implementation approaches in ImGui
  
- url: https://gist.github.com/Shuppin/f10639436a68654aadfdd9d3426dc4ad
  why: Example of programmatic docking with imgui-rs (shows API usage patterns)
  
- url: https://github.com/ocornut/imgui/wiki/Docking
  why: Official ImGui docking documentation explaining concepts
  
- file: editor/src/panel_state.rs
  why: Current panel state management - need to extend for docking
  
- file: editor/src/panels/hierarchy.rs
  why: Example panel implementation showing position update pattern (lines 117-122)
  
- file: editor_layout.json
  why: Current layout format - need to add docked state fields
  
- doc: https://docs.rs/imgui/latest/imgui/
  section: Window positioning and conditions
  critical: FirstUseEver condition allows manual positioning after initial placement
```

### Current Codebase Structure
```bash
editor/
├── src/
│   ├── panel_state.rs          # Panel state management (position, size, visibility)
│   ├── panels/                 # Individual panel implementations
│   │   ├── mod.rs
│   │   ├── hierarchy.rs        # Example: updates position after drag
│   │   ├── inspector.rs
│   │   ├── viewport.rs
│   │   └── assets.rs
│   ├── editor_state.rs         # Main editor loop, renders panels
│   └── lib.rs
editor_layout.json              # Persisted panel layouts
```

### Desired Structure with New Files
```bash
editor/
├── src/
│   ├── docking/               # NEW: Docking system
│   │   ├── mod.rs            # Module exports
│   │   ├── dock_zone.rs      # Edge detection and snap zones
│   │   └── docked_state.rs   # Docked panel state management
│   ├── panel_state.rs        # MODIFY: Add docking fields
│   └── ...existing files...
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: imgui 0.12 has docking feature but limited Rust API
// - No direct DockSpace/DockNode API in safe Rust
// - Must implement edge snapping manually
// - Window conditions affect dragging behavior

// GOTCHA: Panel position updates happen AFTER imgui window build
// - See hierarchy.rs lines 117-122 for pattern
// - Must track drag state across frames

// PATTERN: Use tracing for all logging (NO println!)
use tracing::{debug, info, warn};

// IMPORTANT: Window resize events from winit need special handling
// - Scale factor changes affect positions
// - Must distinguish logical vs physical pixels
```

## Implementation Blueprint

### Data Models and Structure

```rust
// In docking/docked_state.rs
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DockEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockedState {
    pub edge: DockEdge,
    pub offset: f32,        // Offset along the edge (0.0-1.0)
    pub size: f32,          // Size perpendicular to edge
}

// In panel_state.rs - extend PanelLayout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayout {
    pub id: PanelId,
    pub title: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub is_visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docked: Option<DockedState>,  // NEW FIELD
}

// In docking/dock_zone.rs
pub struct DockZone {
    pub edge: DockEdge,
    pub threshold: f32,  // Distance from edge to trigger snapping
}
```

### Implementation Tasks

```yaml
Task 1 - Create Docking Module Structure:
CREATE editor/src/docking/mod.rs:
  - Export public types and functions
  - Document module purpose

CREATE editor/src/docking/dock_zone.rs:
  - Define DockZone struct
  - Implement edge detection logic
  - Add snap threshold calculations

CREATE editor/src/docking/docked_state.rs:
  - Define DockedState and DockEdge enums
  - Add serialization support
  - Implement position calculation from docked state

Task 2 - Extend Panel State:
MODIFY editor/src/panel_state.rs:
  - FIND: "pub struct PanelLayout"
  - ADD: "pub docked: Option<DockedState>" field with serde attributes
  - UPDATE: from_layout() to handle docked state
  - UPDATE: to_layout() to save docked state
  - ADD: calculate_docked_position() method

Task 3 - Add Docking Detection:
MODIFY editor/src/panel_state.rs:
  - ADD: PanelState fields for drag tracking
  - ADD: check_dock_zones() method
  - ADD: apply_docking() method
  - ADD: clear_docking() method

Task 4 - Update Panel Rendering:
MODIFY editor/src/panels/hierarchy.rs (and other panels):
  - PATTERN: Follow existing position update (lines 117-122)
  - ADD: Docking detection during drag
  - ADD: Visual feedback for snap zones
  - UPDATE: Position calculation for docked panels

Task 5 - Handle Window Resize:
MODIFY editor/src/editor_state.rs:
  - FIND: Window resize event handling
  - ADD: Update docked panel positions on resize
  - PRESERVE: Relative positions along edges

Task 6 - Update Layout Persistence:
VERIFY editor_layout.json compatibility:
  - Backward compatible (old layouts still load)
  - New docked field saves/loads correctly
  
Task 7 - Add Tests:
CREATE editor/src/docking/tests.rs:
  - Test edge detection logic
  - Test position calculations
  - Test serialization/deserialization
```

### Task Pseudocode

```rust
// Task 2 - Panel State Extensions
impl PanelState {
    /// Calculate position based on docked state and window size
    pub fn calculate_docked_position(&self, window_size: (f32, f32)) -> (f32, f32) {
        match &self.docked {
            Some(docked) => {
                // PATTERN: Use window_size to calculate absolute position
                match docked.edge {
                    DockEdge::Left => (0.0, window_size.1 * docked.offset),
                    DockEdge::Right => (window_size.0 - self.size.0, window_size.1 * docked.offset),
                    DockEdge::Top => (window_size.0 * docked.offset, 0.0),
                    DockEdge::Bottom => (window_size.0 * docked.offset, window_size.1 - self.size.1),
                }
            }
            None => self.position, // Not docked, use stored position
        }
    }
}

// Task 3 - Docking Detection
impl DockZone {
    pub fn check_snap(&self, panel_pos: (f32, f32), panel_size: (f32, f32), window_size: (f32, f32)) -> Option<DockedState> {
        // CRITICAL: Check distance from edge based on panel position + size
        let threshold = 20.0; // pixels
        
        match self.edge {
            DockEdge::Left => {
                if panel_pos.0 < threshold {
                    Some(DockedState {
                        edge: DockEdge::Left,
                        offset: panel_pos.1 / window_size.1, // Normalize to 0-1
                        size: panel_size.0,
                    })
                } else { None }
            }
            // Similar for other edges...
        }
    }
}

// Task 4 - Panel Update Pattern
// In each panel's render function:
if ui.is_window_hovered() && ui.is_mouse_dragging(MouseButton::Left) {
    // Track dragging
    let new_pos = ui.window_pos();
    
    // Check dock zones
    if let Some(docked) = check_dock_zones(new_pos, panel_size, window_size) {
        // Visual feedback - tint window or show guide
        ui.get_window_draw_list().add_rect(...); 
    }
}

// After window build - update state
if let Some(panel) = panel_manager.get_panel_mut(&panel_id) {
    if !ui.is_mouse_down(MouseButton::Left) && was_dragging {
        // Mouse released - apply docking if in zone
        if let Some(docked) = pending_dock {
            panel.docked = Some(docked);
            info!(panel = ?panel_id, edge = ?docked.edge, "Panel docked");
        }
    }
}
```

### Integration Points
```yaml
PANEL_MANAGER:
  - add to: update_docked_positions() method
  - call on: window resize events
  
EDITOR_STATE:
  - modify: handle_event() for resize tracking
  - add: window_size tracking for dock calculations
  
LAYOUT_FORMAT:
  - backward compatible: old files load without docked field
  - forward: new saves include docked state
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cd editor
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// In editor/src/docking/tests.rs
#[test]
fn test_dock_zone_detection() {
    let zone = DockZone { edge: DockEdge::Left, threshold: 20.0 };
    let docked = zone.check_snap((10.0, 100.0), (200.0, 300.0), (800.0, 600.0));
    assert!(docked.is_some());
    assert_eq!(docked.unwrap().edge, DockEdge::Left);
}

#[test]
fn test_docked_position_calculation() {
    let docked = DockedState { 
        edge: DockEdge::Top, 
        offset: 0.5, 
        size: 300.0 
    };
    let panel = PanelState { docked: Some(docked), ..Default::default() };
    let pos = panel.calculate_docked_position((800.0, 600.0));
    assert_eq!(pos, (400.0, 0.0)); // Centered on top edge
}

#[test]
fn test_layout_serialization() {
    let layout = PanelLayout {
        docked: Some(DockedState { edge: DockEdge::Right, offset: 0.3, size: 250.0 }),
        ..Default::default()
    };
    let json = serde_json::to_string(&layout).unwrap();
    let parsed: PanelLayout = serde_json::from_str(&json).unwrap();
    assert_eq!(layout.docked, parsed.docked);
}
```

```bash
# Run tests
cargo test --package editor -- docking

# Run specific test
cargo test --package editor -- test_dock_zone_detection
```

### Level 3: Integration Test
```bash
# Build and run editor
cargo build --package editor
cargo run --bin editor

# Manual testing checklist:
# 1. Drag Hierarchy panel to left edge - should snap
# 2. Drag Inspector panel to right edge - should snap  
# 3. Resize window - docked panels should stay at edges
# 4. Save layout (Ctrl+S or menu)
# 5. Restart editor - panels should restore docked
# 6. Drag docked panel away from edge - should undock
```

### Level 4: Full Validation
```bash
# Full preflight check (from project root)
just preflight

# Expected: All green
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace`
- [ ] Panels snap to all 4 edges correctly
- [ ] Window resize maintains docked positions
- [ ] Layout save/load preserves docked states
- [ ] No visual glitches during drag/dock
- [ ] Performance smooth (60 FPS maintained)
- [ ] Tracing logs provide useful debugging info

---

## Anti-Patterns to Avoid
- ❌ Don't use println! - use tracing crate
- ❌ Don't hardcode pixel values - use configurable thresholds
- ❌ Don't break existing layouts - maintain backward compatibility
- ❌ Don't ignore imgui conditions - they affect behavior
- ❌ Don't assume window size - track resize events
- ❌ Don't block during drag operations - stay responsive

## Confidence Score: 8/10

The implementation is well-defined with clear patterns to follow. The main complexity is in the manual edge detection since imgui-rs doesn't expose full docking API. Following the existing panel update patterns and maintaining backward compatibility should lead to successful implementation.