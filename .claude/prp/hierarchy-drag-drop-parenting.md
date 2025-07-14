name: "Hierarchy Panel Drag-and-Drop Parenting Implementation"
description: |

## Purpose
Add drag-and-drop functionality to the hierarchy panel to enable users to create parent-child entity relationships by dragging entities onto each other in the editor.

## Core Principles
1. **Context is King**: Include ALL necessary documentation, examples, and caveats
2. **Validation Loops**: Provide executable tests/lints the AI can run and fix
3. **Information Dense**: Use keywords and patterns from the codebase
4. **Progressive Success**: Start simple, validate, then enhance
5. **Global rules**: Be sure to follow all rules in CLAUDE.md

---

## Goal
Enable users to parent entities in the editor by dragging one entity onto another in the hierarchy panel, with proper visual feedback and cycle prevention.

## Why
- Currently no UI way to create parent-child relationships in editor
- Users must manually edit scene files or use code
- Drag-and-drop is intuitive and standard in game editors
- Improves workflow for scene composition and organization

## What
Users can drag any entity in the hierarchy panel and drop it onto another entity to make it a child. Visual feedback shows valid drop targets. System prevents invalid operations like creating cycles.

### Success Criteria
- [ ] Can drag entities in hierarchy panel
- [ ] Can drop entities onto other entities to parent them
- [ ] Visual feedback during drag operation
- [ ] Prevents dragging parent onto its own descendant
- [ ] Can remove parent relationship (drag to root level)
- [ ] Hierarchy updates immediately after drop

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/imgui/latest/imgui/drag_drop/index.html
  why: ImGui drag-drop API reference for Rust bindings
  
- file: editor/src/panels/assets.rs
  why: Existing drag-drop implementation pattern with static state
  lines: 36-238
  
- file: editor/src/panels/inspector.rs  
  why: Drop target implementation pattern and visual feedback
  lines: 300-400
  
- file: engine/src/core/entity/hierarchy.rs
  why: Hierarchy system and cycle detection patterns
  lines: 1-100

- file: editor/src/panels/hierarchy.rs
  why: Target file to modify - understand current structure
  
- url: https://github.com/imgui-rs/imgui-rs/blob/main/imgui-examples/examples/drag_drop.rs
  why: ImGui-rs drag-drop examples showing proper API usage
```

### Current Codebase Structure
```bash
editor/src/panels/
├── assets.rs         # Has drag-drop for files
├── hierarchy.rs      # TARGET FILE - needs drag-drop
├── inspector.rs      # Has drop targets
└── ...

engine/src/core/entity/
├── components.rs     # Parent component definition
├── hierarchy.rs      # Hierarchy update system
└── world.rs          # World mutation patterns
```

### Desired Implementation Structure
```bash
editor/src/panels/
├── hierarchy.rs      # MODIFIED - Added drag-drop functionality
│   ├── HierarchyDragState struct (new)
│   ├── HIERARCHY_DRAG_STATE static (new)
│   ├── is_ancestor_of() helper (new)
│   └── render_entity_tree() with drag-drop (modified)
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: ImGui drag-drop requires static state between frames
// Pattern: Use static mut with Option<State> like ASSET_BROWSER_STATE

// CRITICAL: Must use world.inner.remove_one::<Parent>(entity) for component removal
// NOT world.remove_one() which doesn't exist

// CRITICAL: EditorSharedState requires with_world_write for mutations
// Read-only access uses with_world_read

// CRITICAL: Drag condition should be Condition::Once to start drag
// Drop accepts with DragDropFlags::empty()

// CRITICAL: Entity handles must be stored/retrieved carefully
// Use hecs::Entity type directly in static state
```

## Implementation Blueprint

### Data Models and Structure

```rust
// Add to hierarchy.rs after imports
#[derive(Debug)]
struct HierarchyDragState {
    dragged_entity: Option<hecs::Entity>,
    drag_source_name: String, // For display during drag
}

impl HierarchyDragState {
    fn new() -> Self {
        Self {
            dragged_entity: None,
            drag_source_name: String::new(),
        }
    }
}

// Global state (following assets.rs pattern)
static mut HIERARCHY_DRAG_STATE: Option<HierarchyDragState> = None;
```

### List of Tasks

```yaml
Task 1: Add drag state management
MODIFY editor/src/panels/hierarchy.rs:
  - ADD HierarchyDragState struct after imports
  - ADD static mut HIERARCHY_DRAG_STATE
  - ADD helper functions to get/set dragged entity
  - PATTERN: Follow ASSET_BROWSER_STATE from assets.rs

Task 2: Add is_ancestor_of helper function
MODIFY editor/src/panels/hierarchy.rs:
  - ADD is_ancestor_of(world, potential_parent, potential_child) -> bool
  - USE breadth-first search with visited set
  - PREVENT cycles by checking ancestry

Task 3: Add drag source to entities
MODIFY render_entity_tree function:
  - AFTER selectable/tree_node rendering
  - ADD drag_drop_source_config("ENTITY_PARENT")
  - STORE entity and name in static state when drag starts
  - SHOW dragging feedback text

Task 4: Add drop target to entities
MODIFY render_entity_tree function:
  - WRAP selectable/tree_node in drop target check
  - ADD visual feedback rectangle on hover
  - VALIDATE drop (not self, not ancestor)
  - EXECUTE parent change on valid drop

Task 5: Handle drop with validation
IMPLEMENT drop handling logic:
  - GET dragged entity from static state
  - CHECK is_ancestor_of to prevent cycles
  - USE with_world_write to modify Parent component
  - HANDLE remove parent (drop on empty space)
  - CLEAR drag state after drop

Task 6: Add visual polish
ENHANCE user experience:
  - COLOR code valid/invalid drop targets
  - ADD tooltip explaining why drop is invalid
  - ENSURE hierarchy refreshes after drop
```

### Task 1: Drag State Management
```rust
// After imports in hierarchy.rs
#[derive(Debug)]
struct HierarchyDragState {
    dragged_entity: Option<hecs::Entity>,
    drag_source_name: String,
}

static mut HIERARCHY_DRAG_STATE: Option<HierarchyDragState> = None;

// Helper functions (following assets.rs pattern)
fn get_hierarchy_drag_state() -> &'static mut HierarchyDragState {
    unsafe {
        if HIERARCHY_DRAG_STATE.is_none() {
            HIERARCHY_DRAG_STATE = Some(HierarchyDragState {
                dragged_entity: None,
                drag_source_name: String::new(),
            });
        }
        HIERARCHY_DRAG_STATE.as_mut().unwrap()
    }
}
```

### Task 2: Ancestry Check Helper
```rust
// Add to hierarchy.rs
fn is_ancestor_of(
    world: &World,
    potential_ancestor: hecs::Entity,
    potential_descendant: hecs::Entity,
) -> bool {
    // Check if potential_ancestor is an ancestor of potential_descendant
    let mut current = Some(potential_descendant);
    let mut visited = HashSet::new();
    
    while let Some(entity) = current {
        if entity == potential_ancestor {
            return true;
        }
        
        if !visited.insert(entity) {
            // Cycle detected
            return false;
        }
        
        // Get parent of current entity
        current = world.get::<Parent>(entity)
            .ok()
            .map(|parent| parent.0);
    }
    
    false
}
```

### Task 3-4: Drag and Drop Implementation
```rust
// In render_entity_tree function, after selectable/tree rendering

// Make entity draggable
if ui.drag_drop_source_config("ENTITY_PARENT")
    .condition(Condition::Once)
    .begin()
    .is_some()
{
    let state = get_hierarchy_drag_state();
    state.dragged_entity = Some(entity);
    state.drag_source_name = entity_name.clone();
    
    // Visual feedback during drag
    ui.text(format!("🔗 {}", entity_name));
}

// Make entity a drop target
if let Some(target) = ui.drag_drop_target() {
    let state = get_hierarchy_drag_state();
    
    if let Some(dragged) = state.dragged_entity {
        // Visual feedback when hovering
        let can_drop = dragged != entity && 
            !shared_state.with_world_read(|world| {
                is_ancestor_of(world, dragged, entity)
            }).unwrap_or(false);
        
        if ui.is_item_hovered() {
            let color = if can_drop {
                [0.0, 1.0, 0.0, 0.5] // Green for valid
            } else {
                [1.0, 0.0, 0.0, 0.5] // Red for invalid
            };
            
            ui.get_window_draw_list()
                .add_rect(
                    ui.item_rect_min(),
                    ui.item_rect_max(),
                    color,
                )
                .build();
        }
        
        // Accept drop
        if target.accept_payload_empty("ENTITY_PARENT", DragDropFlags::empty()).is_some() {
            if can_drop {
                // Perform the parenting
                shared_state.with_world_write(|world| {
                    // Remove existing parent if any
                    let _ = world.inner.remove_one::<Parent>(dragged);
                    // Add new parent
                    let _ = world.insert_one(dragged, Parent(entity));
                });
                
                // Clear drag state
                state.dragged_entity = None;
                state.drag_source_name.clear();
            }
        }
    }
}
```

### Integration Points
```yaml
COMPONENTS:
  - uses: Parent component from engine::prelude
  - pattern: "Parent(parent_entity)"
  
WORLD_ACCESS:
  - read: shared_state.with_world_read(|world| {...})
  - write: shared_state.with_world_write(|world| {...})
  
IMGUI_API:
  - drag_source: drag_drop_source_config("ENTITY_PARENT")
  - drop_target: drag_drop_target()
  - accept: accept_payload_empty("ENTITY_PARENT", DragDropFlags::empty())
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Compilation
```bash
# Ensure it compiles
cargo check --workspace

# Build the editor
cargo build --bin editor

# Expected: Successful compilation
```

### Level 3: Runtime Testing
```bash
# Run the editor
cargo run --bin editor

# Manual test checklist:
# 1. Open hierarchy panel
# 2. Create multiple entities (some with parent-child relationships)
# 3. Test drag operations:
#    - Drag child to new parent ✓
#    - Drag parent to its own child (should show red, not allow) ✓
#    - Drag entity to itself (should show red, not allow) ✓
#    - Drag to empty space (removes parent) ✓
# 4. Verify hierarchy updates after each operation
# 5. Check no crashes or panics in console
```

### Level 4: Edge Cases
```rust
// Test scenarios to validate:
// 1. Dragging root entity to child
// 2. Dragging between deeply nested hierarchies  
// 3. Dragging when entity has scripts/components
// 4. Multiple rapid drag operations
// 5. Dragging during scene reload
```

## Final Validation Checklist
- [ ] All clippy warnings resolved
- [ ] Can drag any entity in hierarchy
- [ ] Drop shows green highlight for valid targets
- [ ] Drop shows red highlight for invalid targets
- [ ] Cannot create cycles (parent to child)
- [ ] Cannot parent to self
- [ ] Hierarchy refreshes after drop
- [ ] Can remove parent by dropping on root
- [ ] No panics or crashes during operations

---

## Anti-Patterns to Avoid
- ❌ Don't use non-static state for drag data (won't persist between frames)
- ❌ Don't allow cycles in hierarchy (causes infinite loops)
- ❌ Don't forget to clear drag state after drop
- ❌ Don't modify world outside with_world_write
- ❌ Don't use world.remove_one (use world.inner.remove_one)
- ❌ Don't skip visual feedback (confusing UX)

---

## Score: 8/10

High confidence in one-pass implementation due to:
- Clear examples from existing drag-drop code
- Well-defined patterns to follow
- Straightforward validation approach
- All edge cases identified

Minor deductions for:
- Static mut pattern is tricky to get right
- Potential for subtle ancestry check bugs
- ImGui drag-drop API has some quirks