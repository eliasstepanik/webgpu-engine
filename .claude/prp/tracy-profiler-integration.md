# Tracy Profiler Integration - Project Request Plan (PRP)

**Confidence Score: 9/10** - High confidence due to clear codebase patterns, comprehensive documentation, and well-defined integration points.

## Executive Summary

Integrate Tracy profiler 0.12.2 into the WebGPU template engine for real-time performance profiling. This includes CPU profiling zones, GPU timing, memory tracking, and custom metrics - all behind optional feature flags to maintain zero overhead in production builds.

## Context & Requirements

### Objective
Add comprehensive performance profiling capabilities to the engine using Tracy profiler, enabling developers to:
- Profile CPU execution with hierarchical zones
- Track GPU command execution timing  
- Monitor memory allocations
- Visualize custom engine metrics (entity count, draw calls, etc.)
- Maintain zero overhead when profiling is disabled

### Key Integration Points (from codebase analysis)
1. **Logging System**: `engine/src/lib.rs:59` - `init_logging()` already initializes tracing
2. **Main Render Loop**: `game/src/main.rs:360` - Frame timing and system updates
3. **Renderer**: `engine/src/graphics/renderer.rs` - Critical render paths
4. **ECS Systems**: Transform updates, script execution, physics simulation
5. **Asset Loading**: `engine/src/graphics/asset_manager.rs` and hot reload system

### External Documentation
- Tracy Documentation: https://github.com/wolfpld/tracy
- tracy-client Rust crate: https://docs.rs/tracy-client/0.12.2/tracy_client/
- Tracy User Manual: https://github.com/wolfpld/tracy/releases
- wgpu-profiler crate: https://docs.rs/wgpu-profiler/latest/wgpu_profiler/
- Example implementation: https://github.com/abhirag/tracy_rust_demo

## Implementation Blueprint

### Phase 1: Core Tracy Integration

#### 1.1 Add Dependencies
```toml
# engine/Cargo.toml
[features]
default = []
profiling = ["tracy-client", "wgpu-profiler"]

[dependencies]
tracy-client = { version = "0.12.2", optional = true, features = ["enable"] }
wgpu-profiler = { version = "0.20.0", optional = true }

# game/Cargo.toml  
[features]
default = ["editor"]
editor = ["dep:editor"]
profiling = ["engine/profiling"]
```

#### 1.2 Create Profiling Module
Create `engine/src/profiling/mod.rs`:
```rust
#[cfg(feature = "profiling")]
pub use tracy_client::{Client, frame_mark, plot, span};

#[cfg(not(feature = "profiling"))]
pub mod tracy_client {
    // No-op macros when profiling disabled
    macro_rules! span {
        ($name:expr) => { () };
    }
    pub(crate) use span;
    
    pub fn frame_mark() {}
    pub fn plot(_: &str, _: f64) {}
}

pub fn init_profiling() {
    #[cfg(feature = "profiling")]
    {
        // Tracy client auto-initializes on first use
        tracing::info!("Tracy profiler enabled");
    }
}
```

#### 1.3 Modify Logging Initialization
Update `engine/src/lib.rs`:
```rust
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    init_logging()?;
    profiling::init_profiling();
    Ok(())
}
```

### Phase 2: CPU Profiling Zones

#### 2.1 Main Loop Profiling
Update `game/src/main.rs` render loop:
```rust
use engine::profiling;

// In window_event RedrawRequested:
profiling::frame_mark(); // Mark frame boundary

{
    let _zone = profiling::span!("Frame Update");
    
    // Script initialization
    {
        let _zone = profiling::span!("Script Init System");
        engine::scripting::script_init_system(&mut world);
    }
    
    // Script execution
    {
        let _zone = profiling::span!("Script Execution");
        engine::scripting::script_execution_system(&mut world, dt);
    }
    
    // Physics
    {
        let _zone = profiling::span!("Physics Update");
        engine::physics::update_physics_system(&mut world, dt);
    }
    
    // Hierarchy
    {
        let _zone = profiling::span!("Hierarchy Update");
        engine::update_hierarchy_system(&mut world);
    }
}

// Custom metrics
profiling::plot!("Entity Count", world.len() as f64);
profiling::plot!("Frame Time", dt as f64 * 1000.0); // ms
```

#### 2.2 Renderer Profiling
Update `engine/src/graphics/renderer.rs`:
```rust
use crate::profiling;

pub fn render(&mut self, world: &World, surface: &wgpu::Surface) -> Result<(), wgpu::SurfaceError> {
    let _zone = profiling::span!("Renderer::render");
    
    // Get surface texture
    let output = {
        let _zone = profiling::span!("Surface::get_current_texture");
        surface.get_current_texture()?
    };
    
    // Create command encoder
    let mut encoder = {
        let _zone = profiling::span!("Create Command Encoder");
        self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    };
    
    // Render world
    {
        let _zone = profiling::span!("Render World");
        self.render_world(world, &mut encoder, &output.texture.create_view(&Default::default()))?;
    }
    
    // Submit
    {
        let _zone = profiling::span!("Queue Submit");
        self.context.queue.submit(std::iter::once(encoder.finish()));
    }
    
    output.present();
    Ok(())
}
```

### Phase 3: GPU Profiling

#### 3.1 Create GPU Profiler
Add to `engine/src/graphics/renderer.rs`:
```rust
#[cfg(feature = "profiling")]
use wgpu_profiler::GpuProfiler;

pub struct Renderer {
    // ... existing fields
    #[cfg(feature = "profiling")]
    gpu_profiler: GpuProfiler,
}

impl Renderer {
    pub fn new() -> Self {
        // ... existing initialization
        
        #[cfg(feature = "profiling")]
        let gpu_profiler = GpuProfiler::new_with_tracy_client(&device, &queue);
        
        Self {
            // ... existing fields
            #[cfg(feature = "profiling")]
            gpu_profiler,
        }
    }
}
```

#### 3.2 Profile GPU Operations
```rust
pub fn render_world(&mut self, world: &World, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) -> Result<()> {
    #[cfg(feature = "profiling")]
    let mut profiler_scope = self.gpu_profiler.scope("render_world", encoder);
    #[cfg(not(feature = "profiling"))]
    let encoder = encoder;
    
    {
        #[cfg(feature = "profiling")]
        let mut render_pass_scope = profiler_scope.scoped_render_pass("main_render_pass", &render_pass_descriptor);
        #[cfg(not(feature = "profiling"))]
        let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
        
        // Render operations...
    }
    
    #[cfg(feature = "profiling")]
    self.gpu_profiler.resolve_queries(encoder);
    
    Ok(())
}

// In render() method after queue.submit():
#[cfg(feature = "profiling")]
self.gpu_profiler.end_frame().unwrap();
```

### Phase 4: Memory Tracking

#### 4.1 Track Major Allocations
Update mesh/buffer creation:
```rust
// In create_vertex_buffer:
let buffer = {
    let _zone = profiling::span!("Create Vertex Buffer");
    let size = vertices.len() * std::mem::size_of::<Vertex>();
    profiling::plot!("GPU Memory (MB)", (size as f64) / (1024.0 * 1024.0));
    
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
};
```

### Phase 5: Asset Loading Profiling

Update `engine/src/graphics/asset_manager.rs`:
```rust
pub fn load_mesh(&mut self, path: &Path) -> Result<MeshHandle> {
    let _zone = profiling::span!("AssetManager::load_mesh");
    profiling::plot!("Mesh Cache Size", self.meshes.len() as f64);
    
    // Existing loading logic...
}
```

## Testing & Validation

### Build Commands
```bash
# Build with profiling
cargo build --features profiling

# Build without profiling (production)
cargo build --release

# Run with profiling
cargo run --features profiling

# Verify zero overhead when disabled
cargo bloat --release --filter engine
```

### Tracy Server Setup
1. Download Tracy profiler from: https://github.com/wolfpld/tracy/releases
2. Run Tracy profiler GUI
3. Start the game with profiling enabled
4. Tracy will auto-connect and display profiling data

### Test Scenarios
1. **Frame Time Analysis**: Verify frame marks show accurate frame timing
2. **CPU Zones**: Check hierarchical zone display in Tracy
3. **GPU Timeline**: Confirm GPU operations appear with correct timing
4. **Memory Tracking**: Monitor allocation plots over time
5. **Custom Metrics**: Verify entity count and draw call plots

## Implementation Checklist

- [ ] Add tracy-client and wgpu-profiler dependencies with feature flags
- [ ] Create profiling module with conditional compilation
- [ ] Integrate with existing logging initialization
- [ ] Add frame marks to main render loop
- [ ] Instrument all ECS systems with profiling zones
- [ ] Add GPU profiling to render passes
- [ ] Track memory allocations for buffers/textures
- [ ] Add custom metric plots (entity count, draw calls, etc.)
- [ ] Test with and without profiling feature enabled
- [ ] Document Tracy server setup in README
- [ ] Add profiling best practices to developer guide

## Potential Issues & Solutions

1. **GPU Timestamp Queries**: Not supported on all platforms
   - Solution: Gracefully disable GPU profiling if not available
   
2. **Tracy Version Mismatch**: Client/server protocol compatibility
   - Solution: Document required Tracy server version (0.12.x)
   
3. **Performance Overhead**: Even with conditional compilation
   - Solution: Use `#[inline(always)]` on no-op stubs
   
4. **Thread Safety**: Tracy requires per-thread contexts
   - Solution: Use thread-local storage for multi-threaded systems

## Success Criteria

1. Tracy profiler connects and displays data when feature enabled
2. Zero runtime overhead when profiling feature disabled  
3. All major systems have profiling coverage
4. GPU timeline shows render pass timing
5. Memory usage tracked and visualized
6. Custom engine metrics visible in Tracy
7. `cargo test --workspace` passes with and without profiling
8. Documentation updated with profiling guide

## Additional Resources

- Tracy C API documentation: https://github.com/wolfpld/tracy/blob/master/public/tracy/TracyC.h
- Rust profiling best practices: https://nnethercote.github.io/perf-book/profiling.html
- wgpu timestamp queries: https://github.com/gfx-rs/wgpu/issues/721

---

This PRP provides complete context for implementing Tracy profiler integration with clear, actionable steps and validation criteria. The implementation follows existing codebase patterns and maintains zero overhead through careful use of conditional compilation.