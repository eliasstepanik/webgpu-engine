## FEATURE:

Create a modular system for easily defining new component types with automatic editor UI generation and function registration

## EXAMPLES:

.claude/examples/console-output-before.txt – shows console output patterns (not directly related to components)
.claude/examples/console-output-after.txt – shows console output patterns (not directly related to components)

## DOCUMENTATION:

https://docs.rs/bevy_ecs/latest/bevy_ecs/ – Bevy ECS design patterns for component definitions
https://github.com/jakobhellermann/bevy-inspector-egui – Bevy's inspector UI derive macros for automatic editor integration
https://lib.rs/crates/bevy-inspector-egui-derive – Derive macro implementation for inspector options
https://taintedcoders.com/bevy/components – Bevy component system overview
https://rodneylab.com/rust-entity-component-systems/ – Comparison of Rust ECS libraries

## OTHER CONSIDERATIONS:

- Current system requires manual code in 4+ places to add new components (components.rs, scene.rs serialization/deserialization, inspector.rs UI)
- Existing ComponentRegistry in engine/src/io/component_registry.rs is underutilized - only used for type registration, not actual serialization
- Inspector UI code is hardcoded for each component type in editor/src/panels/inspector.rs
- Script system already has dynamic property support via ScriptProperties - could serve as inspiration
- Consider derive macros like #[derive(Component, EditorUI)] similar to Bevy's #[derive(Component, InspectorOptions)]
- Need to maintain backwards compatibility with existing component definitions
- Performance considerations: avoid runtime overhead in hot paths (render loops)
- Type safety: preserve Rust's compile-time guarantees while adding flexibility
- Serialization: must integrate with existing scene save/load system using serde_json

## EXISTING COMPONENTS TO MIGRATE:

Found 13 component types that need migration to the new system:

### Core Entity Components (engine/src/core/entity/components.rs)
- Transform - f32 precision position/rotation/scale
- WorldTransform - f64 precision for large worlds
- GlobalTransform - World-space matrix (f32)
- GlobalWorldTransform - World-space matrix (f64)
- Parent/ParentData - Parent-child relationships
- Name - String identifier

### Camera Components (engine/src/core/camera.rs)
- Camera - Projection parameters (FOV, aspect, near/far)
- CameraWorldPosition - High-precision camera position

### Graphics Components
- Material (engine/src/graphics/material.rs) - Surface properties
- MeshId (engine/src/graphics/renderer.rs) - Mesh association

### Scripting Components
- ScriptRef (engine/src/scripting/components.rs) - Script asset reference
- ScriptProperties (engine/src/scripting/property_types.rs) - Dynamic script properties

## IMPLEMENTATION APPROACH:

1. Create derive macro crate (engine_derive) with:
   - #[derive(Component)] - Auto-registers with ComponentRegistry
   - #[component(editor = "...")] attributes for UI hints
   - Support for property constraints (min/max, ranges, etc.)

2. Enhance ComponentRegistry to:
   - Store UI metadata alongside deserializers
   - Generate editor UI automatically from metadata
   - Handle serialization/deserialization generically

3. Update inspector.rs to:
   - Query ComponentRegistry for available components
   - Use UI metadata to generate appropriate controls
   - Remove hardcoded component UI code

4. Migration strategy:
   - Add derive macros to existing components
   - Gradually remove hardcoded serialization/UI code
   - Maintain compatibility during transition