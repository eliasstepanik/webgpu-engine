name: "Transform Hierarchy System Implementation"
description: |

## Purpose
Implement a complete transform hierarchy system for the WebGPU engine, enabling parent-child relationships between entities with automatic world transform calculation.

## Core Principles
1. **Context is King**: Include ALL necessary documentation, examples, and caveats
2. **Validation Loops**: Provide executable tests/lints the AI can run and fix
3. **Information Dense**: Use keywords and patterns from the codebase
4. **Progressive Success**: Start simple, validate, then enhance
5. **Global rules**: Be sure to follow all rules in CLAUDE.md

---

## Goal
Implement Transform, GlobalTransform, and Parent components with a hierarchy system that traverses parent relationships breadth-first and calculates world matrices. Include helper APIs for entity creation that auto-insert required components.

## Why
- **Core functionality**: Transform hierarchies are fundamental for 3D scenes (objects, cameras, lights)
- **Scene graphs**: Enables parent-child relationships for intuitive scene organization
- **Animation ready**: Foundation for skeletal animation and complex transformations
- **Editor support**: Required for scene editor to manipulate object hierarchies

## What
User-visible behavior:
- Entities can have parent-child relationships
- Child transforms are relative to parent transforms
- World space positions update automatically each frame
- Helper functions simplify entity creation with required components
- Cyclic parenting is detected and logged as errors

### Success Criteria
- [ ] Transform component stores local position, rotation, scale
- [ ] GlobalTransform component stores world matrix (Mat4)
- [ ] Parent component links to parent entity
- [ ] Hierarchy system updates GlobalTransform breadth-first
- [ ] Helper APIs auto-add required components
- [ ] Cyclic parenting detected and handled gracefully
- [ ] Components serialize/deserialize correctly
- [ ] All tests pass and documentation builds

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/hecs/latest/hecs/
  why: Core ECS functionality - World, Entity, queries, component insertion
  
- url: https://docs.rs/glam/latest/glam/
  why: Math types - Mat4, Vec3, Quat for transform calculations
  section: Look for Mat4::from_scale_rotation_translation
  
- url: https://serde.rs/derive.html
  why: Serialize/Deserialize derives for scene saving/loading
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/CLAUDE.md
  why: Project conventions - module structure, imports, logging requirements
  critical: Never create mod.rs with same name as parent dir
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/PLANNING.md
  why: Architecture decisions - component list, scene JSON format

- docfile: https://docs.rs/hecs/latest/hecs/struct.World.html
  why: World API for entity creation and queries

- pattern: BFS traversal for cache efficiency
  why: Breadth-first access pattern is cache-friendly for transform updates
```

### Current Codebase tree
```bash
webgpu-template/
├── engine/
│   ├── Cargo.toml (missing hecs dependency)
│   └── src/
│       ├── core/
│       │   └── mod.rs (empty, needs entity module)
│       ├── graphics/
│       │   └── mod.rs
│       ├── input/
│       │   └── mod.rs
│       ├── shaders/
│       │   └── mod.rs
│       └── lib.rs
├── game/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs (placeholder)
├── justfile (has preflight command)
├── CLAUDE.md (coding standards)
└── PLANNING.md (architecture)
```

### Desired Codebase tree with files to be added
```bash
webgpu-template/
└── engine/
    ├── Cargo.toml (UPDATE: add hecs, serde dependencies)
    └── src/
        ├── core/
        │   ├── mod.rs (UPDATE: declare entity module)
        │   └── entity/
        │       ├── mod.rs (CREATE: World wrapper, exports)
        │       ├── components.rs (CREATE: Transform, GlobalTransform, Parent)
        │       ├── hierarchy.rs (CREATE: update_hierarchy_system)
        │       └── world.rs (CREATE: World wrapper with helpers)
        └── lib.rs (UPDATE: re-export entity types in prelude)
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: Module naming - NEVER do this:
// engine/src/core/entity/mod.rs declaring "mod entity;"
// Instead: entity.rs contains the module, or entity/mod.rs declares submodules

// CRITICAL: Trait imports required for operations
use std::ops::Mul; // Required for Mat4 * Mat4

// CRITICAL: hecs entities are just IDs, not objects
// Entity is Copy, components must be 'static

// CRITICAL: glam uses column-major matrices
// Parent-to-world = parent_world * child_local

// CRITICAL: Logging - NO println! Ever
use tracing::{debug, error, info, warn};

// CRITICAL: Cycle detection prevents stack overflow
// Track visited entities during traversal
```

## Implementation Blueprint

### Data models and structure

```rust
// engine/src/core/entity/components.rs
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat, 
    pub scale: Vec3,
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlobalTransform {
    pub matrix: Mat4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Parent(pub hecs::Entity);
```

### List of tasks to complete in order

```yaml
Task 1: Add dependencies
MODIFY engine/Cargo.toml:
  - ADD hecs = "0.10"
  - ADD serde = { version = "1.0", features = ["derive"] }
  - VERIFY glam already has "bytemuck" feature

Task 2: Create entity module structure
CREATE engine/src/core/entity/mod.rs:
  - PATTERN: declare submodules and re-exports
  - EXPORTS: World, Transform, GlobalTransform, Parent, update_hierarchy_system
  
MODIFY engine/src/core/mod.rs:
  - ADD: pub mod entity;

Task 3: Implement components
CREATE engine/src/core/entity/components.rs:
  - IMPLEMENT Transform with Default (identity)
  - IMPLEMENT GlobalTransform with Default (identity matrix)
  - IMPLEMENT Parent wrapping hecs::Entity
  - DERIVE Serialize, Deserialize on all

Task 4: Create World wrapper
CREATE engine/src/core/entity/world.rs:
  - WRAP hecs::World with helper methods
  - METHOD add_camera auto-adds Transform if missing
  - METHOD add_with_requirements for generic component bundles
  - PATTERN: Check existing components before adding

Task 5: Implement hierarchy system
CREATE engine/src/core/entity/hierarchy.rs:
  - FUNCTION update_hierarchy_system(world: &mut World)
  - ALGORITHM: BFS traversal starting from roots (no Parent)
  - TRACK visited entities to detect cycles
  - REUSE Vec<Entity> allocation across frames
  - LOG cycles with tracing::error!

Task 6: Update engine prelude
MODIFY engine/src/lib.rs:
  - RE-EXPORT in prelude: Transform, GlobalTransform, Parent, World
  - ENSURE existing exports remain

Task 7: Add comprehensive tests
CREATE engine/src/core/entity/tests.rs or inline:
  - TEST basic parent-child transform
  - TEST multi-level hierarchy
  - TEST cycle detection
  - TEST serialization round-trip
  - TEST helper API behavior
```

### Per task pseudocode

```rust
// Task 5: Hierarchy system pseudocode
pub fn update_hierarchy_system(world: &mut World) {
    use tracing::{debug, error};
    
    // Pre-allocate for performance
    let mut queue = Vec::with_capacity(1024);
    let mut visited = HashSet::new();
    
    // Find root entities (no Parent component)
    for (entity, (transform, _)) in world.query::<(&Transform, Without<Parent>)>().iter() {
        queue.push((entity, Mat4::IDENTITY));
        visited.insert(entity);
    }
    
    // BFS traversal
    let mut next_level = Vec::new();
    while !queue.is_empty() {
        for (entity, parent_matrix) in queue.drain(..) {
            // Update this entity's global transform
            if let Ok(transform) = world.get::<Transform>(entity) {
                let local_matrix = transform.to_matrix();
                let world_matrix = parent_matrix * local_matrix; // Column-major!
                
                world.insert_one(entity, GlobalTransform { matrix: world_matrix });
                
                // Find children
                for (child, Parent(parent)) in world.query::<&Parent>().iter() {
                    if *parent == entity {
                        if visited.contains(&child) {
                            error!(parent = ?entity, child = ?child, "Cyclic parent detected");
                            continue;
                        }
                        visited.insert(child);
                        next_level.push((child, world_matrix));
                    }
                }
            }
        }
        std::mem::swap(&mut queue, &mut next_level);
    }
}

// Task 4: World wrapper helper
impl World {
    pub fn add_camera(&mut self, camera: Camera) -> Entity {
        let entity = self.spawn((camera,));
        
        // Auto-add Transform if missing
        if self.get::<Transform>(entity).is_err() {
            self.insert_one(entity, Transform::default());
        }
        if self.get::<GlobalTransform>(entity).is_err() {
            self.insert_one(entity, GlobalTransform::default());
        }
        
        entity
    }
}
```

### Integration Points
```yaml
DEPENDENCIES:
  - add to: engine/Cargo.toml
  - hecs = "0.10"
  - serde with derive feature
  
MODULE STRUCTURE:
  - create: engine/src/core/entity/ directory
  - update: engine/src/core/mod.rs to declare entity module
  - update: engine/src/lib.rs prelude exports
  
FUTURE INTEGRATION:
  - Renderer will read GlobalTransform for MVP matrices
  - Scripts access Transform via Rhai bindings
  - Editor modifies Transform/Parent for scene editing
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all                # Format all code
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Or simply:
just preflight

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// Tests to implement in each module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transform_to_matrix() {
        let t = Transform {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let mat = t.to_matrix();
        assert_eq!(mat.w_axis.truncate(), t.position);
    }
    
    #[test]
    fn test_hierarchy_update() {
        let mut world = World::new();
        
        // Create parent at (1,0,0)
        let parent = world.spawn((
            Transform { position: Vec3::X, ..Default::default() },
            GlobalTransform::default(),
        ));
        
        // Create child at local (0,1,0)
        let child = world.spawn((
            Transform { position: Vec3::Y, ..Default::default() },
            GlobalTransform::default(),
            Parent(parent),
        ));
        
        update_hierarchy_system(&mut world);
        
        // Child should be at world (1,1,0)
        let global = world.get::<GlobalTransform>(child).unwrap();
        assert_eq!(global.matrix.w_axis.truncate(), Vec3::new(1.0, 1.0, 0.0));
    }
    
    #[test]
    fn test_cycle_detection() {
        let mut world = World::new();
        
        let a = world.spawn((Transform::default(), GlobalTransform::default()));
        let b = world.spawn((Transform::default(), GlobalTransform::default(), Parent(a)));
        
        // Create cycle: a -> b -> a
        world.insert_one(a, Parent(b));
        
        // Should not panic, just log error
        update_hierarchy_system(&mut world);
    }
}
```

```bash
# Run tests
cargo test --workspace
# Expected: All tests pass
```

### Level 3: Documentation Build
```bash
cargo doc --workspace --no-deps --document-private-items
# Expected: Builds without warnings
```

### Level 4: Integration Test
```rust
// In game/src/main.rs or a test file
use engine::prelude::*;

fn test_integration() {
    engine::init_logging();
    
    let mut world = World::new();
    
    // Test helper API
    let cam = world.add_camera(Camera::default());
    assert!(world.get::<Transform>(cam).is_ok());
    assert!(world.get::<GlobalTransform>(cam).is_ok());
    
    // Test serialization
    let json = serde_json::to_string(&Transform::default()).unwrap();
    let _: Transform = serde_json::from_str(&json).unwrap();
}
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No linting errors: `just preflight` 
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] Cycle detection works without panicking
- [ ] Helper APIs auto-add required components
- [ ] Serialization round-trips correctly
- [ ] No use of println! - only tracing macros
- [ ] Module structure follows conventions (no mod.rs conflicts)

---

## Anti-Patterns to Avoid
- ❌ Don't create mod.rs that declares module with same name as parent
- ❌ Don't use println! or eprintln! - use tracing macros
- ❌ Don't skip cycle detection - it prevents stack overflow
- ❌ Don't allocate new Vecs every frame in hierarchy update
- ❌ Don't forget to import traits (e.g., std::ops::Mul)
- ❌ Don't make components non-Copy if possible
- ❌ Don't update GlobalTransform outside the system

## Confidence Score: 9/10

The PRP provides comprehensive context including:
- All necessary documentation URLs
- Complete implementation blueprint with pseudocode
- Specific gotchas and conventions from CLAUDE.md
- Ordered task list with clear modifications
- Executable validation commands
- Test cases covering all requirements

The only uncertainty is around potential additional Camera component structure not yet defined in the codebase, but the helper API pattern is flexible enough to handle this.