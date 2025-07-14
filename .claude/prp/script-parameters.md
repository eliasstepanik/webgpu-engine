name: "Unity-Style Script Parameters Implementation"
description: |

## Purpose
Implement a script parameter system that allows scripts to define configurable properties that are editable through the editor inspector, similar to Unity's SerializeField system.

## Core Principles
1. **Context is King**: Include ALL necessary documentation, examples, and caveats
2. **Validation Loops**: Provide executable tests/lints the AI can run and fix
3. **Information Dense**: Use keywords and patterns from the codebase
4. **Progressive Success**: Start simple, validate, then enhance
5. **Global rules**: Be sure to follow all rules in CLAUDE.md

---

## Goal
Add Unity-style script parameters that:
- Allow scripts to declare typed parameters with default values
- Display parameters in the editor inspector for per-entity configuration
- Serialize parameters with scene data
- Pass parameters to scripts at runtime through Rhai scope
- Maintain backward compatibility with existing scripts

## Why
- **Developer Experience**: Currently, scripts hardcode values like `rotation_speed = 1.0`, requiring script duplication for different behaviors
- **Reusability**: One script can serve multiple purposes with different parameter configurations
- **Designer-Friendly**: Non-programmers can tweak gameplay values without editing code
- **Consistency**: Aligns with how other components (Camera, Material) expose properties

## What
Users can define parameters in script comments:
```rhai
//! @property rotation_speed: float = 1.0
//! @property rotation_axis: vec3 = [0.0, 1.0, 0.0]
//! @property bob_enabled: bool = true

fn on_update(entity, delta_time) {
    // Access parameters through properties object
    let speed = properties["rotation_speed"];
    let axis = properties["rotation_axis"];
    // ...
}
```

### Success Criteria
- [ ] Scripts can declare typed parameters in comments
- [ ] Inspector shows script parameters with appropriate UI widgets
- [ ] Parameters are saved/loaded with scenes
- [ ] Scripts can access their parameters at runtime
- [ ] Existing scripts without parameters continue to work
- [ ] Validation of parameter values in editor

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.unity3d.com/ScriptReference/SerializeField.html
  why: Unity's approach to exposing private fields in inspector
  
- url: https://rhai.rs/book/engine/metadata/index.html
  why: Rhai's metadata capabilities (though we'll use comment parsing instead)
  
- url: https://fyrox.rs/blog/post/feature-highlights-0-27/
  why: Example of compile-time reflection for property editing in Rust game engine
  
- file: engine/src/scripting/components.rs
  why: Current ScriptRef implementation to understand and extend
  
- file: engine/src/scripting/script_system.rs
  why: Script execution system that needs to pass parameters
  
- file: editor/src/panels/inspector.rs
  why: Inspector implementation patterns for component editing
  
- file: engine/src/io/scene.rs
  why: Scene serialization system for saving/loading parameters
  
- file: engine/src/io/component_registry.rs
  why: Component registration pattern (unused but available)
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── scripting/
│   │   ├── mod.rs
│   │   ├── components.rs          # ScriptRef component
│   │   ├── script_system.rs       # Script execution
│   │   └── modules/               # Rhai modules
│   ├── io/
│   │   ├── scene.rs              # Scene serialization
│   │   └── component_registry.rs  # Component registry
│   └── lib.rs
editor/
├── src/
│   └── panels/
│       └── inspector.rs          # Component inspector UI
assets/
└── scripts/
    ├── rotating_cube.rhai        # Example script
    └── fly_camera.rhai          # Example script
```

### Desired Codebase Structure
```bash
engine/
├── src/
│   ├── scripting/
│   │   ├── mod.rs
│   │   ├── components.rs          # Extended with ScriptProperties
│   │   ├── script_system.rs       # Modified to pass properties
│   │   ├── property_parser.rs     # NEW: Parse property definitions
│   │   └── property_types.rs      # NEW: Property type definitions
editor/
├── src/
│   └── panels/
│       └── inspector.rs          # Extended with script property UI
assets/
└── scripts/
    ├── rotating_cube.rhai        # Updated with property definitions
    └── examples/
        └── parameterized_mover.rhai  # NEW: Example with properties
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: Rhai limitations
// - No built-in property/parameter system
// - Metadata feature is for introspection, not configuration
// - Must pass data through Scope or as function parameters

// CRITICAL: hecs ECS patterns
// - Components must be 'static + Send + Sync
// - Use World::remove_one() before modifying in editor
// - Always re-insert component after modification

// CRITICAL: Serialization requirements
// - All property types must implement Serialize/Deserialize
// - Use serde_json::Value for dynamic typing if needed
// - Component registry exists but isn't used by scene loading

// CRITICAL: Inspector UI patterns
// - Each component type needs manual UI code
// - Use ui.push_id()/pop_id() for unique widget IDs
// - Mark scene dirty when properties change
```

## Implementation Blueprint

### Data Models and Structure

```rust
// engine/src/scripting/property_types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyValue {
    Float(f32),
    Integer(i32),
    Boolean(bool),
    String(String),
    Vector3([f32; 3]),
    Color([f32; 4]),
}

#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    pub name: String,
    pub property_type: PropertyType,
    pub default_value: PropertyValue,
    pub metadata: PropertyMetadata,
}

#[derive(Debug, Clone)]
pub enum PropertyType {
    Float,
    Integer,
    Boolean,
    String,
    Vector3,
    Color,
}

#[derive(Debug, Clone)]
pub struct PropertyMetadata {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub tooltip: Option<String>,
}

// Component to store script properties per entity
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct ScriptProperties {
    pub values: HashMap<String, PropertyValue>,
}
```

### List of Tasks

```yaml
Task 1: Create Property Type System
CREATE engine/src/scripting/property_types.rs:
  - Define PropertyValue enum with all supported types
  - Define PropertyDefinition for script metadata
  - Create ScriptProperties component
  - Implement Display trait for debugging
  - Add conversion methods to/from Rhai Dynamic

Task 2: Implement Property Parser
CREATE engine/src/scripting/property_parser.rs:
  - Parse property definitions from script comments
  - Support format: //! @property name: type = default
  - Handle all supported types (float, int, bool, string, vec3, color)
  - Return Vec<PropertyDefinition>
  - Add comprehensive error handling

Task 3: Extend Script System
MODIFY engine/src/scripting/script_system.rs:
  - Load property definitions when caching scripts
  - Store definitions in ScriptCache
  - Pass properties to script scope before execution
  - Convert PropertyValue to Rhai Dynamic types
  - Handle missing ScriptProperties component gracefully

Task 4: Update ScriptRef Component  
MODIFY engine/src/scripting/components.rs:
  - Keep ScriptRef as-is for compatibility
  - Export new ScriptProperties component
  - Update module exports

Task 5: Add Inspector UI for Script Properties
MODIFY editor/src/panels/inspector.rs:
  - Add script property detection when ScriptRef exists
  - Fetch property definitions from script system
  - Render property widgets based on type:
    * Float: DragFloat with min/max/step
    * Integer: DragInt
    * Boolean: Checkbox
    * String: InputText
    * Vector3: 3x DragFloat
    * Color: ColorEdit4
  - Update ScriptProperties component on changes
  - Create component if missing but script has properties

Task 6: Update Scene Serialization
MODIFY engine/src/io/scene.rs:
  - Add ScriptProperties to component match arms
  - Ensure proper serialization/deserialization
  - No special entity mapping needed (no Entity references)

Task 7: Create Example Scripts
CREATE assets/scripts/examples/parameterized_mover.rhai:
  - Demonstrate all property types
  - Show how to access properties in script
UPDATE assets/scripts/rotating_cube.rhai:
  - Add property definitions for existing hardcoded values
  - Maintain backward compatibility

Task 8: Add Tests
CREATE engine/src/scripting/tests/property_tests.rs:
  - Test property parser with various inputs
  - Test type conversions
  - Test serialization round-trip
  - Test script execution with properties
```

### Per Task Pseudocode

```rust
// Task 2: Property Parser
pub fn parse_script_properties(script_content: &str) -> Result<Vec<PropertyDefinition>, ParseError> {
    let mut properties = Vec::new();
    
    for line in script_content.lines() {
        // PATTERN: Look for //! @property lines
        if let Some(prop_def) = line.strip_prefix("//! @property ") {
            // Parse: name: type = default_value
            // GOTCHA: Handle optional metadata like @range(0, 10)
            let definition = parse_property_line(prop_def)?;
            properties.push(definition);
        }
    }
    
    Ok(properties)
}

// Task 3: Script System Integration
impl ScriptSystem {
    fn execute_script_with_properties(&self, entity: Entity, script: &Script, properties: &ScriptProperties) {
        let mut scope = Scope::new();
        
        // CRITICAL: Convert properties to Rhai Dynamic
        let mut prop_map = rhai::Map::new();
        for (name, value) in &properties.values {
            prop_map.insert(name.clone(), convert_to_dynamic(value));
        }
        scope.push("properties", prop_map);
        
        // PATTERN: Pass entity and delta_time as before
        scope.push("entity", entity_to_i64(entity));
        // ... rest of execution
    }
}

// Task 5: Inspector UI
fn render_script_properties(ui: &imgui::Ui, properties: &mut ScriptProperties, definitions: &[PropertyDefinition]) {
    for def in definitions {
        // PATTERN: Get or create with default
        let value = properties.values.entry(def.name.clone())
            .or_insert_with(|| def.default_value.clone());
        
        // GOTCHA: Use push_id for unique widget IDs
        ui.push_id(&def.name);
        
        match value {
            PropertyValue::Float(f) => {
                // PATTERN: Use metadata for constraints
                let mut drag = Drag::new(format!("##{}", def.name))
                    .display_format("%.3f")
                    .speed(0.01);
                
                if let Some(min) = def.metadata.min {
                    drag = drag.range(min..=def.metadata.max.unwrap_or(f32::MAX));
                }
                
                if drag.build(ui, f) {
                    mark_dirty = true;
                }
            }
            // ... other types
        }
        
        ui.pop_id();
    }
}
```

### Integration Points
```yaml
SCRIPT_LOADING:
  - location: ScriptSystem::load_and_cache_script()
  - action: Parse properties after loading script content
  - store: Cache definitions alongside compiled AST

COMPONENT_REGISTRY:
  - add to: register_components() if using registry
  - component: ScriptProperties
  - note: Currently registry isn't used by scene loading

SCENE_FORMAT:
  - example: {"ScriptProperties": {"values": {"rotation_speed": {"Float": 1.0}}}}
  - location: Scene::instantiate() match arms
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cd engine && cargo fmt --all
cd engine && cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors. If errors, READ the error and fix.
```

### Level 2: Unit Tests
```rust
// Create tests in engine/src/scripting/tests/property_tests.rs
#[test]
fn test_parse_float_property() {
    let script = r#"
    //! @property speed: float = 1.0
    fn on_update(entity, dt) {}
    "#;
    
    let props = parse_script_properties(script).unwrap();
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].name, "speed");
    assert_eq!(props[0].default_value, PropertyValue::Float(1.0));
}

#[test]
fn test_property_serialization() {
    let mut props = ScriptProperties::default();
    props.values.insert("test".to_string(), PropertyValue::Float(42.0));
    
    let json = serde_json::to_string(&props).unwrap();
    let decoded: ScriptProperties = serde_json::from_str(&json).unwrap();
    assert_eq!(props.values, decoded.values);
}

#[test]
fn test_script_with_properties() {
    // Test that scripts can access properties through scope
    // Mock or use actual script system
}
```

```bash
# Run and iterate until passing:
cd engine && cargo test scripting::tests::property_tests
```

### Level 3: Integration Test
```bash
# Build everything
just preflight

# Run the game with a test scene containing scripted entities
cd game && cargo run

# Manually test in editor:
# 1. Add entity with ScriptRef component
# 2. Select script with properties
# 3. Verify properties appear in inspector
# 4. Modify values and verify they persist
# 5. Save scene and reload
# 6. Verify properties are restored
```

## Final Validation Checklist
- [ ] All tests pass: `just preflight`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Scripts with properties work correctly
- [ ] Scripts without properties still work
- [ ] Inspector shows property UI appropriately
- [ ] Properties serialize/deserialize correctly
- [ ] Property values passed to scripts at runtime
- [ ] Example scripts demonstrate all features

---

## Anti-Patterns to Avoid
- ❌ Don't modify existing ScriptRef structure (breaks compatibility)
- ❌ Don't hardcode property types in multiple places
- ❌ Don't skip validation of property values
- ❌ Don't parse properties on every frame (cache them)
- ❌ Don't forget ui.push_id() in inspector (widget ID conflicts)
- ❌ Don't assume scripts have properties (handle None case)
- ❌ Don't use reflection macros that complicate the build