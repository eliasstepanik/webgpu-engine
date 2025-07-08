# Demo Scenes

This directory contains example scene files demonstrating the scene serialization system.

## Files

### `demo_scene.json`
A hand-crafted scene showcasing:
- Main camera positioned at (0, 0, 5) looking at the origin
- Root object at origin with two child objects
- One child has its own sub-child (3-level hierarchy)
- Ground plane for reference
- Various transforms, rotations, and scales

### `demo_scene_generated.json`
Programmatically generated scene created by running:
```bash
cargo run --package engine --example scene_demo
```

This scene features:
- Orbital system with 4 objects orbiting around a central root
- Each orbiter has a satellite child
- Some satellites have sub-satellites (up to 3 levels deep)
- Camera positioned above looking down at the scene
- Ground plane and floating objects
- Complex parent-child relationships (10 total)

## Scene Structure

Each scene file follows this JSON format:
```json
{
  "entities": [
    {
      "components": {
        "Transform": {
          "position": [x, y, z],
          "rotation": [x, y, z, w],
          "scale": [x, y, z]
        },
        "GlobalTransform": {
          "matrix": [16 floats representing 4x4 matrix]
        },
        "Camera": {
          "fov_y_radians": float,
          "aspect_ratio": float,
          "z_near": float,
          "z_far": float,
          "projection_mode": "Perspective" | {"Orthographic": {"height": float}}
        },
        "Parent": {
          "entity_id": integer
        }
      }
    }
  ]
}
```

## Usage

### Load a scene into your world:
```rust
// Replace current world content
world.load_scene("assets/scenes/demo_scene.json")?;

// Load additively (keeping existing entities)
let mapper = world.load_scene_additive("assets/scenes/demo_scene.json")?;
```

### Save current world as scene:
```rust
world.save_scene("assets/scenes/my_scene.json")?;
```

### Direct Scene API:
```rust
use engine::prelude::*;

// Create from world
let scene = Scene::from_world(&world);
scene.save_to_file("my_scene.json")?;

// Load from file
let scene = Scene::load_from_file("my_scene.json")?;
let mapper = scene.instantiate(&mut world)?;
```

## Notes

- Entity IDs in the JSON are automatically remapped when loading
- Parent component references use these remapped IDs
- Unknown component types are logged as warnings but don't fail loading
- GlobalTransform matrices are computed from Transform during hierarchy updates
- All scenes are human-readable and editable