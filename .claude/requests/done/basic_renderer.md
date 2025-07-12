## FEATURE:

**Basic WebGPU Renderer System**

Implement a basic rendering system that can draw simple 3D objects using the transform hierarchy and camera system. This will establish the foundation for the graphics pipeline.

* Create render context with wgpu device, queue, and surface
* Implement basic vertex/fragment shaders for 3D rendering
* Add mesh component and basic primitives (cube, sphere, plane)
* Create render system that uses GlobalTransform for object positioning
* Implement camera uniform buffer for view/projection matrices
* Basic depth buffer and render pass setup

## EXAMPLES:

```rust
// Spawn a cube in the world
world.spawn((
    Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
    GlobalTransform::default(),
    Mesh::cube(1.0),
    Material::default(), // Basic colored material
));

// Render system usage
render_system.render(&world, &camera_entity, &render_context);
```

## DOCUMENTATION:

* wgpu tutorial series: https://sotrh.github.io/learn-wgpu/
* wgpu examples: https://github.com/gfx-rs/wgpu/tree/trunk/examples
* WebGPU spec for understanding concepts: https://www.w3.org/TR/webgpu/
* Render pass best practices: https://github.com/gfx-rs/wgpu/wiki/Encapsulating-Graphics-Work

## OTHER CONSIDERATIONS:

* Must integrate with existing ECS (query entities with Mesh + GlobalTransform)
* Use GlobalTransform matrix directly for MVP calculation
* Follow wgpu best practices for resource management
* Create abstraction for render pipelines to allow easy extension
* Vertex format should support position, normal, UV coordinates
* Use uniform buffers for per-object transforms and camera data
* Implement proper error handling for GPU resource creation
* Consider using bytemuck for vertex data serialization
* Depth buffer format should be Depth24Plus for compatibility
* Start with simple forward rendering, no complex lighting yet
* Ensure render system can be called from game loop
* Follow the project's module structure (graphics module)