# Modular Component System Implementation

## Overview
Create a derive macro system that automatically generates component registration, serialization, and editor UI code. This eliminates the need to manually update 4+ places when adding new components.

## Context and Current State

### Current Component Definition Pattern
Components currently require manual updates in multiple places. Example from `engine/src/core/entity/components.rs`:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

### Manual Inspector UI (editor/src/panels/inspector.rs:200-276)
Each component has hardcoded UI logic:
```rust
if has_mesh && ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
    let mut remove_component = false;
    shared_state.with_world_write(|world| {
        if let Ok(mut mesh_id) = world.inner_mut().remove_one::<MeshId>(entity) {
            // Manual UI code for each field...
            if ui.small_button("Remove Mesh") {
                remove_component = true;
            }
        }
    });
}
```

### Manual Serialization (engine/src/io/scene.rs:200-254)
Each component needs explicit serialization:
```rust
if let Ok(material) = world.get::<Material>(entity) {
    match serde_json::to_value(*material) {
        Ok(value) => {
            components.insert("Material".to_string(), value);
        }
        Err(e) => {
            error!(error = %e, "Failed to serialize Material");
        }
    }
}
```

### Existing Dynamic Property System (engine/src/scripting/property_types.rs)
The script system already demonstrates metadata-driven properties:
```rust
pub struct PropertyDefinition {
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: PropertyValue,
    pub metadata: PropertyMetadata,
}

pub struct PropertyMetadata {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub tooltip: Option<String>,
}
```

## Implementation Blueprint

### Phase 1: Create Derive Macro Crate

1. **Create new crate** `engine_derive/` with proc-macro support
2. **Implement derive macros**:

```rust
// Example usage after implementation:
#[derive(Component, EditorUI, Serialize, Deserialize)]
#[component(name = "Transform")]
pub struct Transform {
    #[editor(drag_speed = 0.1)]
    pub position: Vec3,
    
    #[editor(widget = "rotation")]
    pub rotation: Quat,
    
    #[editor(min = 0.0, max = 10.0, drag_speed = 0.01)]
    pub scale: Vec3,
}
```

The macro will generate:
- Registration with ComponentRegistry
- UI generation metadata
- Serialization helpers

### Phase 2: Enhance ComponentRegistry

Update `engine/src/io/component_registry.rs` to store UI metadata:

```rust
pub struct ComponentMetadata {
    pub name: &'static str,
    pub ui_builder: Box<dyn Fn(&mut World, Entity, &mut imgui::Ui) -> bool>,
    pub serializer: Box<dyn Fn(&dyn Any) -> Result<serde_json::Value>>,
    pub deserializer: ComponentDeserializerFn,
}

impl ComponentRegistry {
    pub fn register_with_metadata<T: Component + EditorUI>(&mut self) {
        // Auto-registration from derive macro
    }
}
```

### Phase 3: Update Inspector

Modify `editor/src/panels/inspector.rs` to use registry:

```rust
// Replace hardcoded component UI with:
for (type_id, metadata) in registry.iter_components() {
    if world.has_component(entity, type_id) {
        if ui.collapsing_header(&metadata.name, TreeNodeFlags::DEFAULT_OPEN) {
            let modified = (metadata.ui_builder)(world, entity, ui);
            if modified {
                shared_state.mark_scene_modified();
            }
        }
    }
}
```

### Phase 4: Update Scene Serialization

Modify `engine/src/io/scene.rs` to use registry:

```rust
// Replace manual serialization with:
for (type_id, metadata) in registry.iter_components() {
    if let Some(component) = world.get_component_raw(entity, type_id) {
        match (metadata.serializer)(component) {
            Ok(value) => {
                components.insert(metadata.name.to_string(), value);
            }
            Err(e) => {
                error!(error = %e, component = metadata.name, "Failed to serialize");
            }
        }
    }
}
```

## Implementation Tasks (In Order)

1. **Setup derive macro crate**
   - Create `engine_derive/Cargo.toml` with syn, quote, proc-macro2
   - Add to workspace members
   - Create lib.rs with proc_macro attribute

2. **Implement Component derive macro**
   - Parse struct and attributes
   - Generate registration code
   - Handle generic types properly

3. **Implement EditorUI derive macro**
   - Parse field attributes (min, max, drag_speed, widget type)
   - Generate UI builder function
   - Support all current widget types

4. **Create enhanced ComponentMetadata struct**
   - Add to component_registry.rs
   - Include UI builder, serializer, type info

5. **Update ComponentRegistry**
   - Add register_with_metadata method
   - Store metadata alongside deserializers
   - Add iteration methods for inspector

6. **Create component traits**
   - Component trait (marker)
   - EditorUI trait with ui_metadata method
   - Auto-implement via derive

7. **Update inspector.rs**
   - Replace hardcoded UI with registry queries
   - Maintain exact same UI behavior
   - Add "Add Component" dropdown from registry

8. **Update scene serialization**
   - Use registry for generic serialization
   - Maintain backwards compatibility
   - Update instantiate to use registry

9. **Migrate existing components**
   - Start with Transform (simplest)
   - Then Camera, Material, MeshId
   - Finally complex ones like ScriptProperties

10. **Add tests**
    - Macro expansion tests
    - Registry functionality tests
    - Serialization round-trip tests

## Validation Gates

### After Phase 1 (Macro Creation):
```bash
cd engine_derive && cargo check
cd .. && cargo check --workspace
```

### After Phase 2 (Registry Enhancement):
```bash
cargo test -p engine -- component_registry
```

### After Phase 3 (Inspector Update):
```bash
# Run editor and verify UI still works
cargo run -p editor
# Check component can be added/removed/edited
```

### After Phase 4 (Serialization):
```bash
# Test scene save/load
cargo test -p engine -- scene::tests
```

### Final Validation:
```bash
just preflight
```

## External References

- **Bevy InspectorOptions**: https://github.com/jakobhellermann/bevy-inspector-egui
- **Derive macro patterns**: https://docs.rs/syn/latest/syn/
- **Similar ECS patterns**: https://docs.rs/bevy_ecs/latest/bevy_ecs/

## Gotchas and Considerations

1. **hecs compatibility**: hecs doesn't require Component trait, our macro just needs to ensure 'static bound
2. **Generic types**: Handle Transform vs WorldTransform (f32 vs f64)
3. **Performance**: UI generation happens only in editor, no runtime cost
4. **Backwards compatibility**: Keep manual registration as fallback during migration
5. **Type erasure**: Use Any + TypeId for registry storage while maintaining type safety
6. **Drag and drop**: Preserve existing drag-drop functionality for assets

## Success Criteria

- All 13 existing components work with new system
- No performance regression (measure with `just preflight`)
- Inspector UI behavior identical to current
- Scene files remain compatible
- Adding new component requires only struct definition with derives

## Migration Example

Before:
```rust
// 1. Define in components.rs
pub struct MyComponent { value: f32 }

// 2. Add to inspector.rs (50+ lines)
// 3. Add to scene.rs serialization (10 lines)
// 4. Add to scene.rs deserialization (10 lines)
// 5. Register in ComponentRegistry
```

After:
```rust
#[derive(Component, EditorUI, Serialize, Deserialize)]
pub struct MyComponent {
    #[editor(min = 0.0, max = 100.0)]
    value: f32
}
// Done! Everything else is automatic
```

## Confidence Score: 8/10

High confidence due to:
- Clear patterns in existing code
- Similar successful implementations (Bevy)
- Well-defined scope and validation gates
- Existing PropertyMetadata pattern to follow

Risk mitigation:
- Incremental migration approach
- Each phase independently testable
- Backwards compatibility maintained