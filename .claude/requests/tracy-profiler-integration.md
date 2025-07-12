## FEATURE:

Integrate Tracy profiler 0.12.2 into the WebGPU template engine for real-time performance profiling and analysis. This includes:

1. **Tracy Client Integration**: Add tracy-client 0.12.2 dependency with proper feature flags
2. **Profiling Zones**: Instrument critical code paths with Tracy zones (rendering, ECS updates, asset loading)
3. **Frame Marking**: Mark frame boundaries for accurate frame time analysis
4. **GPU Profiling**: Integrate with WebGPU for GPU timing information
5. **Memory Tracking**: Monitor memory allocations and deallocations
6. **Conditional Compilation**: Make Tracy integration optional via Cargo features for release builds
7. **Custom Plots**: Add custom value plotting for engine metrics (entity count, draw calls, etc.)

## EXAMPLES:

Example Tracy integration patterns:
```rust
// Frame marking in main loop
tracy_client::frame_mark();

// Zone profiling for render functions
pub fn render(&mut self, world: &World, surface: &wgpu::Surface) -> Result<(), wgpu::SurfaceError> {
    let _zone = tracy_client::span!("Renderer::render");
    // ... existing render code
}

// GPU profiling for WebGPU operations
let mut encoder = {
    let _zone = tracy_client::span!("Create Command Encoder");
    self.context.create_command_encoder(Some("Render Encoder"))
};

// Memory allocation tracking
let vertex_buffer = {
    let _zone = tracy_client::span!("Create Vertex Buffer");
    self.context.device.create_buffer_init(&descriptor)
};

// Custom value plotting
tracy_client::plot!("Entity Count", world.len() as f64);
tracy_client::plot!("Draw Calls", draw_call_count as f64);
```

Example feature configuration:
```toml
# Cargo.toml
[features]
default = []
profiling = ["tracy-client"]

[dependencies]
tracy-client = { version = "0.12.2", optional = true }
```

## DOCUMENTATION:

1. **Tracy Profiler Documentation**: https://github.com/wolfpld/tracy
2. **tracy-client Rust Crate**: https://docs.rs/tracy-client/0.12.2/tracy_client/
3. **Tracy User Manual**: https://github.com/wolfpld/tracy/releases (PDF documentation)
4. **WebGPU Performance Guidelines**: https://toji.dev/webgpu-best-practices/profiling.html
5. **Current Renderer Implementation**: `engine/src/graphics/renderer.rs`
6. **ECS Update Loop**: `game/src/main.rs` and engine core modules
7. **Asset Loading**: `engine/src/io/` and `engine/src/graphics/asset_manager.rs`

## OTHER CONSIDERATIONS:

1. **Conditional Compilation**: Tracy should be completely compiled out in release builds by default unless explicitly enabled. Use `#[cfg(feature = "profiling")]` guards.

2. **Performance Impact**: Tracy client has minimal overhead but should still be optional. Macro usage should have zero cost when disabled.

3. **Build Configuration**:
   - Tracy requires specific build flags for optimal performance
   - May need to link against Tracy static library on some platforms
   - Windows may require additional DLL considerations

4. **Integration Points**:
   - **Main Loop**: Frame marking in the event loop (`game/src/main.rs`)
   - **Renderer**: All major render functions need profiling zones
   - **ECS Systems**: Transform hierarchy updates, component queries
   - **Asset Loading**: File I/O operations and mesh upload
   - **Hot-Reload**: File watching and recompilation events

5. **Existing Tracing Integration**: 
   - The project already uses the `tracing` crate for logging
   - Tracy integration should complement, not replace, existing logging
   - Consider using Tracy's message system for important events

6. **WebGPU Specifics**:
   - GPU profiling requires careful integration with command buffer submission
   - May need timestamp queries for accurate GPU timing
   - Ensure compatibility with different WebGPU backends

7. **Memory Profiling**:
   - Track major allocations like vertex buffers, textures, uniform buffers
   - Monitor ECS world memory usage
   - Track asset loading memory patterns

8. **Custom Metrics**:
   - Entity count per frame
   - Number of draw calls
   - Mesh cache hit rate
   - Shader compilation times (if Slang integration exists)
   - File watcher events

9. **Platform Considerations**:
   - Tracy server needs to be available for profiling sessions
   - Network connectivity for Tracy client-server communication
   - Consider offline profiling mode for restricted environments

10. **Development Workflow**:
    - Document how to use Tracy profiler with the engine
    - Provide example Tracy server configuration
    - Include profiling best practices in documentation

11. **Error Handling**:
    - Tracy client failures should not crash the application
    - Graceful degradation when Tracy server is unavailable
    - Proper cleanup of profiling resources

12. **Feature Flags Strategy**:
    - `profiling`: Enables Tracy client
    - Consider sub-features like `gpu-profiling`, `memory-profiling`
    - Ensure zero overhead when features are disabled