name: "Fix Editor Inspector and Hierarchy Panel Entity Component Display"
description: |
  Complete fix for editor inspector and hierarchy panels not displaying entity components and names correctly

## Goal
Fix the editor's Inspector and Hierarchy panels to properly display entity components and names. The panels should show entities with their proper names (e.g., "Main Camera", "Center Cube", "Ground Plane") and allow selection/editing of components in the inspector.

## Why
- **User Experience**: Essential editor functionality is broken, preventing users from inspecting and editing entities
- **Development Productivity**: Without working inspector/hierarchy, debugging and scene editing becomes extremely difficult
- **Feature Completeness**: Core editor features should work reliably for a functional game development workflow

## What
Fix three critical issues:
1. Hierarchy panel showing all entities as "Entity ... [No Transform]" instead of proper names
2. Inspector panel not displaying components when entities are selected
3. Component editing functionality not working properly

### Success Criteria
- [ ] Hierarchy panel displays entity names correctly ("Main Camera", "Center Cube", "Ground Plane", etc.)
- [ ] Selecting entities in hierarchy populates inspector with components
- [ ] Inspector shows collapsible headers for each component (Name, Transform, Material, Camera, Mesh)
- [ ] Component values are editable in the inspector
- [ ] Changes are properly saved and reflected in the scene
- [ ] Debug logging confirms entities exist with expected components

## All Needed Context

### Documentation & References
```yaml
- url: https://docs.rs/hecs/latest/hecs/
  why: Official ECS library documentation for understanding query patterns and world access
  critical: Archetype-based querying, component storage patterns
  
- url: https://github.com/imgui-rs/imgui-rs
  why: ImGui Rust bindings for UI patterns and state management
  critical: Immediate mode GUI state management, widget lifetime
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/CLAUDE.md
  why: Project conventions, testing requirements, mandatory protocols
  critical: Must run 'just preflight' after changes, testing protocol
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/editor/src/panels/inspector.rs
  why: Current inspector implementation showing component checking patterns
  critical: SharedState world access, component editing patterns
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/editor/src/panels/hierarchy.rs
  why: Current hierarchy implementation showing entity name retrieval
  critical: Entity querying, name display logic, selection handling
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/editor/src/shared_state.rs
  why: Thread-safe world access patterns using Arc<Mutex<World>>
  critical: with_world_read/write methods, entity selection state management
```

### Current Codebase Structure
```bash
webgpu-template/
├── engine/src/
│   ├── core/entity/
│   │   ├── components.rs    # Name, Transform, GlobalTransform components
│   │   ├── world.rs         # ECS World wrapper
│   │   └── hierarchy.rs     # Parent-child relationships
│   └── prelude.rs           # Common imports including Name component
├── editor/src/
│   ├── panels/
│   │   ├── inspector.rs     # Component inspector panel (BROKEN)
│   │   └── hierarchy.rs     # Scene hierarchy panel (BROKEN)
│   ├── shared_state.rs      # Thread-safe world access
│   └── scene_operations.rs  # Scene creation/loading
└── game/src/
    └── main.rs              # Demo scene creation with named entities
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: Avoid nested world access calls in imgui-rs immediate mode
// BAD: with_world_write -> with_world_read (causes deadlock)
shared_state.with_world_write(|world| {
    shared_state.with_world_read(|inner_world| { ... }) // DEADLOCK!
});

// GOOD: Check components first, then edit
let (has_name, has_transform) = shared_state.with_world_read(|world| {
    (world.get::<&Name>(entity).is_ok(), world.get::<&Transform>(entity).is_ok())
}).unwrap_or((false, false));

if has_name {
    shared_state.with_world_write(|world| { /* edit name */ });
}

// CRITICAL: HECS component removal/insertion pattern for editing
// Use world.inner_mut().remove_one::<T>() and world.insert_one() for mutation
if let Ok(mut component) = world.inner_mut().remove_one::<Component>(entity) {
    // Edit component
    component.field = new_value;
    // Re-insert component
    let _ = world.insert_one(entity, component);
}

// CRITICAL: Scene clearing issue - create_default_scene() clears world
// This may be overwriting demo scene entities with Name components
// Location: editor/src/scene_operations.rs:17 calls world.inner_mut().clear()

// GOTCHA: Static mutable state in inspector.rs needs proper handling
unsafe static mut INSPECTOR_STATE: Option<InspectorState> = None;
// Should be refactored to use proper state management patterns
```

## Implementation Blueprint

### Root Cause Analysis
The issue appears to be timing-related where `create_default_scene()` is being called and clearing the world after the demo scene with named entities is created. This happens when:
1. Demo scene is created with proper Name components in `game/src/main.rs:594-648`
2. Editor is initialized and may trigger `SceneOperation::NewScene`
3. `create_default_scene()` clears the world at `editor/src/scene_operations.rs:17`
4. New entities without proper Name components are created

### Task List (Execute in Order)

```yaml
Task 1: Debug Entity Creation and World State
MODIFY editor/src/panels/hierarchy.rs:
  - ADD comprehensive debug logging to track entity counts and component presence
  - PRESERVE existing query patterns
  - IDENTIFY if entities exist and have expected components

Task 2: Fix Scene Creation Race Condition  
INVESTIGATE game/src/main.rs initialization flow:
  - VERIFY when create_demo_scene() vs create_default_scene() are called
  - ENSURE demo scene entities persist in editor mode
  - PREVENT automatic scene clearing in editor startup

Task 3: Fix Inspector Component Detection
MODIFY editor/src/panels/inspector.rs:
  - VERIFY component checking logic works correctly
  - FIX any issues with SharedState world access patterns
  - ENSURE component editing follows proper remove/insert pattern

Task 4: Fix Hierarchy Entity Name Display
MODIFY editor/src/panels/hierarchy.rs:
  - VERIFY get_entity_name() function works correctly
  - FIX entity filtering logic for Transform components
  - ENSURE proper Name component reading

Task 5: Add Comprehensive Testing
CREATE test cases to verify:
  - Entity creation with Name components
  - Component querying through SharedState
  - UI panel functionality with mock entities
```

### Task 1: Debug Entity Creation and World State
```rust
// Add to hierarchy.rs render function
shared_state.with_world_read(|world| {
    let total_entities = world.query::<()>().iter().count();
    let entities_with_name = world.query::<&Name>().iter().count();
    let entities_with_transform = world.query::<&Transform>().iter().count();
    
    eprintln!("DEBUG: Total entities: {}, with Name: {}, with Transform: {}", 
              total_entities, entities_with_name, entities_with_transform);
    
    // Log first few entities with their components
    for (entity, _) in world.query::<()>().iter().take(5) {
        let has_name = world.get::<&Name>(entity).is_ok();
        let has_transform = world.get::<&Transform>(entity).is_ok();
        let name = if has_name { 
            world.get::<&Name>(entity).map(|n| n.0.clone()).unwrap_or_default() 
        } else { "NO_NAME".to_string() };
        
        eprintln!("  Entity {:?}: Name='{}' ({}), Transform={}", 
                  entity, name, has_name, has_transform);
    }
});
```

### Task 2: Fix Scene Creation Race Condition
```rust
// In game/src/main.rs, verify editor initialization sequence
#[cfg(feature = "editor")]
let editor_state = {
    // CRITICAL: Demo scene is created BEFORE moving world to editor
    // Ensure this world (with named entities) is preserved
    let world = std::mem::replace(&mut self.world, World::new());
    
    EditorState::new(
        &render_context,
        &window,
        surface_config.format,
        (window_size.width, window_size.height),
        world, // This world should contain demo scene entities
    )
};

// PREVENT automatic scene clearing by checking if scene operations are triggered
// In main loop, add condition to prevent unwanted scene clearing:
if let Some(operation) = editor_state.pending_scene_operation.take() {
    match operation {
        SceneOperation::NewScene => {
            // Only clear if explicitly requested by user, not automatically
            if editor_state.user_requested_new_scene {
                editor_state.shared_state.with_world_write(|world| {
                    editor::scene_operations::create_default_scene(world, renderer);
                });
            }
        }
        // ... other operations
    }
}
```

### Task 3: Fix Inspector Component Detection
```rust
// In inspector.rs, ensure proper component checking
let (has_name, has_transform, has_camera, has_material, has_mesh) = 
    shared_state.with_world_read(|world| {
        let components = (
            world.get::<&Name>(entity).is_ok(),
            world.get::<&Transform>(entity).is_ok(),
            world.get::<&Camera>(entity).is_ok(),
            world.get::<&Material>(entity).is_ok(),
            world.get::<&MeshId>(entity).is_ok(),
        );
        eprintln!("Entity {:?} components: Name={}, Transform={}, Camera={}, Material={}, Mesh={}", 
                  entity, components.0, components.1, components.2, components.3, components.4);
        components
    }).unwrap_or((false, false, false, false, false));

// Verify each component display block works
if has_name {
    if ui.collapsing_header("Name", TreeNodeFlags::DEFAULT_OPEN) {
        shared_state.with_world_write(|world| {
            if let Ok(mut name) = world.inner_mut().remove_one::<Name>(entity) {
                let mut name_buffer = name.0.clone();
                if ui.input_text("##name", &mut name_buffer).build() {
                    name.0 = name_buffer;
                    shared_state.mark_scene_modified();
                }
                let _ = world.insert_one(entity, name);
            }
        });
    }
} else {
    eprintln!("WARNING: Entity {:?} missing Name component", entity);
}
```

### Task 4: Fix Hierarchy Entity Name Display
```rust
// In hierarchy.rs get_entity_name function, add debugging
fn get_entity_name(world: &World, entity: hecs::Entity) -> String {
    // Try Name component first
    if let Ok(name) = world.get::<&Name>(entity) {
        if !name.0.is_empty() {
            eprintln!("Found name '{}' for entity {:?}", name.0, entity);
            return name.0.clone();
        }
    }
    
    eprintln!("No name found for entity {:?}, checking Transform...", entity);
    
    // Fallback to ID with component indicator
    if world.get::<&Transform>(entity).is_ok() {
        format!("Entity {entity:?}")
    } else {
        format!("Entity {entity:?} [No Transform]")
    }
}
```

### Integration Points
```yaml
SCENE_MANAGEMENT:
  - Ensure demo scene entities persist in editor mode
  - Prevent automatic scene clearing without user confirmation
  - Add flags to track user-initiated vs automatic scene operations

UI_STATE:
  - Fix static mutable state in inspector.rs to use proper patterns
  - Ensure entity selection state is properly synchronized
  - Add error handling for world access failures

DEBUGGING:
  - Add comprehensive logging to track entity lifecycle
  - Log component presence and world state changes
  - Add validation checkpoints in entity creation flow
```

## Validation Loop

### Level 1: Compilation & Basic Syntax
```bash
# Must pass without errors before proceeding
cargo check --features editor
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Level 2: Unit Tests for Component Access
```rust
// CREATE tests/editor_panels_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use engine::prelude::*;
    
    #[test]
    fn test_entity_with_name_component() {
        let mut world = World::new();
        let entity = world.spawn((
            Name::new("Test Entity"),
            Transform::default(),
        ));
        
        // Verify component exists
        assert!(world.get::<&Name>(entity).is_ok());
        let name = world.get::<&Name>(entity).unwrap();
        assert_eq!(name.0, "Test Entity");
    }
    
    #[test]
    fn test_shared_state_world_access() {
        let world = World::new();
        let shared_state = EditorSharedState::new(world);
        
        // Test read access works
        let result = shared_state.with_world_read(|world| {
            world.query::<()>().iter().count()
        });
        assert!(result.is_some());
    }
    
    #[test]
    fn test_get_entity_name_function() {
        let mut world = World::new();
        let entity = world.spawn((Name::new("Test Name"),));
        
        let name = get_entity_name(&world, entity);
        assert_eq!(name, "Test Name");
    }
}
```

```bash
# Run tests and ensure they pass
cargo test --workspace
```

### Level 3: Integration Test with Editor
```bash
# Build and run editor to manually verify fixes
cargo build --features editor
timeout 30s cargo run --features editor 2>&1 | grep -E "(DEBUG:|Total entities:|Entity.*components:)" | head -20

# Expected output should show:
# - Entities with Name components detected
# - Proper entity names in hierarchy
# - Component detection working in inspector
```

### Level 4: Manual Testing Checklist
```bash
# Start editor and verify:
cargo run --features editor

# Manual verification steps:
# 1. Open hierarchy panel - should show "Main Camera", "Center Cube", "Ground Plane"
# 2. Select entity in hierarchy - inspector should populate with components
# 3. Edit entity name in inspector - should update in hierarchy
# 4. Edit transform values - should update in scene
# 5. Add new components via inspector dropdown - should work correctly
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No compilation errors: `cargo check --features editor`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features`
- [ ] Manual test: Hierarchy shows proper entity names
- [ ] Manual test: Inspector shows components for selected entity
- [ ] Manual test: Component editing works correctly
- [ ] Debug output confirms entities exist with expected components
- [ ] No performance regressions in UI responsiveness

---

## Anti-Patterns to Avoid
- ❌ Don't use nested `with_world_read`/`with_world_write` calls (deadlock risk)
- ❌ Don't ignore the world clearing in `create_default_scene()` 
- ❌ Don't skip adding debug logging to track entity state
- ❌ Don't assume UI state issues are separate from world access issues
- ❌ Don't modify inspector static state without understanding thread safety
- ❌ Don't test UI changes without running the actual editor interface

## Success Confidence Score: 8/10
High confidence due to:
- Comprehensive analysis of root cause (scene clearing race condition)
- Detailed debugging approach with logging
- Clear reproduction steps and validation criteria
- Well-understood patterns for both ECS and ImGui usage
- Specific code examples for each fix

Potential risk areas:
- Thread safety around shared world state
- Complex timing issues in editor initialization