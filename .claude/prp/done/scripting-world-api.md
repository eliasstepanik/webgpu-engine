name: "Scripting World API - Command Pattern Implementation"
description: |

## Purpose
Implement a command pattern for safe world access from Rhai scripts, enabling scripts to query and modify ECS components through deferred execution.

## Core Principles
1. **Thread Safety First**: Use Arc<RwLock> pattern proven in input module
2. **Command Queue Pattern**: Queue mutations for deferred execution
3. **Type Registry**: Enable dynamic component access by name
4. **Progressive Implementation**: Start with Transform/Material, expand later
5. **Follow CLAUDE.md**: Adhere to all project guidelines

---

## Goal
Enable Rhai scripts to safely read and write ECS components (Transform, Material, Name) through a command queue system that defers mutations until after script execution, fixing the current placeholder implementations that return hardcoded values.

## Why
- Scripts currently cannot interact with the game world despite API calls in example scripts
- Developers expect scripting to enable gameplay logic and entity manipulation
- Current placeholder functions mislead users about functionality
- Thread safety constraints prevent direct world access from scripts

## What
Replace placeholder world module functions with a command queue system that:
- Allows scripts to query component values from entities
- Queues component modifications for deferred application
- Maintains thread safety through Arc<RwLock> pattern
- Provides clear execution model for script developers

### Success Criteria
- [ ] Scripts can read Transform components from entities
- [ ] Scripts can modify Transform components (position, rotation, scale)
- [ ] Scripts can read/write Material components
- [ ] Scripts can query entities by component type
- [ ] All example scripts (rotating_cube.rhai, fly_camera.rhai) work correctly
- [ ] Thread safety maintained throughout
- [ ] Performance impact measured and acceptable

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/bevy/latest/bevy/ecs/system/struct.Commands.html
  why: Bevy's command pattern implementation reference
  
- url: https://rhai.rs/book/about/features.html
  why: Rhai sync feature documentation for thread safety
  
- url: https://github.com/rhaiscript/rhai/issues/95
  why: Discussion of Rhai ECS integration patterns and solutions
  
- file: engine/src/scripting/modules/input.rs
  why: Existing Arc<RwLock> pattern to follow for thread-safe state sharing
  
- file: engine/src/scripting/modules/world.rs
  why: Current placeholder implementation to replace
  
- file: assets/scripts/rotating_cube.rhai
  why: Example script showing expected world API usage
  
- file: assets/scripts/fly_camera.rhai
  why: Example script showing Transform manipulation needs
  
- file: engine/src/scripting/system.rs
  why: Script execution system where commands will be processed
  
- file: engine/src/core/entity/components.rs
  why: Transform component definition
  
- file: engine/src/graphics/material.rs
  why: Material component definition
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── scripting/
│   │   ├── mod.rs
│   │   ├── engine.rs
│   │   ├── script.rs
│   │   ├── system.rs
│   │   ├── components.rs
│   │   └── modules/
│   │       ├── mod.rs
│   │       ├── input.rs      # Thread-safe pattern example
│   │       ├── math.rs       # Transform registration
│   │       └── world.rs      # Placeholder to replace
│   ├── core/
│   │   └── entity/
│   │       └── components.rs # Transform component
│   └── graphics/
│       └── material.rs       # Material component
assets/
└── scripts/
    ├── rotating_cube.rhai    # Example needing world access
    └── fly_camera.rhai       # Example needing world access
```

### Desired Codebase Structure
```bash
engine/
├── src/
│   ├── scripting/
│   │   ├── commands.rs       # NEW: Command definitions and queue
│   │   ├── component_access.rs # NEW: Type-safe component access
│   │   └── modules/
│   │       └── world.rs      # MODIFIED: Implement command queueing
```

### Known Gotchas & Constraints
```rust
// CRITICAL: World has raw pointers, cannot be Send+Sync
// Solution: Use command queue with Arc<RwLock<Vec<ScriptCommand>>>

// CRITICAL: Rhai sync feature already enabled in Cargo.toml
// This allows Engine, AST, Scope to be Send+Sync

// PATTERN: Input module shows exact Arc<RwLock> pattern:
// 1. Create shared state: Arc<RwLock<T>>
// 2. Clone Arc for each closure
// 3. Use move closures capturing the Arc
// 4. Access with .read()/.write().unwrap()

// GOTCHA: Scripts use string-based component names
// Need runtime type resolution for "Transform", "Material", etc.

// GOTCHA: Entity IDs are u64 in World but i64 in Rhai
// Need conversion and validation

// PERFORMANCE: Commands accumulate during frame
// Clear queue after processing to prevent memory growth
```

## Implementation Blueprint

### Data Models and Structure

Create command types and queue structure:
```rust
// engine/src/scripting/commands.rs
use std::sync::{Arc, RwLock};
use crate::core::entity::components::Transform;
use crate::graphics::material::Material;

#[derive(Clone, Debug)]
pub enum ScriptCommand {
    SetTransform { entity: u64, transform: Transform },
    SetMaterial { entity: u64, material: Material },
    CreateEntity { components: Vec<ComponentData> },
    DestroyEntity { entity: u64 },
}

#[derive(Clone, Debug)]
pub enum ComponentData {
    Transform(Transform),
    Material(Material),
    Name(String),
}

pub type CommandQueue = Arc<RwLock<Vec<ScriptCommand>>>;

// For querying components - store results in script-accessible cache
#[derive(Clone, Default)]
pub struct ComponentCache {
    transforms: HashMap<u64, Transform>,
    materials: HashMap<u64, Material>,
    names: HashMap<u64, String>,
}

pub type SharedComponentCache = Arc<RwLock<ComponentCache>>;
```

### List of Tasks

```yaml
Task 1: Create command system infrastructure
CREATE engine/src/scripting/commands.rs:
  - Define ScriptCommand enum with all mutation types
  - Define ComponentData enum for runtime component storage
  - Create CommandQueue type alias
  - Create ComponentCache for query results
  - Add to engine/src/scripting/mod.rs exports

Task 2: Create component access utilities
CREATE engine/src/scripting/component_access.rs:
  - Create query_component function that populates cache
  - Create apply_command function that executes commands on World
  - Handle entity ID validation (u64 exists in World)
  - Add error handling for missing entities/components

Task 3: Update world module with command queueing
MODIFY engine/src/scripting/modules/world.rs:
  - Remove placeholder implementations
  - Add command_queue: CommandQueue parameter
  - Add component_cache: SharedComponentCache parameter
  - Implement get_component to read from cache
  - Implement set_component to queue commands
  - Implement find_entities_with_component using cache
  - Update create_world_module signature

Task 4: Update script execution system
MODIFY engine/src/scripting/system.rs:
  - Add CommandQueue resource initialization
  - Add ComponentCache resource initialization
  - Before scripts run: populate cache with current component values
  - After scripts run: drain command queue and apply commands
  - Clear cache after frame to prevent stale data

Task 5: Register command types with type registry
MODIFY engine/src/scripting/engine.rs:
  - Ensure Transform is properly registered
  - Ensure Material is properly registered
  - Add any missing type registrations

Task 6: Add tests for command system
CREATE engine/src/scripting/commands_test.rs:
  - Test command queueing
  - Test component cache population
  - Test command application
  - Test error cases (missing entities)

Task 7: Update example scripts documentation
MODIFY assets/scripts/rotating_cube.rhai:
  - Add comments explaining deferred execution model
MODIFY assets/scripts/fly_camera.rhai:
  - Add comments about when changes are applied
```

### Per Task Implementation Details

```rust
// Task 1: Command system infrastructure
// Key insight: Follow input module's Arc<RwLock> pattern exactly

// Task 2: Component access implementation
pub fn query_component(world: &World, entity_id: u64, component_type: &str, cache: &mut ComponentCache) -> Result<(), String> {
    // PATTERN: Use world.get::<T>(entity) for type-safe access
    let entity = Entity::from_raw(entity_id);
    
    match component_type {
        "Transform" => {
            if let Ok(transform) = world.get::<Transform>(entity) {
                cache.transforms.insert(entity_id, transform.clone());
                Ok(())
            } else {
                Err("Entity missing Transform".to_string())
            }
        }
        // Similar for Material, Name
    }
}

// Task 3: World module implementation
// CRITICAL: Clone Arc for each closure like input module does
let queue = command_queue.clone();
let cache = component_cache.clone();
module.set_native_fn("get_transform", move |entity_id: i64| -> Result<Dynamic, Box<EvalAltResult>> {
    let cache = cache.read().unwrap();
    if let Some(transform) = cache.transforms.get(&(entity_id as u64)) {
        Ok(Dynamic::from(transform.clone()))
    } else {
        Err("Entity not found".into())
    }
});

// Task 4: System execution flow
// BEFORE scripts run:
for (entity, (transform, material, name)) in world.query::<(&Transform, Option<&Material>, Option<&Name>)>().iter() {
    let mut cache = component_cache.write().unwrap();
    cache.transforms.insert(entity.id(), transform.clone());
    // Populate materials and names if present
}

// AFTER scripts run:
let commands = command_queue.write().unwrap().drain(..).collect::<Vec<_>>();
for command in commands {
    apply_command(&mut world, command)?;
}
component_cache.write().unwrap().clear(); // Prevent stale data
```

### Integration Points
```yaml
RESOURCES:
  - add to: World resources in script_execution_system
  - pattern: "world.insert_resource(CommandQueue::default())"
  - pattern: "world.insert_resource(SharedComponentCache::default())"
  
MODULE REGISTRATION:
  - modify: create_world_module function signature
  - add parameters: command_queue, component_cache
  - update call site in script_execution_system
  
ERROR HANDLING:
  - pattern: Use Result<Dynamic, Box<EvalAltResult>> for Rhai functions
  - log errors with tracing::error! not println!
```

## Validation Loop

### Level 1: Syntax & Formatting
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// Create tests for each component:
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_queue_thread_safety() {
        let queue = CommandQueue::default();
        let q1 = queue.clone();
        let q2 = queue.clone();
        
        // Verify can write from multiple threads
        std::thread::spawn(move || {
            q1.write().unwrap().push(ScriptCommand::SetTransform {
                entity: 1,
                transform: Transform::default(),
            });
        }).join().unwrap();
        
        assert_eq!(queue.read().unwrap().len(), 1);
    }
    
    #[test]
    fn test_component_cache_query() {
        // Test cache population and retrieval
    }
    
    #[test]
    fn test_command_application() {
        // Test each command type applies correctly
    }
}
```

```bash
# Run tests:
cargo test --package engine scripting
```

### Level 3: Integration Test
```bash
# Test with example scripts:
just run

# In game, verify:
# 1. Rotating cube actually rotates (not just placeholder)
# 2. Fly camera can move with WASD
# 3. Check logs for any script errors

# Verify no performance regression:
# FPS should remain stable with scripts running
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Format check passes: `cargo fmt --all -- --check`
- [ ] Docs build: `cargo doc --workspace --no-deps --document-private-items`
- [ ] Example scripts work correctly in game
- [ ] No memory leaks (command queue cleared each frame)
- [ ] Thread safety maintained (no data races)
- [ ] Performance acceptable (measure with cargo bench if needed)

---

## Anti-Patterns to Avoid
- ❌ Don't pass raw World pointers to scripts
- ❌ Don't hold locks across script execution
- ❌ Don't forget to clear caches between frames
- ❌ Don't use println! - use tracing crate
- ❌ Don't skip entity validation before access
- ❌ Don't ignore Result types - handle errors

## Performance Considerations
- Commands are batched and applied once per frame
- Cache is populated before scripts run (one query)
- Consider limiting command queue size to prevent runaway scripts
- Profile with many entities to ensure query performance is acceptable

## Future Enhancements (Not in scope)
- Support more component types dynamically
- Add spatial queries (find entities in radius)
- Implement event system for script communication
- Add component creation/removal beyond predefined types

---

**Confidence Score: 9/10**

This PRP provides comprehensive context including:
- Exact patterns to follow from the codebase (input module)
- External documentation for command patterns
- Clear implementation steps with code examples
- Validation gates that can be executed
- Known gotchas and solutions

The only uncertainty is around dynamic component type resolution, but the proposed string-based approach with a fixed set of components is pragmatic for initial implementation.