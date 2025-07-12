## FEATURE:

**Renderer-Scene Integration System**

Connect the existing renderer with the scene serialization system to enable visual representation of serialized scenes. This integration should allow loading scenes and immediately rendering them with proper mesh assignments, materials, and camera setup.

Key Requirements:
* Extend scene serialization to include graphics components (MeshId, Material)
* Create renderable component system that bridges ECS entities to GPU resources
* Implement automatic mesh and material assignment for scene entities
* Add camera system integration for scene-based rendering
* Support hot-reloading of scenes during development
* Enable scene preview and debugging tools

## EXAMPLES:

```rust
// Load and render a scene with automatic graphics setup
let mut world = World::new();
let mut renderer = Renderer::new(&render_context)?;

// Load scene with graphics components
world.load_scene("assets/scenes/demo_scene.json")?;

// Automatic mesh assignment based on entity hierarchy
world.assign_default_meshes(&mut renderer)?;

// Render the scene
let camera_entity = world.query::<&Camera>().iter().next().unwrap().0;
renderer.render_world(&world, camera_entity)?;

// Scene format with graphics components
{
  "entities": [
    {
      "components": {
        "Transform": {"position": [0,0,0], "rotation": [0,0,0,1], "scale": [1,1,1]},
        "MeshId": {"id": "cube"},
        "Material": {"base_color": [1.0, 0.5, 0.2, 1.0], "metallic": 0.0, "roughness": 0.5},
        "Camera": {"fov_y_radians": 1.047, "aspect_ratio": 1.777, "z_near": 0.1, "z_far": 1000.0}
      }
    }
  ]
}

// Hot-reload scenes during development
world.watch_scene("assets/scenes/level1.json", |world, renderer| {
    world.clear_graphics_components();
    world.assign_default_meshes(renderer)?;
})?;

// Scene debugging and preview
let scene_stats = world.get_scene_stats();
println!("Renderables: {}, Cameras: {}, Lights: {}", 
         scene_stats.renderable_count, 
         scene_stats.camera_count,
         scene_stats.light_count);
```

## DOCUMENTATION:

* WebGPU rendering pipeline: https://sotrh.github.io/learn-wgpu/
* Scene graphs and rendering: https://paroj.github.io/gltf-tutorial/
* ECS rendering patterns: https://github.com/bevyengine/bevy/tree/main/crates/bevy_render
* Material and mesh management: https://docs.rs/wgpu/latest/wgpu/
* Hot-reloading techniques: https://fasterthanli.me/articles/so-you-want-to-live-reload-rust
* Scene debugging tools: https://github.com/microsoft/DirectX-Graphics-Samples/tree/master/MiniEngine

## OTHER CONSIDERATIONS:

**Graphics Component Serialization:**
* MeshId components need string-based mesh references that can be resolved at load time
* Material components should serialize all parameters (base_color, metallic, roughness, etc.)
* Consider mesh library/asset management for predefined shapes (cube, sphere, plane)
* Handle missing mesh references gracefully (use default cube/error mesh)

**Rendering Integration:**
* Current renderer expects entities to have Transform + MeshId + Material
* Need to bridge between ECS world state and GPU buffer updates
* Camera entity selection and view matrix calculation from GlobalTransform
* Efficient rendering of hierarchical transforms (use GlobalTransform matrices)

**Performance Considerations:**
* Only re-upload changed transform/material data to GPU
* Batch similar materials/meshes for efficient draw calls
* Use transform hierarchy system for LOD and culling
* Consider instanced rendering for repeated mesh types

**Development Workflow:**
* Scene files should be hot-reloadable during development
* Visual feedback when scene loading fails (error meshes, debug info)
* Scene debugging overlay showing entity bounds, hierarchy, camera frustum
* Integration with existing logging system for render statistics

**Asset Management:**
* Standardize mesh naming conventions (cube, sphere, plane, etc.)
* Material presets for common surface types
* Default fallbacks when assets are missing
* Asset dependency tracking for scene files

**Camera System:**
* Support multiple cameras in scene with active camera selection
* Automatic aspect ratio updates on window resize
* Camera controls integration (fly camera, orbit camera)
* Viewport and scissor rect support for multiple views

**Scene Validation:**
* Validate scene integrity on load (circular hierarchies, missing references)
* Sanity checks for transform values and material parameters
* Warning system for potential rendering issues (extreme scales, invalid materials)
* Performance hints for complex scenes (entity count, draw call estimates)

**Error Handling:**
* Graceful degradation when graphics resources fail to load
* Visual indicators for missing components (wireframe, error texture)
* Comprehensive error reporting with entity context
* Recovery mechanisms for corrupted scene files

**Integration Points:**
* World convenience methods for graphics operations
* Renderer::render_world() method for complete scene rendering
* Scene statistics and debugging information
* Hot-reload infrastructure for development iteration