name: "Scene Serialization System - WebGPU Template"
description: |

## Purpose
Implement a robust scene serialization system enabling persistence of entity hierarchies, transforms, and components for level editing and scene management in the WebGPU template engine.

## Core Principles
1. **Entity ID Remapping**: Handle Parent component serialization through ID mapping
2. **Extensible Design**: Support adding new components without breaking existing scenes
3. **Human-Readable Format**: JSON format for easy editing and debugging
4. **Integration First**: Seamlessly integrate with existing ECS and future io module
5. **Global rules**: Follow all rules in CLAUDE.md

---

## Goal
Build a scene loading and saving system that allows developers to:
- Save entire worlds or entity subsets to JSON files
- Load scenes with proper entity relationship restoration
- Support additive loading for composing complex scenes
- Enable level editing workflows and scene templates

## Why
- **Persistence**: Save game state and player-created content
- **Level Design**: Create and edit levels outside of code
- **Rapid Iteration**: Modify scenes without recompilation
- **Reusability**: Create prefab/template systems for common entity groups

## What
Implement scene serialization that handles:
- Component serialization using serde
- Entity ID remapping for Parent components
- Graceful handling of missing component types
- Scene asset management in the io module
- Round-trip serialization guarantees

### Success Criteria
- [ ] Scenes can be saved to and loaded from JSON files
- [ ] Parent-child relationships are preserved through ID remapping
- [ ] Missing component types are logged but don't crash loading
- [ ] All existing serializable components work out of the box
- [ ] Unit tests verify round-trip serialization
- [ ] Integration with io module for file management

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://serde.rs/json.html
  why: Core JSON serialization patterns, especially tagged enums for polymorphic types
  
- url: https://github.com/Ralith/hecs/blob/master/examples/serialize.rs
  why: hecs serialization example showing ColumnBatchType and entity handling
  critical: Shows how to serialize/deserialize entities with dynamic components
  
- url: https://docs.rs/bevy/latest/bevy/scene/
  why: Reference implementation of entity remapping in production ECS
  section: DynamicScene and entity mapping approaches
  
- file: engine/src/core/entity/components.rs
  why: Existing component definitions with serde derives, Parent component limitation
  
- file: engine/src/core/entity/world.rs
  why: World wrapper API and entity spawning patterns
  
- file: engine/src/core/entity/hierarchy.rs
  why: Parent-child relationship handling and cycle detection

- doc: https://serde.rs/enum-representations.html
  section: Internally tagged and adjacently tagged representations
  critical: Use internally tagged JSON for component type discrimination
```

### Current Codebase tree
```bash
webgpu-template/
├── engine/
│   ├── src/
│   │   ├── core/
│   │   │   ├── entity/
│   │   │   │   ├── components.rs  # Transform, Parent, GlobalTransform
│   │   │   │   ├── hierarchy.rs   # Parent-child traversal
│   │   │   │   ├── world.rs       # World wrapper
│   │   │   │   └── mod.rs
│   │   │   ├── camera.rs          # Camera component
│   │   │   └── mod.rs
│   │   ├── graphics/               # Material, mesh components
│   │   ├── input/
│   │   ├── shaders/
│   │   └── lib.rs                 # Public exports
│   └── Cargo.toml
├── game/
├── PLANNING.md                     # Scene JSON format example
└── CLAUDE.md                      # Development guidelines
```

### Desired Codebase tree with files to be added
```bash
engine/
├── src/
│   ├── io/                        # NEW MODULE
│   │   ├── mod.rs                 # Module exports
│   │   ├── scene.rs               # Scene struct and serialization
│   │   ├── entity_mapper.rs       # Entity ID remapping logic
│   │   └── component_registry.rs  # Dynamic component deserialization
│   ├── core/
│   │   └── entity/
│   │       ├── components.rs      # Add EntityId wrapper for Parent
│   └── lib.rs                     # Export io module
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: hecs::Entity cannot be serialized directly
// It's an opaque handle only valid within its World instance
// Must serialize as u64 ID and remap on load

// CRITICAL: serde_json requires explicit type tags for polymorphic deserialization
// Use #[serde(tag = "type")] for component discrimination

// CRITICAL: Component registration must happen before deserialization
// Use TypeId or string names for component type mapping

// GOTCHA: Entity IDs must be contiguous for efficient remapping
// Use HashMap<OldId, NewEntity> for sparse ID spaces

// PATTERN: All components must be 'static (already enforced by hecs)
// PATTERN: Use tracing crate for logging, NO println!
```

## Implementation Blueprint

### Data models and structure

```rust
// engine/src/io/scene.rs
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub entities: Vec<SerializedEntity>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedEntity {
    // Store components as JSON value map
    pub components: HashMap<String, serde_json::Value>,
}

// engine/src/core/entity/components.rs
// Add this for Parent serialization
#[derive(Serialize, Deserialize)]
pub struct ParentData {
    pub entity_id: u64,  // Will be remapped on load
}

// engine/src/io/entity_mapper.rs
pub struct EntityMapper {
    // Maps old entity IDs to new entities
    mapping: HashMap<u64, hecs::Entity>,
}
```

### List of tasks to be completed in order

```yaml
Task 1: Create io module structure
CREATE engine/src/io/mod.rs:
  - Export public types: Scene, SceneError
  - Module declarations for scene, entity_mapper, component_registry
  
MODIFY engine/src/lib.rs:
  - Add: pub mod io;
  - Update prelude with Scene type

Task 2: Implement Scene serialization structures
CREATE engine/src/io/scene.rs:
  - Scene struct with entities: Vec<SerializedEntity>
  - SerializedEntity with components: HashMap<String, serde_json::Value>
  - from_world() method to capture entity data
  - save_to_file() and load_from_file() methods
  - Use tracing for error logging

Task 3: Create entity ID mapping system
CREATE engine/src/io/entity_mapper.rs:
  - EntityMapper struct with HashMap<u64, hecs::Entity>
  - new() constructor
  - register(old_id, new_entity) method
  - remap(old_id) -> Option<hecs::Entity> method
  - Handle missing IDs gracefully

Task 4: Implement component registry
CREATE engine/src/io/component_registry.rs:
  - ComponentRegistry with deserializers map
  - register_component<T>() for each component type
  - deserialize_component() returning Box<dyn Any>
  - Default registry with all engine components

Task 5: Add Parent component serialization support
MODIFY engine/src/core/entity/components.rs:
  - Add ParentData struct with entity_id: u64
  - Implement From<Parent> for ParentData
  - Implement TryFrom<ParentData> for Parent (requires EntityMapper)
  - Document the serialization approach

Task 6: Implement Scene::from_world
MODIFY engine/src/io/scene.rs:
  - Iterate world entities
  - For each component type, serialize to JSON
  - Special handling for Parent -> ParentData conversion
  - Assign monotonic IDs to entities for serialization

Task 7: Implement Scene::instantiate
MODIFY engine/src/io/scene.rs:
  - Create EntityMapper for remapping
  - First pass: spawn all entities, build ID map
  - Second pass: add components with remapped Parent IDs
  - Return EntityMapper for caller reference

Task 8: Add convenience methods to World
MODIFY engine/src/core/entity/world.rs:
  - save_scene(path) -> Result<()>
  - load_scene(path) -> Result<()>
  - load_scene_additive(path) -> Result<EntityMapper>

Task 9: Write comprehensive tests
CREATE engine/src/io/mod.rs (tests module):
  - test_scene_round_trip: Save and load scene
  - test_parent_remapping: Verify Parent IDs are correct
  - test_missing_component: Unknown component type handling
  - test_circular_parents: Cycle detection after load
  - test_empty_scene: Edge case handling
  - test_additive_loading: Multiple scene composition
```

### Per task pseudocode

```rust
// Task 2 - Scene::from_world pseudocode
pub fn from_world(world: &World) -> Self {
    let mut entities = Vec::new();
    let mut entity_to_id = HashMap::new();
    let mut next_id = 0u64;
    
    // First pass: assign IDs
    for (entity, _) in world.query::<()>().iter() {
        entity_to_id.insert(entity, next_id);
        next_id += 1;
    }
    
    // Second pass: serialize components
    for (entity, ()) in world.query::<()>().iter() {
        let mut components = HashMap::new();
        
        // Check each component type (hardcoded for now)
        if let Ok(transform) = world.get::<Transform>(entity) {
            components.insert("Transform".to_string(), 
                            serde_json::to_value(&*transform).unwrap());
        }
        
        if let Ok(parent) = world.get::<Parent>(entity) {
            // Special handling for Parent
            let parent_id = entity_to_id[&parent.0];
            let parent_data = ParentData { entity_id: parent_id };
            components.insert("Parent".to_string(),
                            serde_json::to_value(&parent_data).unwrap());
        }
        
        // ... other components
        
        entities.push(SerializedEntity { components });
    }
    
    Scene { entities }
}

// Task 7 - Scene::instantiate pseudocode
pub fn instantiate(&self, world: &mut World) -> Result<EntityMapper> {
    let mut mapper = EntityMapper::new();
    let mut entities_to_build = Vec::new();
    
    // First pass: spawn entities and build mapping
    for (id, serialized) in self.entities.iter().enumerate() {
        let entity = world.spawn(());
        mapper.register(id as u64, entity);
        entities_to_build.push((entity, serialized));
    }
    
    // Second pass: add components with remapping
    for (entity, serialized) in entities_to_build {
        for (component_type, value) in &serialized.components {
            match component_type.as_str() {
                "Transform" => {
                    let transform: Transform = serde_json::from_value(value.clone())?;
                    world.insert_one(entity, transform)?;
                }
                "Parent" => {
                    let parent_data: ParentData = serde_json::from_value(value.clone())?;
                    if let Some(parent_entity) = mapper.remap(parent_data.entity_id) {
                        world.insert_one(entity, Parent(parent_entity))?;
                    } else {
                        warn!(parent_id = parent_data.entity_id, 
                              "Parent entity not found in scene");
                    }
                }
                // ... other components
                unknown => {
                    warn!(component_type = unknown, "Unknown component type in scene");
                }
            }
        }
    }
    
    Ok(mapper)
}
```

### Integration Points
```yaml
WORLD:
  - Add convenience methods for scene operations
  - Ensure cycle detection runs after scene load
  
COMPONENTS:
  - All new components must derive Serialize, Deserialize
  - Parent needs special serialization logic via ParentData
  
ERROR HANDLING:
  - Create SceneError enum for load/save failures
  - Use thiserror for error derivation
  - Log warnings for non-fatal issues (missing components)

FILE PATHS:
  - Scene files go in assets/scenes/
  - Use .json extension for human readability
  - Support relative paths from project root
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all                     # Format code
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// Test scene round-trip
#[test]
fn test_scene_round_trip() {
    let mut world = World::new();
    
    // Create test hierarchy
    let parent = world.spawn((
        Transform::from_position(Vec3::new(1.0, 2.0, 3.0)),
        GlobalTransform::default(),
    ));
    
    let child = world.spawn((
        Transform::from_position(Vec3::X),
        GlobalTransform::default(),
        Parent(parent),
    ));
    
    // Save to scene
    let scene = Scene::from_world(&world);
    
    // Create new world and load
    let mut new_world = World::new();
    let mapper = scene.instantiate(&mut new_world).unwrap();
    
    // Verify structure preserved
    assert_eq!(new_world.query::<()>().iter().count(), 2);
    
    // Check parent relationship remapped correctly
    let remapped_child = mapper.remap(1).unwrap(); // child was second entity
    let child_parent = new_world.get::<Parent>(remapped_child).unwrap();
    let remapped_parent = mapper.remap(0).unwrap();
    assert_eq!(child_parent.0, remapped_parent);
}

// Test missing component handling
#[test] 
fn test_missing_component() {
    let json = r#"{
        "entities": [{
            "components": {
                "Transform": {"position":[0,0,0],"rotation":[0,0,0,1],"scale":[1,1,1]},
                "UnknownComponent": {"data": "ignored"}
            }
        }]
    }"#;
    
    let scene: Scene = serde_json::from_str(json).unwrap();
    let mut world = World::new();
    
    // Should not panic, just warn
    let result = scene.instantiate(&mut world);
    assert!(result.is_ok());
    assert_eq!(world.query::<Transform>().iter().count(), 1);
}
```

```bash
# Run tests iteratively until passing:
cargo test --package engine --lib io::tests -v
# Fix issues, re-run until green
```

### Level 3: Integration Test
```bash
# Build the project
just preflight

# Create test scene file
cat > assets/scenes/test.json << 'EOF'
{
  "entities": [
    {
      "components": {
        "Transform": {"position":[0,0,5],"rotation":[0,0,0,1],"scale":[1,1,1]},
        "Camera": {"projection":"Perspective","fov":60.0,"near":0.1,"far":500.0}
      }
    }
  ]
}
EOF

# Verify in game code
# Add to game/src/main.rs in setup:
# let scene = Scene::load_from_file("assets/scenes/test.json")?;
# scene.instantiate(&mut world)?;

# Run and verify camera entity created
just run
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Formatting clean: `cargo fmt --all -- --check`
- [ ] Scene round-trip test passes
- [ ] Parent remapping works correctly
- [ ] Unknown components handled gracefully
- [ ] Circular parent detection still works
- [ ] Documentation complete: `cargo doc --workspace --no-deps`
- [ ] Integration test loads scene successfully

---

## Anti-Patterns to Avoid
- ❌ Don't serialize hecs::Entity directly - use u64 IDs
- ❌ Don't panic on unknown components - log and continue
- ❌ Don't use println! - use tracing macros
- ❌ Don't hardcode component types - use registry pattern
- ❌ Don't skip entity remapping - breaks Parent relationships
- ❌ Don't modify component derives without updating serialization

## Confidence Score: 8/10

The implementation path is clear with good reference documentation. The main complexity is in entity ID remapping and dynamic component deserialization, but the hecs example and Bevy's approach provide solid patterns to follow. Two points deducted for: 
1. Dynamic component registration will require careful type handling
2. The component registry pattern needs iterative refinement based on actual component types