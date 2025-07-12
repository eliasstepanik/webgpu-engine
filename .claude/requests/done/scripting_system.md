## FEATURE:

**Rhai Scripting System with ScriptRef Component**

Implement a scripting system using Rhai that allows loading and executing scripts to control game logic. The system should include:
* `ScriptRef` component that stores the script path/name
* Script loading from the `assets/` directory
* Entity script lifecycle functions (on_start, on_update, on_destroy)
* Access to entity components (Transform, Camera, etc.) from scripts
* World querying capabilities for finding entities with specific components
* Error handling that doesn't crash the engine

The system should integrate cleanly with the existing ECS architecture and support the scene JSON format outlined in PLANNING.md.

## EXAMPLES:

```rust
// Example usage in Rust
let camera_entity = world.spawn((
    Transform::from_position(Vec3::new(0.0, 0.0, 5.0)),
    Camera::perspective(60.0, 16.0 / 9.0, 0.1, 500.0),
    ScriptRef::new("fly_camera"), // References assets/scripts/fly_camera.rhai
));
```

```javascript
// assets/scripts/fly_camera.rhai
// Camera controller script for flying around the scene

let move_speed = 5.0;
let mouse_sensitivity = 0.1;

// Called when the entity is first spawned
fn on_start(entity) {
    print("Camera controller started for entity: " + entity);
}

// Called every frame
fn on_update(entity, delta_time) {
    let transform = world.get_component(entity, "Transform");
    
    // Handle WASD movement
    if input.is_key_pressed("W") {
        let forward = transform.forward();
        transform.position += forward * move_speed * delta_time;
    }
    
    if input.is_key_pressed("S") {
        let forward = transform.forward();
        transform.position -= forward * move_speed * delta_time;
    }
    
    if input.is_key_pressed("A") {
        let right = transform.right();
        transform.position -= right * move_speed * delta_time;
    }
    
    if input.is_key_pressed("D") {
        let right = transform.right();
        transform.position += right * move_speed * delta_time;
    }
    
    // Handle mouse look
    let mouse_delta = input.mouse_delta();
    if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
        transform.rotate_y(-mouse_delta.x * mouse_sensitivity * delta_time);
        transform.rotate_x(-mouse_delta.y * mouse_sensitivity * delta_time);
    }
}

// Called when the entity is destroyed
fn on_destroy(entity) {
    print("Camera controller destroyed for entity: " + entity);
}
```

```javascript
// assets/scripts/rotating_cube.rhai
// Simple script that rotates a cube continuously

fn on_update(entity, delta_time) {
    let transform = world.get_component(entity, "Transform");
    transform.rotate_y(delta_time);
}
```

```json
// Scene JSON example (from PLANNING.md)
{
  "entities": [
    {
      "components": {
        "Transform": {"pos": [0,0,5], "rot": [0,0,0,1], "scale": [1,1,1]},
        "Camera":    {"fov": 60.0, "near": 0.1, "far": 500.0},
        "ScriptRef": {"name": "fly_camera"}
      }
    }
  ]
}
```

## DOCUMENTATION:

* Rhai scripting language: https://rhai.rs/
* Rhai book/documentation: https://rhai.rs/book/
* Rhai GitHub repository: https://github.com/rhaiscript/rhai
* Rhai API documentation: https://docs.rs/rhai/latest/rhai/
* Embedding Rhai in Rust: https://rhai.rs/book/rust/
* Rhai module system: https://rhai.rs/book/rust/modules.html
* Custom types in Rhai: https://rhai.rs/book/rust/custom-types.html

## OTHER CONSIDERATIONS:

* **Architecture**: Follow the PLANNING.md structure - create `engine/src/scripting/` module as specified
* **ScriptRef Component**: Must be serializable/deserializable for scene loading as shown in PLANNING.md
* **Asset Organization**: Scripts should be in `assets/scripts/` directory with `.rhai` extension
* **Integration**: Must work with existing ECS patterns - entities with `ScriptRef` get script lifecycle called
* **Error Handling**: Script compilation/runtime errors should be logged but not crash the engine
* **Performance**: Consider caching compiled scripts to avoid recompilation
* **Type Mapping**: Map Rust types (Vec3, Quat, Transform) to Rhai custom types
* **World Access**: Scripts need safe access to world queries and component access
* **Input Integration**: Scripts should access the input system for interactive behaviors
* **Safety**: Rhai is already sandboxed by design - no file I/O or unsafe operations
* **Testing**: Include unit tests for script loading, execution, and error handling
* **Documentation**: Follow existing code style with doc comments on all public APIs
* **Module Structure**: Follow the existing pattern in other engine modules (mod.rs, components, systems)
* **Scene Loading**: Must integrate with the existing scene loading system in the `io` module
* **Lifecycle Management**: Scripts should be updated as part of the main game loop
* **Memory Management**: Consider script state persistence between frames for performance