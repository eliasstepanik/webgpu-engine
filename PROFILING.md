# Profiling Guide

This guide explains how to use the Tracy profiler integration to analyze performance of the WebGPU engine.

## Overview

The engine includes optional Tracy profiler integration for real-time performance analysis. Tracy provides:
- Frame timing visualization
- Hierarchical profiling zones
- CPU timing for all major systems
- Memory allocation tracking (optional)
- GPU profiling on supported platforms

## Prerequisites

1. **Download Tracy Profiler v0.11.1**
   - Download from: https://github.com/wolfpld/tracy/releases/tag/v0.11.1
   - **Important**: Use version 0.11.1 for compatibility with tracy-client 0.17.5
   - Extract and run the Tracy profiler UI

2. **Build with Tracy Feature**
   ```bash
   # Build with tracy support
   just build-tracy
   
   # Or manually:
   cargo build -p game --features "editor tracy"
   ```

## Running with Tracy

1. **Start Tracy UI First**
   - Launch the Tracy profiler application
   - Keep it running in the background

2. **Run the Game with Tracy**
   ```bash
   just run-tracy
   
   # Or manually:
   cargo run -p game --features "editor tracy"
   ```

3. **Connect Tracy to the Game**
   - In Tracy UI, click "Connect" → "localhost"
   - You should see "Connection established"
   - Frame timing graph will appear
   - Profiling zones will be visible in the timeline

## Profiling Zones

The engine includes profiling zones for:

- **Frame Boundaries**: Marked after each frame presentation
- **Renderer**: 
  - `Renderer::render` - Main render function
  - `Update camera uniforms` - Camera matrix updates
  - `Main render pass` - GPU render pass
  - `Collect entities` - Entity gathering for rendering
  - `Draw calls` - Actual GPU draw operations
- **Scripting System**:
  - `ScriptSystem::update` - Main script execution
  - `Script::on_start` - Script initialization
  - `Script::on_update` - Per-frame script updates
- **Asset Loading**:
  - `Scene::load` - Scene file loading
  - `Scene::instantiate` - Scene instantiation into world

## Memory Profiling (Optional)

Memory allocation tracking has significant overhead (5-10%) and is disabled by default.

To enable memory profiling:
```bash
TRACY_MEMORY=1 cargo run -p game --features tracy
```

This will track all memory allocations and show them in Tracy's memory view.

## GPU Profiling

GPU profiling requires hardware support for timestamp queries. The engine will automatically detect and enable GPU profiling if available.

**Platform Support**:
- ✅ Native platforms with Vulkan backend
- ✅ Some DirectX 12 configurations
- ⚠️ Limited Metal support (macOS)
- ❌ WebGPU/WebGL (not supported)

If GPU profiling is available, you'll see GPU timing information in Tracy's GPU timeline.

## Performance Impact

- **Tracy Disabled**: Zero overhead (all profiling code is compiled out)
- **Tracy Enabled but Not Connected**: <1% overhead
- **Tracy Connected**: 1-5% overhead depending on zone density
- **Memory Tracking Enabled**: Additional 5-10% overhead

## Troubleshooting

### Tracy UI Won't Connect
- Ensure Tracy UI is running before starting the game
- Check firewall settings (Tracy uses TCP port 8086)
- Verify the game was built with `--features tracy`

### No Profiling Data Visible
- Confirm tracy feature is enabled: `cargo build -p game --features tracy`
- Check that frame marking is happening (frame counter should increment)
- Try restarting both Tracy UI and the game

### GPU Profiling Not Working
- Check the console for "GPU profiling enabled with Tracy" message
- If you see "GPU timestamp queries not supported", your hardware/driver doesn't support it
- Try updating graphics drivers
- Use native build instead of web target

## Best Practices

1. **Profile Release Builds**: Debug builds have different performance characteristics
   ```bash
   cargo run -p game --release --features tracy
   ```

2. **Focus on Hot Paths**: Look for functions called frequently or taking significant time

3. **Compare With/Without Tracy**: Measure the profiling overhead in your specific use case

4. **Use Zones Sparingly**: Don't add profiling to very tight loops (per-vertex operations)

## Adding Custom Profiling Zones

To add profiling to your own code:

```rust
use crate::profile_zone;

fn my_function() {
    profile_zone!("MyFunction");
    
    // Your code here
}
```

For scoped profiling:
```rust
{
    profile_zone!("Expensive Operation");
    // Code to profile
} // Zone ends here
```

## Further Reading

- Tracy Documentation: https://github.com/wolfpld/tracy
- tracy-client Rust Docs: https://docs.rs/tracy-client/
- wgpu-profiler Docs: https://docs.rs/wgpu-profiler/