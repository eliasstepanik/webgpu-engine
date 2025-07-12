name: "Extended Editor Panels - Name Component, Editable Inspector, and Selection Highlighting"
description: |

## Purpose
Implement comprehensive editor panel enhancements including entity naming, editable component inspector with dropdown for adding components, and visual selection highlighting in the viewport.

## Core Principles
1. **Thread Safety**: All world mutations through shared_state methods
2. **Type Safety**: Handle Rust's static typing for dynamic component operations
3. **Progressive Implementation**: Start with Name component, then editing, then rendering
4. **Follow CLAUDE.md**: Adhere to all project guidelines
5. **Test Everything**: Unit tests for new functionality

---

## Goal
Transform the read-only editor into a fully functional scene editor where users can:
- Name entities for better organization
- Edit component values in real-time
- Add/remove components dynamically
- See selected entities highlighted in the viewport

## Why
- **Usability**: Current debug IDs are not user-friendly
- **Productivity**: Direct component editing speeds up iteration
- **Visual Feedback**: Selection highlighting improves spatial awareness
- **Professional Tool**: Brings editor up to industry standards

## What
- Add Name component to engine
- Display entity names in hierarchy (fallback to ID if no name)
- Convert inspector from read-only to editable
- Add dropdown to dynamically add components
- Implement selection outline rendering in viewport

### Success Criteria
- [ ] Entities can be named and names persist in saved scenes
- [ ] All component values are editable in inspector
- [ ] Component dropdown shows only unattached components
- [ ] Selected entities have visible outline in viewport
- [ ] All changes mark scene as modified
- [ ] `just preflight` passes

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/imgui/latest/imgui/struct.ComboBox.html
  why: ComboBox API for component dropdown implementation
  
- url: https://ameye.dev/notes/edge-detection-outlines/
  why: Edge detection techniques for selection outline
  
- file: engine/src/core/entity/components.rs
  why: Pattern for defining new components
  
- file: engine/src/io/component_registry.rs
  why: How to register new component types
  
- file: editor/src/panels/inspector.rs
  why: Current inspector implementation to modify
  
- file: editor/src/shared_state.rs
  why: Thread-safe world access patterns
  
- file: engine/src/graphics/renderer.rs
  why: Current rendering pipeline for multi-pass implementation
  
- file: engine/src/shaders/basic.wgsl
  why: Shader structure for outline shader
  
- file: CLAUDE.md
  why: Project guidelines and mandatory rules
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── core/
│   │   └── entity/
│   │       ├── components.rs    # Component definitions
│   │       ├── mod.rs          # Module exports
│   │       └── world.rs        # ECS wrapper
│   ├── graphics/
│   │   ├── renderer.rs         # Main renderer
│   │   ├── pipeline.rs         # Pipeline creation
│   │   └── render_pass.rs      # Render pass management
│   ├── io/
│   │   ├── component_registry.rs # Component registration
│   │   └── scene.rs            # Scene serialization
│   └── shaders/
│       └── basic.wgsl          # Basic 3D shader
editor/
├── src/
│   ├── panels/
│   │   ├── inspector.rs        # Inspector panel
│   │   └── hierarchy.rs        # Hierarchy panel
│   └── shared_state.rs         # Thread-safe state
```

### Desired Codebase Structure
```bash
engine/
├── src/
│   ├── core/
│   │   └── entity/
│   │       ├── components.rs    # + Name component
│   ├── graphics/
│   │   ├── outline_renderer.rs  # NEW: Outline rendering system
│   └── shaders/
│       ├── basic.wgsl          
│       └── outline.wgsl        # NEW: Outline shader
```

### Known Gotchas & Critical Patterns
```rust
// CRITICAL: World access must use shared_state wrappers
// NEVER: world.insert_one() directly
// ALWAYS: shared_state.with_world_write(|world| { world.insert_one() })

// CRITICAL: Component registry uses type-erased function pointers
// Components don't implement traits, just need 'static + Serialize + Deserialize

// CRITICAL: ImGui immediate mode - state doesn't persist between frames
// Must track dropdown state in EditorState or panel

// CRITICAL: Mark scene modified after ANY change
// shared_state.mark_scene_modified()

// CRITICAL: Quaternion rotation editing needs euler conversion
// Users expect degrees, not quaternions
```

## Implementation Blueprint

### Data Models and Structure

```rust
// Name component - engine/src/core/entity/components.rs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Name(pub String);

// Inspector state - editor/src/panels/inspector.rs
struct InspectorState {
    // Track which component type to add
    selected_component_type: Option<String>,
    // Temporary euler angles for rotation editing
    euler_cache: HashMap<Entity, Vec3>,
}
```

### List of Tasks

```yaml
Task 1: Add Name Component
MODIFY engine/src/core/entity/components.rs:
  - ADD Name struct after Material
  - DERIVE Debug, Clone, Default, Serialize, Deserialize
  
MODIFY engine/src/core/entity/mod.rs:
  - EXPORT Name in pub use statement
  
MODIFY engine/src/prelude.rs:
  - ADD Name to prelude exports

MODIFY engine/src/io/component_registry.rs:
  - REGISTER Name component in new() method
  - PATTERN: Follow Material registration

Task 2: Update Hierarchy to Display Names
MODIFY editor/src/panels/hierarchy.rs:
  - UPDATE get_entity_name() function
  - CHECK for Name component first, fallback to ID
  - PATTERN: Use world.get::<&Name>(entity)

Task 3: Convert Inspector to Editable
MODIFY editor/src/panels/inspector.rs:
  - CHANGE with_world_read to with_world_write
  - REPLACE text displays with input widgets
  - ADD state struct for euler angles
  - IMPLEMENT component modification
  - MARK scene modified after changes

Task 4: Add Component Dropdown
MODIFY editor/src/panels/inspector.rs:
  - CREATE combo box for component selection
  - FILTER already attached components
  - IMPLEMENT type-erased component addition
  - USE match statement on component names

Task 5: Implement Selection Highlighting
CREATE engine/src/graphics/outline_renderer.rs:
  - DESIGN two-pass rendering approach
  - PASS 1: Render selected entity to stencil
  - PASS 2: Render outline using edge detection

CREATE engine/src/shaders/outline.wgsl:
  - VERTEX: Expand vertices along normals
  - FRAGMENT: Solid color output

MODIFY engine/src/graphics/renderer.rs:
  - ADD outline rendering pass
  - INTEGRATE with main render loop
  - PASS selected entity from editor state

Task 6: Add Tests
CREATE tests for:
  - Name component serialization
  - Inspector value editing
  - Component addition/removal
  - Outline renderer (if testable)
```

### Per Task Implementation Details

```rust
// Task 1: Name Component
// In engine/src/core/entity/components.rs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

// Task 2: Hierarchy Name Display
// In get_entity_name() function
fn get_entity_name(world: &World, entity: Entity) -> String {
    // Try Name component first
    if let Ok(name) = world.get::<&Name>(entity) {
        if !name.0.is_empty() {
            return name.0.clone();
        }
    }
    // Fallback to ID with component indicator
    if world.get::<&Transform>(entity).is_ok() {
        format!("Entity {:?}", entity)
    } else {
        format!("Entity {:?} [No Transform]", entity)
    }
}

// Task 3: Editable Transform
// Pattern for mutable component access
shared_state.with_world_write(|world| {
    if let Ok(mut transform) = world.get::<&mut Transform>(entity) {
        // Convert quaternion to euler for display
        let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
        let mut euler = [x.to_degrees(), y.to_degrees(), z.to_degrees()];
        
        // Edit position
        let mut pos = [transform.position.x, transform.position.y, transform.position.z];
        if ui.drag_float3("Position", &mut pos) {
            transform.position = Vec3::new(pos[0], pos[1], pos[2]);
            shared_state.mark_scene_modified();
        }
        
        // Edit rotation (euler)
        if ui.drag_float3("Rotation", &mut euler) {
            transform.rotation = Quat::from_euler(
                EulerRot::XYZ,
                euler[0].to_radians(),
                euler[1].to_radians(),
                euler[2].to_radians()
            );
            shared_state.mark_scene_modified();
        }
    }
});

// Task 4: Component Dropdown
let available_components = ComponentRegistry::registered_types();
let mut selected = None;

// Filter out already attached components
let filtered: Vec<&str> = available_components
    .iter()
    .filter(|comp_type| {
        match *comp_type {
            "Transform" => world.get::<&Transform>(entity).is_err(),
            "Camera" => world.get::<&Camera>(entity).is_err(),
            "Material" => world.get::<&Material>(entity).is_err(),
            "MeshId" => world.get::<&MeshId>(entity).is_err(),
            "Name" => world.get::<&Name>(entity).is_err(),
            _ => true
        }
    })
    .map(|s| s.as_str())
    .collect();

// Render combo box
if let Some(_token) = ui.begin_combo("##add_component", "Add Component...") {
    for comp_type in &filtered {
        if ui.selectable(comp_type) {
            selected = Some(comp_type.to_string());
        }
    }
}

// Add selected component
if let Some(comp_type) = selected {
    match comp_type.as_str() {
        "Transform" => world.insert_one(entity, Transform::default()),
        "Camera" => world.insert_one(entity, Camera::default()),
        "Material" => world.insert_one(entity, Material::default()),
        "MeshId" => world.insert_one(entity, MeshId::default()),
        "Name" => world.insert_one(entity, Name::default()),
        _ => Ok(())
    };
    shared_state.mark_scene_modified();
}
```

### Integration Points
```yaml
COMPONENT_REGISTRY:
  - location: engine/src/io/component_registry.rs
  - add: register_component::<Name>("Name", deserialize_name)
  
SCENE_SERIALIZATION:
  - automatic: Name component will serialize/deserialize with scenes
  
RENDER_PIPELINE:
  - location: engine/src/graphics/renderer.rs
  - add: outline_pass after main render pass
  - input: selected_entity from EditorSharedState
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run after EVERY file modification
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
# If errors: READ and fix based on clippy suggestions
```

### Level 2: Unit Tests
```rust
// Test Name component - engine/src/core/entity/components.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_name_component() {
        let name = Name::new("Test Entity");
        assert_eq!(name.0, "Test Entity");
        
        // Test serialization
        let json = serde_json::to_string(&name).unwrap();
        let deserialized: Name = serde_json::from_str(&json).unwrap();
        assert_eq!(name.0, deserialized.0);
    }
}

// Test component addition - in integration tests
#[test]
fn test_add_component_to_entity() {
    let mut world = World::new();
    let entity = world.spawn((Transform::default(),));
    
    // Add Name component
    world.insert_one(entity, Name::new("Test")).unwrap();
    
    // Verify it exists
    assert!(world.get::<&Name>(entity).is_ok());
}
```

```bash
# Run tests
cargo test --workspace

# If failing: Debug with
cargo test --workspace -- --nocapture
```

### Level 3: Integration Test
```bash
# Build and run editor
just run-editor

# Manual testing checklist:
# 1. Create entity (verify it appears in hierarchy)
# 2. Add Name component via dropdown
# 3. Edit name in inspector
# 4. Save scene and reload (verify name persists)
# 5. Edit Transform values (verify they update)
# 6. Select entity (verify outline appears)
```

### Level 4: Full Preflight
```bash
# Final validation before completion
just preflight

# Expected: All green
# This runs: fmt, clippy, test, doc
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Formatting correct: `cargo fmt --all -- --check`
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] Name component saves/loads with scenes
- [ ] All existing components are editable
- [ ] Component dropdown filters correctly
- [ ] Selection outline renders without artifacts
- [ ] Scene modified flag set on all edits
- [ ] No performance regression in editor

---

## Anti-Patterns to Avoid
- ❌ Don't access world directly - use shared_state
- ❌ Don't forget to mark scene modified after edits
- ❌ Don't hardcode component types - use registry
- ❌ Don't skip euler/quaternion conversion for rotation
- ❌ Don't create new patterns - follow existing code
- ❌ Don't add components without checking if they exist
- ❌ Don't modify renderer without considering performance

## Common Errors and Solutions

### "Cannot borrow world as mutable"
**Solution**: Use `with_world_write` not `with_world_read`

### "Component not found in registry"
**Solution**: Add to `ComponentRegistry::new()` in component_registry.rs

### "Outline renders on top of everything"
**Solution**: Check depth testing in outline pipeline configuration

### "Changes don't persist"
**Solution**: Call `shared_state.mark_scene_modified()` after edits

---

**Confidence Score: 8/10**

This PRP provides comprehensive context for implementing the extended editor panels. The score is 8 because:
- Clear implementation path with concrete code examples
- All file locations and patterns documented
- Validation steps are executable
- Common pitfalls identified with solutions

Points deducted for:
- Outline rendering might need iteration for best visual quality
- Component dropdown state management might need refinement