name: "Basic WebGPU Renderer System"
description: |

## Purpose
Implement a foundational WebGPU rendering system that integrates with the existing ECS to render 3D objects using the transform hierarchy and camera system.

## Core Principles
1. **Context is King**: Include ALL necessary documentation, examples, and caveats
2. **Validation Loops**: Provide executable tests/lints the AI can run and fix
3. **Information Dense**: Use keywords and patterns from the codebase
4. **Progressive Success**: Start simple, validate, then enhance
5. **Global rules**: Be sure to follow all rules in CLAUDE.md

---

## Goal
Create a basic but extensible WebGPU renderer that can draw 3D primitives (cube, sphere, plane) with proper transform and camera support. The system should integrate seamlessly with the existing ECS, using GlobalTransform for positioning and supporting a basic material system.

## Why
- **Core functionality**: Rendering is fundamental for any 3D application
- **Visualization**: Enables seeing the transform hierarchy in action
- **Foundation**: Establishes patterns for more complex rendering features
- **Integration test**: Validates the ECS and transform system work correctly

## What
User-visible behavior:
- Render 3D primitives with proper perspective projection
- Camera controls view with transform hierarchy integration
- Basic colored materials (no textures yet)
- Depth testing for proper occlusion
- Window resizing handled gracefully
- 60 FPS target with vsync

### Success Criteria
- [ ] Camera component with projection matrix calculation
- [ ] Mesh component with primitive generation (cube, sphere, plane)
- [ ] Basic vertex/fragment shaders in WGSL
- [ ] Render context managing wgpu resources
- [ ] Uniform buffers for camera and object transforms
- [ ] Render system integrating with ECS queries
- [ ] Depth buffer with proper testing
- [ ] Window event loop in game main
- [ ] All tests pass and no clippy warnings

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/
  why: Surface and device creation patterns, swap chain configuration
  section: Focus on device/queue creation and surface configuration
  
- url: https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/
  why: Render pipeline creation, shader module setup
  section: Pipeline layout and vertex buffer descriptions
  
- url: https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/
  why: Uniform buffer patterns for camera and transforms
  section: Bind group layouts and buffer updates
  
- url: https://sotrh.github.io/learn-wgpu/beginner/tutorial8-depth/
  why: Depth buffer setup and configuration
  critical: Use Depth24Plus format for compatibility
  
- url: https://www.w3.org/TR/WGSL/
  why: WGSL shader syntax reference
  section: Vertex and fragment shader entry points
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/CLAUDE.md
  why: Project conventions - logging, module structure, no println!
  
- file: /mnt/c/Users/elias/RustroverProjects/webgpu-template/engine/src/core/entity/components.rs
  why: Component patterns - derives, methods, tests
  
- docfile: https://docs.rs/wgpu/latest/wgpu/
  why: wgpu API reference for device, queue, surface

- pattern: bytemuck for vertex data
  why: Zero-copy vertex buffer uploads, already enabled in glam
```

### Current Codebase tree
```bash
webgpu-template/
├── engine/
│   ├── Cargo.toml (has wgpu, winit, glam with bytemuck)
│   └── src/
│       ├── core/
│       │   ├── camera.rs (placeholder, needs implementation)
│       │   └── entity/
│       │       ├── components.rs (Transform, GlobalTransform)
│       │       ├── hierarchy.rs (update system)
│       │       └── world.rs (World wrapper)
│       ├── graphics/
│       │   └── mod.rs (empty, implement renderer here)
│       ├── shaders/
│       │   └── mod.rs (empty, shaders go here)
│       └── lib.rs
├── game/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs (placeholder, needs winit loop)
└── justfile (has preflight command)
```

### Desired Codebase tree with files to be added
```bash
webgpu-template/
└── engine/
    ├── Cargo.toml (UPDATE: add bytemuck = "1.23")
    └── src/
        ├── core/
        │   └── camera.rs (UPDATE: implement Camera component)
        ├── graphics/
        │   ├── mod.rs (UPDATE: declare submodules and exports)
        │   ├── context.rs (CREATE: RenderContext with device/queue)
        │   ├── mesh.rs (CREATE: Mesh component and primitives)
        │   ├── material.rs (CREATE: Material component)
        │   ├── pipeline.rs (CREATE: RenderPipeline abstraction)
        │   ├── renderer.rs (CREATE: main Renderer struct)
        │   └── uniform.rs (CREATE: uniform buffer types)
        └── shaders/
            ├── mod.rs (UPDATE: shader loading)
            └── basic.wgsl (CREATE: vertex/fragment shaders)
└── game/
    └── src/
        └── main.rs (UPDATE: winit event loop and rendering)
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: wgpu surface must be created after window
// Window must outlive surface!
let surface = unsafe { instance.create_surface(&window) }.unwrap();

// CRITICAL: Depth buffer format - use Depth24Plus for compatibility
depth_format: wgpu::TextureFormat::Depth24Plus,

// CRITICAL: Matrix order - glam is column-major
// MVP = Projection * View * Model (right-to-left application)
let mvp = camera_proj * camera_view * object_transform;

// CRITICAL: bytemuck requires repr(C) and Pod/Zeroable
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

// CRITICAL: Lifetime management
// RenderContext must own device/queue, Renderer borrows them

// CRITICAL: No println! - use tracing
use tracing::{debug, error, info, warn};

// CRITICAL: Handle minimize - swap chain needs recreation
if new_size.width > 0 && new_size.height > 0 {
    context.resize(new_size);
}
```

## Implementation Blueprint

### Data models and structure

```rust
// engine/src/core/camera.rs
use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
    pub fov_y_radians: f32,
    pub aspect_ratio: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub projection_mode: ProjectionMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProjectionMode {
    Perspective,
    Orthographic { height: f32 },
}

impl Camera {
    pub fn perspective(fov_y_degrees: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            fov_y_radians: fov_y_degrees.to_radians(),
            aspect_ratio,
            z_near,
            z_far,
            projection_mode: ProjectionMode::Perspective,
        }
    }
    
    pub fn projection_matrix(&self) -> Mat4 {
        match self.projection_mode {
            ProjectionMode::Perspective => {
                Mat4::perspective_rh(self.fov_y_radians, self.aspect_ratio, self.z_near, self.z_far)
            }
            ProjectionMode::Orthographic { height } => {
                let half_height = height * 0.5;
                let half_width = half_height * self.aspect_ratio;
                Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, self.z_near, self.z_far)
            }
        }
    }
    
    pub fn view_matrix(camera_transform: &GlobalTransform) -> Mat4 {
        camera_transform.matrix.inverse()
    }
}

// engine/src/graphics/mesh.rs
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Mesh {
    pub fn cube(size: f32) -> Self {
        // Generate cube vertices and indices
    }
}

// engine/src/graphics/material.rs
#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: [f32; 4], // RGBA
}

impl Default for Material {
    fn default() -> Self {
        Self { color: [1.0, 1.0, 1.0, 1.0] }
    }
}
```

### List of tasks to complete in order

```yaml
Task 1: Add bytemuck dependency
MODIFY engine/Cargo.toml:
  - ADD bytemuck = "1.23"
  - VERIFY wgpu and winit versions are compatible

Task 2: Implement Camera component
UPDATE engine/src/core/camera.rs:
  - IMPLEMENT Camera struct with projection parameters
  - ADD perspective and orthographic projection methods
  - ADD view_matrix static method using GlobalTransform
  - DERIVE appropriate traits (Debug, Clone, Copy, Serialize, Deserialize)
  - ADD unit tests for projection matrices

Task 3: Create Vertex and Mesh types
CREATE engine/src/graphics/mesh.rs:
  - DEFINE Vertex struct with bytemuck derives
  - IMPLEMENT Mesh component with vertices and indices
  - ADD primitive generators: cube, sphere, plane
  - IMPLEMENT vertex attribute descriptors
  - ADD tests for primitive generation

Task 4: Create Material component
CREATE engine/src/graphics/material.rs:
  - DEFINE Material struct with color
  - IMPLEMENT Default with white color
  - PREPARE for future texture support

Task 5: Write WGSL shaders
CREATE engine/src/shaders/basic.wgsl:
  - VERTEX shader with MVP transform
  - FRAGMENT shader with basic color output
  - DEFINE uniform buffer layouts
UPDATE engine/src/shaders/mod.rs:
  - ADD shader loading as &'static str

Task 6: Implement RenderContext
CREATE engine/src/graphics/context.rs:
  - CREATE RenderContext managing device, queue, surface
  - IMPLEMENT initialization from window
  - ADD swap chain configuration
  - HANDLE resize and recreation
  - IMPLEMENT resource cleanup

Task 7: Create uniform buffer management
CREATE engine/src/graphics/uniform.rs:
  - DEFINE CameraUniform struct (view-proj matrix)
  - DEFINE ObjectUniform struct (model matrix)
  - IMPLEMENT buffer creation and update methods
  - ADD bind group layout utilities

Task 8: Build render pipeline abstraction
CREATE engine/src/graphics/pipeline.rs:
  - CREATE RenderPipeline wrapper
  - IMPLEMENT pipeline creation with layout
  - ADD vertex buffer layout from Vertex type
  - CONFIGURE depth testing

Task 9: Implement main Renderer
CREATE engine/src/graphics/renderer.rs:
  - CREATE Renderer struct owning resources
  - IMPLEMENT render method querying ECS
  - ADD uniform buffer updates per frame
  - MANAGE vertex/index buffers per mesh
  - IMPLEMENT proper render pass encoding

Task 10: Update graphics module exports
MODIFY engine/src/graphics/mod.rs:
  - DECLARE all submodules
  - RE-EXPORT public types
  - ADD render_system function for ECS

Task 11: Update engine prelude
MODIFY engine/src/lib.rs:
  - RE-EXPORT Camera, Mesh, Material in prelude
  - ENSURE existing exports remain

Task 12: Implement game main with rendering
UPDATE game/src/main.rs:
  - CREATE winit event loop and window
  - INITIALIZE engine and renderer
  - SPAWN test scene (camera + cube)
  - IMPLEMENT render loop
  - HANDLE window events and resize

Task 13: Add comprehensive tests
ADD tests to each module:
  - TEST mesh primitive vertex counts
  - TEST camera projection matrices
  - TEST uniform buffer sizes
  - INTEGRATION test of full render
```

### Per task pseudocode

```rust
// Task 9: Renderer implementation pseudocode
pub struct Renderer {
    pipeline: RenderPipeline,
    depth_texture: wgpu::Texture,
    camera_bind_group: wgpu::BindGroup,
    // Mesh cache
    mesh_buffers: HashMap<Entity, MeshBuffers>,
}

impl Renderer {
    pub fn render(&mut self, 
        world: &World, 
        camera_entity: Entity,
        context: &RenderContext,
    ) -> Result<(), wgpu::SurfaceError> {
        // Get camera matrices
        let camera = world.get::<Camera>(camera_entity)?;
        let camera_transform = world.get::<GlobalTransform>(camera_entity)?;
        let view_proj = camera.projection_matrix() * Camera::view_matrix(&camera_transform);
        
        // Update camera uniform
        context.queue.write_buffer(&self.camera_buffer, 0, 
            bytemuck::cast_slice(&[view_proj]));
        
        // Get next frame
        let output = context.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        
        // Create command encoder
        let mut encoder = context.device.create_command_encoder(&Default::default());
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                // Configure with color and depth attachments
            });
            
            render_pass.set_pipeline(&self.pipeline);
            
            // Query and render all entities with Mesh + GlobalTransform
            for (entity, (mesh, transform, material)) in 
                world.query::<(&Mesh, &GlobalTransform, Option<&Material>)>().iter() 
            {
                // Update or create mesh buffers
                let buffers = self.get_or_create_mesh_buffers(entity, mesh, &context.device);
                
                // Set object uniform
                let model_matrix = transform.matrix;
                
                // Draw
                render_pass.set_vertex_buffer(0, buffers.vertex.slice(..));
                render_pass.set_index_buffer(buffers.index.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
            }
        }
        
        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

// Task 12: Game main pseudocode
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    engine::init_logging();
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WebGPU Template")
        .build(&event_loop)
        .unwrap();
    
    // Initialize renderer
    let mut context = pollster::block_on(RenderContext::new(&window));
    let mut renderer = Renderer::new(&context);
    
    // Create world and spawn test scene
    let mut world = World::new();
    
    // Spawn camera
    let camera = world.spawn((
        Transform::from_position(Vec3::new(0.0, 5.0, 10.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        Camera::perspective(60.0, window.inner_size().width as f32 / window.inner_size().height as f32, 0.1, 1000.0),
    ));
    
    // Spawn cube
    world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Mesh::cube(2.0),
        Material::default(),
    ));
    
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                context.resize(size);
                // Update camera aspect ratio
            }
            Event::RedrawRequested(_) => {
                // Update systems
                update_hierarchy_system(&mut world);
                
                // Render
                match renderer.render(&world, camera, &context) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => context.reconfigure(),
                    Err(e) => eprintln!("Render error: {:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
```

### Integration Points
```yaml
DEPENDENCIES:
  - add to: engine/Cargo.toml
  - bytemuck = "1.23"
  - pollster = "0.3" (for game/Cargo.toml)
  
MODULE STRUCTURE:
  - create: engine/src/graphics/ submodules
  - update: engine/src/core/camera.rs
  - update: engine/src/shaders/mod.rs
  - update: engine/src/lib.rs prelude exports
  
ECS INTEGRATION:
  - Renderer queries for Mesh + GlobalTransform + Material
  - Camera entity provides view matrix
  - Works with existing transform hierarchy
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all
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
    fn test_camera_perspective_projection() {
        let camera = Camera::perspective(60.0, 16.0/9.0, 0.1, 1000.0);
        let proj = camera.projection_matrix();
        // Verify projection properties
        assert!(proj.w_axis.w == 0.0); // Perspective has w=0
    }
    
    #[test]
    fn test_mesh_cube_vertices() {
        let cube = Mesh::cube(1.0);
        assert_eq!(cube.vertices.len(), 24); // 6 faces * 4 vertices
        assert_eq!(cube.indices.len(), 36); // 6 faces * 2 triangles * 3 indices
    }
    
    #[test]
    fn test_vertex_size() {
        use std::mem;
        // Ensure vertex is tightly packed for GPU
        assert_eq!(mem::size_of::<Vertex>(), 32); // 8 floats * 4 bytes
    }
}
```

```bash
# Run tests
cargo test --workspace
# Expected: All tests pass
```

### Level 3: Integration Test
```rust
// In game/tests/render_test.rs
#[test]
fn test_basic_render_setup() {
    use engine::prelude::*;
    
    // Create world with camera and mesh
    let mut world = World::new();
    
    let camera = world.spawn((
        Transform::from_position(Vec3::new(0.0, 0.0, 5.0)),
        GlobalTransform::default(),
        Camera::perspective(60.0, 1.0, 0.1, 100.0),
    ));
    
    let cube = world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Mesh::cube(1.0),
        Material::default(),
    ));
    
    // Update hierarchy
    update_hierarchy_system(&mut world);
    
    // Verify components exist
    assert!(world.get::<Camera>(camera).is_ok());
    assert!(world.get::<Mesh>(cube).is_ok());
}
```

### Level 4: Visual Validation
```bash
# Run the game to see a white cube
cargo run

# Expected: Window opens showing a white cube on dark background
# Cube should be visible with proper perspective
# Window resizing should maintain aspect ratio
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No linting errors: `just preflight`
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] Basic cube renders correctly
- [ ] Window resizing works without crashes
- [ ] Depth testing works (overlapping objects)
- [ ] No use of println! - only tracing macros
- [ ] All GPU resources properly cleaned up on exit

---

## Anti-Patterns to Avoid
- ❌ Don't create the surface before the window
- ❌ Don't forget to handle swap chain recreation
- ❌ Don't use println! for debugging - use tracing
- ❌ Don't hardcode window size in camera aspect ratio
- ❌ Don't skip depth buffer - it's essential
- ❌ Don't leak GPU resources - implement Drop
- ❌ Don't block the event loop with heavy computation

## Confidence Score: 8/10

The PRP provides comprehensive context including:
- All necessary wgpu documentation and tutorials
- Complete implementation blueprint with proper structure
- Specific gotchas around lifetime and matrix math
- Clear task ordering to minimize dependencies
- Executable validation commands and tests

The main uncertainties are:
- Camera component design (created based on PLANNING.md example)
- Exact mesh primitive implementations (standard approaches exist)

The extensive wgpu documentation and clear patterns make this highly implementable.