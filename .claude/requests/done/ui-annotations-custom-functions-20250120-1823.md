# UI Annotations and Custom UI Functions

## Summary
Implement a system for setting UI behavior using annotations and defining custom UI functions that determine what the UI should look like. This should include automatic UI definitions for default types like Vec3, Quat, bool, and string if no custom UI annotation is defined.

## Context
The current component system uses manual UI creation in the inspector panel, with each component type having hardcoded UI logic. The system already has:
- A derive macro system with `Component` and `EditorUI` traits
- Manual UI rendering for each component type in the inspector
- ScriptProperties that handle different property types with custom UI
- A component registry system for metadata

## Research

### Current Implementation Analysis
1. **Component System** (`engine/src/component_system/mod.rs`):
   - Uses `Component` and `EditorUI` traits
   - Has UIBuilderFn type for UI builder functions
   - ComponentMetadata stores UI builder functions

2. **Derive Macros** (`engine_derive/src/lib.rs`):
   - `#[derive(Component)]` with `#[component(name = "...")]` attribute
   - `#[derive(EditorUI)]` currently generates empty implementation

3. **Inspector UI** (`editor/src/panels/inspector.rs`):
   - Manually implements UI for each component type
   - Already handles different types: Vec3 (3 drag inputs), Quat (euler angles), bool (checkbox), string (input text), colors (color picker)
   - ScriptProperties render different PropertyValue types with appropriate UI

4. **Property Types** in ScriptProperties:
   - Float: drag input with speed 0.01
   - Integer: drag input with speed 1.0
   - Boolean: checkbox
   - String: input text
   - Vector3: 3 separate drag inputs for X, Y, Z
   - Color: color edit with RGBA

### Industry Best Practices
- Bevy uses component-based architecture with derive macros
- egui integration typically uses helper functions for type conversions
- Attribute-based UI configuration is common in game engines (Unity's [Range], [Header], etc.)

## Requirements

### 1. UI Annotation System
- Extend the derive macro to support UI attributes
- Allow custom UI functions to be specified via attributes
- Support common UI hints like ranges, tooltips, headers, etc.

### 2. Custom UI Functions
- Allow components to define custom UI rendering logic
- Support both inline attributes and separate UI function implementations
- Provide access to imgui context and component data

### 3. Automatic UI Generation
- Generate default UI for common types when no custom annotation is provided:
  - `Vec3`: Three drag inputs for X, Y, Z
  - `Quat`: Euler angle editor with degree conversion
  - `bool`: Checkbox
  - `String`: Text input
  - `f32/f64`: Drag input with appropriate precision
  - `i32/i64`: Integer drag input
  - Arrays/Vecs: List editor with add/remove
  - Enums: Dropdown/combo box

### 4. Attribute Examples
```rust
#[derive(Component, EditorUI)]
struct MyComponent {
    #[ui(range = 0.0..1.0, tooltip = "Alpha value")]
    alpha: f32,
    
    #[ui(header = "Transform Settings")]
    position: Vec3,
    
    #[ui(custom = "color_picker_ui")]
    tint_color: [f32; 4],
    
    #[ui(multiline, height = 100)]
    description: String,
    
    #[ui(hidden)]
    internal_state: u32,
}
```

## Implementation Plan

### Phase 1: Extend Derive Macro System
1. Modify `engine_derive` to parse UI attributes
2. Generate EditorUI implementations based on attributes
3. Create attribute parsing utilities

### Phase 2: Default UI Generators
1. Create UI generator functions for each common type
2. Implement type detection in the macro
3. Generate appropriate UI calls based on field types

### Phase 3: Custom UI Functions
1. Add support for custom UI function references
2. Create a registry for custom UI functions
3. Allow both inline closures and function references

### Phase 4: Integration
1. Update existing components to use the new system
2. Remove manual UI code from inspector panel
3. Update documentation and examples

## Testing Requirements
- Unit tests for macro expansion
- Integration tests for each UI type
- Visual tests in the editor
- Performance benchmarks for UI generation

## Documentation Requirements
- Update CLAUDE.md with UI annotation guidelines
- Create examples for each annotation type
- Document custom UI function API
- Migration guide for existing components

## Performance Considerations
- Macro expansion should be compile-time only
- UI generation should be lazy where possible
- Consider caching UI layouts for complex components
- Avoid allocations in render loops

## Migration Strategy
1. Keep existing manual UI as fallback
2. Gradually migrate components to use annotations
3. Deprecate manual UI code once all components migrated
4. Provide tooling to help convert existing components

## Related Files
- `engine_derive/src/lib.rs` - Derive macro implementation
- `engine/src/component_system/mod.rs` - Component traits
- `editor/src/panels/inspector.rs` - Current UI implementation
- `engine/src/scripting/property_types.rs` - Property value types
- `engine/src/core/entity/components.rs` - Example components

## Success Criteria
- All built-in components use UI annotations
- Custom components can define UI with minimal boilerplate
- UI is automatically generated for common types
- Performance is equal or better than manual UI
- System is extensible for future UI needs