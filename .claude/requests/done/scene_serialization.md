## FEATURE:

**Scene Serialization System**

Implement scene loading and saving functionality to enable persistence of entity hierarchies, transforms, and components. This allows for level editing and scene management.

* Create scene format based on JSON (as shown in PLANNING.md)
* Implement entity serialization with component data
* Handle Parent component serialization (entity ID remapping)
* Support loading scenes with proper entity relationship restoration
* Add scene asset management in the io module
* Include component registry for dynamic deserialization

## EXAMPLES:

```rust
// Save current world to a scene file
let scene = Scene::from_world(&world);
scene.save_to_file("assets/scenes/level1.json")?;

// Load a scene into the world
let loaded_scene = Scene::load_from_file("assets/scenes/level1.json")?;
loaded_scene.instantiate(&mut world);

// Scene JSON format (from PLANNING.md)
{
  "entities": [
    {
      "components": {
        "Transform": {"pos": [0,0,5], "rot": [0,0,0,1], "scale": [1,1,1]},
        "Camera": {"fov": 60.0, "near": 0.1, "far": 500.0},
        "Parent": {"entity_id": 0}  // Will be remapped on load
      }
    }
  ]
}
```

## DOCUMENTATION:

* serde JSON serialization: https://serde.rs/json.html
* hecs world serialization examples: https://github.com/Ralith/hecs/blob/master/examples/serialize.rs
* Entity remapping patterns: https://docs.rs/bevy/latest/bevy/scene/
* Component registration: https://serde.rs/impl-deserialize.html

## OTHER CONSIDERATIONS:

* Parent component needs special handling due to Entity IDs
* Entity IDs must be remapped when loading scenes
* Use serde's tag-based polymorphism for component types
* Support partial scene loading (additive loading)
* Handle missing component types gracefully
* Version compatibility for scene format evolution
* Follow existing Transform component serialization pattern
* Scene files should be human-readable and editable
* Support both individual entity and full world serialization
* Consider prefab/template system for reusable entity groups
* Integrate with existing io module structure
* Add tests for serialization round-trips
* Handle circular parent references during load