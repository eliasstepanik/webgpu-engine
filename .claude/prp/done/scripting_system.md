name: "Rhai Scripting System - WebGPU Template"
description: |

## Purpose
Implement a flexible Rhai-based scripting system that allows entities to execute scripts with access to components, input, and world queries. Scripts should support lifecycle functions (on_start, on_update, on_destroy) and integrate seamlessly with the existing ECS architecture and scene loading system.

## Core Principles
1. **Safety First**: Scripts are sandboxed - no file I/O, network, or unsafe operations
2. **Performance**: Cache compiled scripts to avoid recompilation per frame
3. **Integration**: Works with existing scene JSON format and ECS patterns
4. **Error Resilience**: Script errors log warnings but don't crash the engine
5. **Developer Experience**: Clear error messages with script line numbers

---

## Goal
Create a scripting system that enables:
- Entity behavior through script lifecycle functions
- Access to entity components (Transform, Camera, etc.) from scripts
- Input handling from scripts (keyboard, mouse)
- World queries to find entities with specific components
- Scene-defined script references that load automatically

## Why
- **Rapid Prototyping**: Iterate on gameplay without recompiling
- **Modding Support**: Users can create custom behaviors
- **Separation of Concerns**: Game logic in scripts, engine logic in Rust
- **Hot Reload**: Scripts can be modified while the game runs

## What
Implement a complete scripting module with:
- ScriptRef component for attaching scripts to entities
- Script loading and caching system
- Rhai engine with custom types for Vec3, Quat, Transform
- World API for component access and queries
- Input API for keyboard and mouse state
- Script execution system integrated with game loop
- Error handling and logging
- Scene loading integration

### Success Criteria
- [ ] ScriptRef component serializes/deserializes in scene JSON
- [ ] Scripts load from assets/scripts/ directory
- [ ] on_start, on_update, on_destroy lifecycle functions work
- [ ] Scripts can read/write Transform components
- [ ] Scripts can query keyboard/mouse input
- [ ] Script errors don't crash the engine
- [ ] Example fly_camera.rhai script controls camera
- [ ] Example rotating_cube.rhai script rotates entities
- [ ] Unit tests pass for script loading and execution

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://rhai.rs/book/
  why: Complete Rhai documentation with embedding guide
  critical: Start with "Getting Started" and "Using Rhai" sections
  
- url: https://rhai.rs/book/rust/modules.html
  why: How to create Rhai modules for exposing Rust APIs
  section: Focus on "Creating a Module" and "Module Resolvers"
  
- url: https://rhai.rs/book/rust/custom-types.html
  why: Register custom types like Vec3, Transform with Rhai
  critical: getters/setters and operator overloading patterns
  
- url: https://docs.rs/rhai/latest/rhai/
  why: Rhai API documentation for Engine, Scope, Module types
  
- url: https://github.com/rhaiscript/rhai/tree/main/examples
  why: Practical examples of Rhai integration
  section: Look at custom_types.rs and modules.rs examples
  
- file: engine/src/core/entity/components.rs
  why: Transform and other component definitions to expose
  
- file: engine/src/io/scene.rs
  why: Scene loading pattern for adding ScriptRef support
  lines: 195-300 show component deserialization pattern
  
- file: game/src/main.rs
  why: Game loop where script system needs integration
  lines: 84-94 show update location
  
- file: PLANNING.md
  why: Architecture guidelines and module structure
  lines: 30-38 confirm scripting module location
```

### Current Codebase Structure
```bash
webgpu-template/
├── assets/
│   └── scenes/              # Scene JSON files
├── engine/
│   ├── src/
│   │   ├── core/           # ECS and components
│   │   ├── graphics/       # Rendering
│   │   ├── input/          # Input module (minimal)
│   │   ├── io/             # Scene loading
│   │   └── scripting/      # TO BE CREATED
└── game/src/main.rs        # Main loop integration point
```

### Desired Structure
```bash
engine/src/scripting/
├── mod.rs                  # Module exports
├── components.rs           # ScriptRef component
├── engine.rs              # Rhai engine setup and caching
├── modules/               # Rhai modules for APIs
│   ├── mod.rs
│   ├── input.rs          # Input API module
│   ├── math.rs           # Vec3, Quat custom types
│   └── world.rs          # World query API
├── script.rs             # Script loading and compilation
└── system.rs             # Script execution system

assets/scripts/           # Script files
├── fly_camera.rhai
└── rotating_cube.rhai
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: Rhai 1.19+ required for no_std support
// Use: rhai = { version = "1.19", features = ["sync"] }

// GOTCHA: Custom types must be cloned when passed to/from Rhai
// Scripts get copies, not references to components

// PATTERN: Use Rc<RefCell<T>> for mutable shared state
// But our ECS already handles mutability, so we clone

// GOTCHA: Rhai functions can't have lifetime parameters
// Must use owned types or static references

// PERFORMANCE: Cache Engine instances, they're expensive to create
// Use Arc<RwLock<HashMap<String, Arc<Engine>>>> for thread-safe cache

// ERROR HANDLING: Rhai errors include position info
// Format as: "script.rhai:10:5 - undefined variable 'x'"

// LIMITATION: Rhai doesn't support async functions
// All script functions must be synchronous

// SECURITY: Rhai is sandboxed by default
// No file I/O, network, or process spawning
```

## Implementation Blueprint

### Data Models and Structure

```rust
// engine/src/scripting/components.rs
use serde::{Deserialize, Serialize};

/// Reference to a script asset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ScriptRef {
    /// Script name without extension (e.g., "fly_camera")
    pub name: String,
}

impl ScriptRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
    
    /// Get the full path to the script file
    pub fn path(&self) -> String {
        format!("assets/scripts/{}.rhai", self.name)
    }
}

// engine/src/scripting/engine.rs
use rhai::{Engine, Scope, AST, Dynamic};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

/// Cached script data
struct CachedScript {
    ast: AST,
    has_on_start: bool,
    has_on_update: bool, 
    has_on_destroy: bool,
}

/// Script engine with caching
pub struct ScriptEngine {
    engine: Arc<Engine>,
    cache: Arc<RwLock<HashMap<String, CachedScript>>>,
}

// engine/src/scripting/modules/input.rs
/// Input state accessible from scripts
#[derive(Clone)]
pub struct ScriptInputState {
    keys_pressed: HashSet<String>,
    mouse_position: (f32, f32),
    mouse_delta: (f32, f32),
    mouse_buttons: HashSet<u8>,
}

// engine/src/scripting/system.rs
/// System to execute scripts on entities
pub fn script_execution_system(
    world: &mut World,
    script_engine: &mut ScriptEngine,
    input_state: &ScriptInputState,
    delta_time: f32,
) {
    // Implementation details below
}
```

### Module Setup Pattern

```rust
// engine/src/scripting/modules/math.rs
use rhai::{Engine, Module};
use glam::{Vec3, Quat};

/// Register math types with Rhai
pub fn register_math_types(engine: &mut Engine) {
    // Vec3 type
    engine.register_type_with_name::<Vec3>("Vec3")
        .register_fn("new", |x: f64, y: f64, z: f64| {
            Vec3::new(x as f32, y as f32, z as f32)
        })
        .register_get("x", |v: &mut Vec3| v.x as f64)
        .register_set("x", |v: &mut Vec3, x: f64| v.x = x as f32)
        .register_get("y", |v: &mut Vec3| v.y as f64)
        .register_set("y", |v: &mut Vec3, y: f64| v.y = y as f32)
        .register_get("z", |v: &mut Vec3| v.z as f64)
        .register_set("z", |v: &mut Vec3, z: f64| v.z = z as f32)
        .register_fn("+", |a: Vec3, b: Vec3| a + b)
        .register_fn("-", |a: Vec3, b: Vec3| a - b)
        .register_fn("*", |a: Vec3, b: f64| a * b as f32);
        
    // More math functions...
}
```

### Script Loading and Execution

```rust
// Pseudocode for script execution flow
impl ScriptEngine {
    pub fn load_script(&self, script_ref: &ScriptRef) -> Result<(), Box<EvalAltResult>> {
        let path = script_ref.path();
        
        // Check cache first
        if self.cache.read().unwrap().contains_key(&script_ref.name) {
            return Ok(());
        }
        
        // Load and compile script
        let script_content = std::fs::read_to_string(&path)?;
        let ast = self.engine.compile(&script_content)?;
        
        // Check which lifecycle functions exist
        let has_on_start = ast.has_function("on_start", 1);
        let has_on_update = ast.has_function("on_update", 2);
        let has_on_destroy = ast.has_function("on_destroy", 1);
        
        // Cache the compiled script
        self.cache.write().unwrap().insert(
            script_ref.name.clone(),
            CachedScript { ast, has_on_start, has_on_update, has_on_destroy }
        );
        
        Ok(())
    }
    
    pub fn call_on_update(
        &self,
        script_name: &str,
        entity_id: u64,
        scope: &mut Scope,
        delta_time: f32
    ) -> Result<(), Box<EvalAltResult>> {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.get(script_name) {
            if cached.has_on_update {
                self.engine.call_fn(
                    scope,
                    &cached.ast,
                    "on_update",
                    (entity_id as i64, delta_time as f64)
                )?;
            }
        }
        Ok(())
    }
}
```

### List of Tasks to Complete (in order)

```yaml
Task 1: Create base scripting module structure
CREATE engine/src/scripting/mod.rs:
  - Export public types and systems
  - Follow pattern from other engine modules
  
CREATE engine/src/scripting/components.rs:
  - Define ScriptRef component with serde derives
  - Add path() helper method
  - Add tests for serialization

Task 2: Implement Rhai engine wrapper
CREATE engine/src/scripting/engine.rs:
  - ScriptEngine struct with Arc<Engine> and cache
  - Script loading with AST compilation
  - Lifecycle function detection
  - Error formatting with file:line:column
  
ADD to Cargo.toml:
  - rhai = { version = "1.19", features = ["sync"] }

Task 3: Create math module for Rhai
CREATE engine/src/scripting/modules/mod.rs:
  - Export all modules
  
CREATE engine/src/scripting/modules/math.rs:
  - Register Vec3 type with operators
  - Register Quat type (at least slerp)
  - Register common math functions
  - Transform component access helpers

Task 4: Create world API module
CREATE engine/src/scripting/modules/world.rs:
  - get_component(entity, component_type) function
  - set_component(entity, component_type, value) function
  - find_entities_with_component(component_type) function
  - Use dynamic dispatch for component access

Task 5: Create input module  
CREATE engine/src/scripting/modules/input.rs:
  - ScriptInputState struct
  - is_key_pressed(key_name) function
  - mouse_position() function
  - mouse_delta() function
  - is_mouse_button_pressed(button) function

CREATE engine/src/input/state.rs:
  - Basic input state tracking
  - Update from winit events

Task 6: Implement script execution system
CREATE engine/src/scripting/system.rs:
  - script_execution_system function
  - Query entities with ScriptRef component
  - Create scope with entity ID
  - Call appropriate lifecycle functions
  - Handle errors gracefully with logging

Task 7: Integrate with scene loading
MODIFY engine/src/io/scene.rs:
  - Add "ScriptRef" case to component match (~line 265)
  - Follow pattern of other components
  
MODIFY engine/src/io/component_registry.rs:
  - Register ScriptRef in with_default_components()

Task 8: Integrate with game loop
MODIFY game/src/main.rs:
  - Create ScriptEngine in main()
  - Add input state tracking
  - Call script_execution_system in update loop
  - Handle WindowEvent::KeyboardInput and MouseInput

Task 9: Create example scripts
CREATE assets/scripts/fly_camera.rhai:
  - Implement WASD movement
  - Mouse look controls
  - Use delta_time for frame-independent movement

CREATE assets/scripts/rotating_cube.rhai:
  - Simple rotation example
  - Show component access pattern

Task 10: Add tests and documentation
CREATE tests in each module file:
  - Script loading and caching
  - Component access from scripts
  - Error handling
  - Lifecycle function calls
```

### Integration Points

```yaml
SCENE LOADING:
  - ScriptRef deserializes from JSON: {"ScriptRef": {"name": "fly_camera"}}
  - Automatically loads referenced scripts on scene instantiation
  
GAME LOOP:
  - Scripts update after game logic, before hierarchy update
  - Delta time passed to all on_update functions
  - Input state captured at start of frame
  
ERROR HANDLING:
  - Script compilation errors prevent entity spawn
  - Runtime errors log warning with script:line:column
  - Missing scripts log error but don't crash
  
COMPONENT ACCESS:
  - Scripts work with component clones, not references
  - Changes written back after script execution
  - Invalid component access returns nil/unit
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
// Test script loading
#[test]
fn test_script_ref_serialization() {
    let script_ref = ScriptRef::new("test_script");
    let json = serde_json::to_string(&script_ref).unwrap();
    assert_eq!(json, r#"{"name":"test_script"}"#);
}

// Test script execution
#[test]
fn test_script_lifecycle() {
    let engine = ScriptEngine::new();
    // Create test script in memory
    let script = r#"
        fn on_start(entity) { 
            print("Started: " + entity);
        }
    "#;
    
    // Compile and verify
    let ast = engine.engine.compile(script).unwrap();
    assert!(ast.has_function("on_start", 1));
}

// Test component access
#[test]
fn test_transform_access() {
    let mut engine = Engine::new();
    register_math_types(&mut engine);
    
    let result: Vec3 = engine.eval("Vec3::new(1.0, 2.0, 3.0)").unwrap();
    assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
}
```

### Level 3: Integration Test
```bash
# Create test assets
mkdir -p assets/scripts
cat > assets/scripts/test_rotate.rhai << 'EOF'
fn on_update(entity, delta_time) {
    let transform = world.get_component(entity, "Transform");
    transform.rotate_y(delta_time);
    world.set_component(entity, "Transform", transform);
}
EOF

# Test in game
just run

# Verify:
# 1. No script compilation errors in logs
# 2. Entities with ScriptRef("test_rotate") rotate
# 3. Script errors show file:line:column
```

### Level 4: Example Scripts
```javascript
// assets/scripts/fly_camera.rhai
let move_speed = 5.0;
let look_speed = 2.0;

fn on_start(entity) {
    print("Fly camera initialized for entity " + entity);
}

fn on_update(entity, delta_time) {
    let transform = world.get_component(entity, "Transform");
    
    // Movement
    let move_delta = Vec3::new(0.0, 0.0, 0.0);
    
    if input.is_key_pressed("W") {
        move_delta.z -= 1.0;
    }
    if input.is_key_pressed("S") {
        move_delta.z += 1.0;
    }
    if input.is_key_pressed("A") {
        move_delta.x -= 1.0;
    }
    if input.is_key_pressed("D") {
        move_delta.x += 1.0;
    }
    
    // Apply movement in local space
    if move_delta.length() > 0.0 {
        move_delta = move_delta.normalize() * move_speed * delta_time;
        transform.position += transform.rotate_vector(move_delta);
    }
    
    // Mouse look
    let mouse_delta = input.mouse_delta();
    if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
        transform.rotate_y(-mouse_delta.x * look_speed * delta_time);
        transform.rotate_x(-mouse_delta.y * look_speed * delta_time);
    }
    
    world.set_component(entity, "Transform", transform);
}
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Scripts load from assets/scripts/ directory
- [ ] Scene JSON with ScriptRef works
- [ ] fly_camera.rhai controls camera with WASD + mouse
- [ ] rotating_cube.rhai rotates entities
- [ ] Script errors don't crash engine
- [ ] Script errors show file:line:column
- [ ] Documentation complete: `cargo doc --workspace --no-deps`

---

## Anti-Patterns to Avoid
- ❌ Don't expose unsafe operations to scripts
- ❌ Don't recompile scripts every frame - use caching
- ❌ Don't panic on script errors - log and continue
- ❌ Don't allow scripts to spawn/destroy entities directly
- ❌ Don't expose raw pointers or references to scripts
- ❌ Don't use global state - pass everything through scope

## Confidence Score: 8/10

Strong foundation with clear patterns from existing codebase. Two points deducted for:
1. Input system needs to be built alongside scripting
2. Component access pattern requires careful dynamic dispatch design

The implementation path is clear with good examples and documentation available.