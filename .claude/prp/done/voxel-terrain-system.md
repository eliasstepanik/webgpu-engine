name: "Voxel Terrain System with Sparse Octree Streaming"
description: |

## Purpose
Implement a high-performance voxel terrain component for the WebGPU engine using sparse voxel octrees (64-trees) with disk streaming, raymarching renderer, LOD support, and movable terrain sections with interpolation. The system will integrate with the engine's large world coordinate system to support massive planetary terrains and asteroids.

## Core Principles
1. **Performance First**: Use 64-trees and GPU-optimized data structures
2. **Memory Efficient**: Stream from disk with configurable memory budget
3. **Large World Ready**: Full integration with f64 coordinate system
4. **Smooth Movement**: Interpolated motion for planets and asteroids
5. **Engine Integration**: Follow established patterns and conventions

---

## Goal
Create a terrain module that enables rendering of massive voxel worlds (planet-scale and beyond) with smooth real-time movement, efficient streaming from disk, and LOD-based rendering. The system should support both static terrain and moving celestial bodies with seamless interpolation.

## Why
- **Planetary Scale Worlds**: Enable creation of full-scale planets with detailed voxel terrain
- **Dynamic Environments**: Support moving asteroids, rotating planets, orbital mechanics
- **Memory Efficiency**: Stream multi-terabyte worlds with limited RAM
- **Performance**: Achieve 60+ FPS with massive view distances using LOD
- **Engine Integration**: Provide terrain as a first-class engine component

## What
A complete voxel terrain system that:
- Renders voxel terrain using GPU raymarching in fragment shaders
- Streams terrain data from disk with LRU caching
- Supports 5 LOD levels for massive view distances
- Enables smooth movement of entire terrain sections (planets, asteroids)
- Integrates with the engine's ECS and large world coordinates

### Success Criteria
- [ ] Render voxel terrain at 60+ FPS with 1000km view distance
- [ ] Stream terrain data with <100ms latency
- [ ] Support terrain sections moving at any speed with smooth interpolation
- [ ] Memory usage stays within configured budget (default 4GB)
- [ ] Integrate seamlessly with existing Transform/WorldTransform components
- [ ] All tests pass and documentation is complete

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/
  why: Complete guide on 64-tree implementation and GPU traversal algorithms
  
- url: https://github.com/expenses/tree64
  why: Reference implementation of sparse 64-trees with benchmarks
  
- url: https://github.com/davids91/shocovox
  why: WGSL sparse voxel octree implementation to reference for shader code
  
- url: https://www.w3.org/TR/WGSL/
  why: WGSL specification for shader syntax and storage buffer usage
  
- file: engine/src/graphics/renderer.rs
  why: Understand how to integrate new rendering pipelines
  
- file: engine/src/core/coordinates/world_transform.rs
  why: Large world coordinate system integration
  
- file: engine/src/graphics/pipeline.rs
  why: Pattern for creating render pipelines
  
- file: engine/src/shaders/logarithmic_depth.wgsl
  why: Reference for shader uniform structure and logarithmic depth
  
- file: engine/src/core/entity/components.rs
  why: Component definition patterns
  
- file: CLAUDE.md
  why: Project conventions and rules
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── lib.rs              # Module declarations
│   ├── core/               # Core systems
│   │   ├── entity/         # ECS components
│   │   └── coordinates/    # Large world coords
│   ├── graphics/           # Rendering systems
│   │   ├── pipeline.rs     # Render pipeline creation
│   │   └── renderer.rs     # Main renderer
│   ├── shaders/            # WGSL shaders
│   │   ├── mod.rs          # Shader constants
│   │   └── *.wgsl          # Shader files
│   └── io/                 # Serialization
│       └── component_registry.rs
```

### Desired Codebase Structure
```bash
engine/
├── src/
│   ├── lib.rs              # Add: pub mod terrain;
│   └── terrain/            # NEW MODULE
│       ├── mod.rs          # Module exports
│       ├── octree.rs       # 64-tree data structure
│       ├── streaming.rs    # Disk streaming system
│       ├── components.rs   # VoxelTerrain, TerrainMotion
│       ├── pipeline.rs     # Terrain render pipeline
│       ├── lod.rs          # LOD selection logic
│       └── interpolation.rs # Movement interpolation
├── shaders/
│   ├── terrain_raymarch.wgsl # Voxel raymarching shader
│   └── mod.rs              # Add terrain shader constant
```

### Known Gotchas & Conventions
```rust
// CRITICAL: No compute shaders available - use fragment shader raymarching
// The engine doesn't enable compute features in WebGPU device creation

// PATTERN: All components are simple structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelTerrain {
    pub octree_data: Arc<TerrainData>,
    pub lod_bias: f32,
}

// GOTCHA: Storage buffer size limited to 2GB in WebGPU
// Must split large octrees across multiple buffers

// PATTERN: Use tracing for logging, never println!
use tracing::{debug, info, warn, error};

// CRITICAL: Camera-relative rendering is automatic
// Renderer converts f64 world positions to f32 camera-relative

// PATTERN: Shaders use standard bind group layout:
// Group 0: Camera uniforms
// Group 1: Object uniforms  
// Group 2+: Custom bindings (we'll use for octree data)
```

## Implementation Blueprint

### Data Models and Structures

```rust
// Core 64-tree node structure (GPU-friendly)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OctreeNode {
    pub child_mask: u64,      // Bitmask for 64 children
    pub child_pointer: u32,   // Offset to first child
    pub voxel_data: u32,      // Material/color data
}

// Terrain data container
pub struct TerrainData {
    pub nodes: Vec<OctreeNode>,
    pub gpu_buffer: Option<wgpu::Buffer>,
    pub dirty: bool,
}

// ECS Components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelTerrain {
    pub data_id: TerrainDataId,
    pub lod_bias: f32,
    pub cast_shadows: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainMotion {
    pub previous_transform: WorldTransform,
    pub velocity: DVec3,
    pub angular_velocity: Quat,
}
```

### Task List

```yaml
Task 1 - Create Terrain Module Structure:
CREATE engine/src/terrain/mod.rs:
  - Export public types
  - Follow pattern from engine/src/graphics/mod.rs
  
CREATE engine/src/terrain/octree.rs:
  - Define OctreeNode structure
  - Implement basic 64-tree operations
  - Add Morton encoding utilities

MODIFY engine/src/lib.rs:
  - ADD: pub mod terrain;
  - Follow existing module pattern

Task 2 - Implement Core Data Structures:
CREATE engine/src/terrain/components.rs:
  - Define VoxelTerrain component
  - Define TerrainMotion component
  - Add TerrainDataId type
  - Follow pattern from engine/src/core/entity/components.rs

Task 3 - Create Streaming System:
CREATE engine/src/terrain/streaming.rs:
  - Implement TerrainStreamer with memory-mapped files
  - Add LRU cache with configurable size
  - Create async loading with tokio
  - Add frustum-based chunk selection

Task 4 - Build Terrain Pipeline:
CREATE engine/src/terrain/pipeline.rs:
  - Create TerrainPipeline following RenderPipeline pattern
  - Add storage buffer for octree data
  - Configure bind groups (camera, object, terrain data)
  
CREATE engine/src/shaders/terrain_raymarch.wgsl:
  - Implement DDA ray traversal
  - Add hierarchical space skipping
  - Support logarithmic depth
  - Follow shader patterns from basic.wgsl

MODIFY engine/src/shaders/mod.rs:
  - ADD: pub const TERRAIN_SHADER: &str = include_str!("terrain_raymarch.wgsl");

Task 5 - Implement LOD System:
CREATE engine/src/terrain/lod.rs:
  - Define 5 LOD levels (64³ → 32³ → 16³ → 8³ → 4³)
  - Implement distance-based selection
  - Add smooth LOD transitions
  - Support LOD bias per terrain

Task 6 - Add Movement Interpolation:
CREATE engine/src/terrain/interpolation.rs:
  - Implement frame interpolation
  - Add velocity prediction
  - Support hierarchical movement
  - Handle large world coordinates

Task 7 - Integrate with Renderer:
MODIFY engine/src/graphics/renderer.rs:
  - Add terrain rendering method
  - Query VoxelTerrain components
  - Handle TerrainMotion interpolation
  - Follow existing rendering patterns

Task 8 - Add Tests:
CREATE engine/src/terrain/tests.rs:
  - Test 64-tree operations
  - Test Morton encoding
  - Test LRU cache eviction
  - Test LOD selection
  - Follow test patterns from coordinates module
```

### Task 1-2 Pseudocode
```rust
// Task 1: mod.rs structure
pub mod octree;
pub mod streaming;
pub mod components;
pub mod pipeline;
pub mod lod;
pub mod interpolation;

pub use octree::{OctreeNode, TerrainOctree};
pub use components::{VoxelTerrain, TerrainMotion};
pub use pipeline::TerrainPipeline;

// Task 2: 64-tree implementation
impl TerrainOctree {
    pub fn new(depth: u8) -> Self {
        // PATTERN: Use Vec for dynamic allocation
        Self {
            nodes: Vec::with_capacity(1000000),
            depth,
            node_size: 1.0, // meters
        }
    }
    
    pub fn insert_voxel(&mut self, pos: DVec3, data: u32) {
        // CRITICAL: Use Morton encoding for spatial indexing
        let morton = morton_encode_3d(pos);
        
        // GOTCHA: Check bounds to prevent overflow
        if !self.bounds.contains(pos) { return; }
        
        // Traverse and create nodes as needed
        let mut node_idx = 0; // root
        for level in 0..self.depth {
            // Calculate child index in 4x4x4 grid
            let child_idx = self.get_child_index(morton, level);
            // ... traversal logic
        }
    }
}
```

### Task 3-4 Pseudocode  
```rust
// Task 3: Streaming system
impl TerrainStreamer {
    pub async fn new(cache_size_mb: usize) -> Result<Self> {
        // PATTERN: Use tokio for async I/O
        let runtime = tokio::runtime::Runtime::new()?;
        
        // CRITICAL: Align page size with octree nodes
        let page_size = 4096; // 4KB = ~64 nodes
        
        Ok(Self {
            cache: LruCache::new(cache_size_mb * 1024 * 1024 / page_size),
            runtime: Arc::new(runtime),
            loading: Arc::new(Mutex::new(HashSet::new())),
        })
    }
    
    pub async fn load_chunk(&self, chunk_id: ChunkId) -> Result<Arc<TerrainData>> {
        // PATTERN: Check cache first
        if let Some(data) = self.cache.get(&chunk_id) {
            return Ok(data.clone());
        }
        
        // GOTCHA: Prevent duplicate loads
        if self.loading.lock().contains(&chunk_id) {
            // Wait for existing load
        }
        
        // Load from disk with mmap
        let file_path = self.get_chunk_path(chunk_id);
        let mmap = unsafe { Mmap::map(&File::open(file_path)?)? };
        
        // Parse and cache
        let data = Arc::new(parse_terrain_data(&mmap)?);
        self.cache.put(chunk_id, data.clone());
        Ok(data)
    }
}

// Task 4: Shader structure
// terrain_raymarch.wgsl
struct TerrainData {
    nodes: array<OctreeNode>,
}

@group(2) @binding(0)
var<storage, read> terrain: TerrainData;

fn traverse_octree(ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> HitInfo {
    // CRITICAL: Use DDA for grid traversal
    var t = 0.0;
    let max_t = 10000.0; // 10km max
    
    while (t < max_t) {
        let pos = ray_origin + ray_dir * t;
        
        // PATTERN: Hierarchical traversal
        var node_idx = 0u;
        var node_size = WORLD_SIZE;
        
        for (var level = 0u; level < MAX_DEPTH; level++) {
            let node = terrain.nodes[node_idx];
            
            // GOTCHA: Check child mask before access
            if ((node.child_mask & child_bit) == 0u) {
                // Empty space, skip
                t += node_size;
                break;
            }
            
            // Continue traversal...
        }
    }
}
```

### Integration Points
```yaml
RENDERER:
  - location: engine/src/graphics/renderer.rs
  - method: "Add render_terrain() method after render_with_selection()"
  - pattern: "Query world for VoxelTerrain components like MeshId"
  
COMPONENTS:
  - location: engine/src/io/component_registry.rs
  - action: "Register VoxelTerrain and TerrainMotion if using serialization"
  
SHADERS:
  - location: engine/src/shaders/
  - files: "terrain_raymarch.wgsl"
  - pattern: "Follow logarithmic_depth.wgsl for uniform structure"
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run after each file creation
cargo fmt --all -- --check
cargo clippy -p engine --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// In engine/src/terrain/octree.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_morton_encoding() {
        let pos = DVec3::new(100.0, 200.0, 300.0);
        let morton = morton_encode_3d(pos);
        let decoded = morton_decode_3d(morton);
        assert!((pos - decoded).length() < 0.001);
    }
    
    #[test]
    fn test_octree_insert_query() {
        let mut octree = TerrainOctree::new(5);
        octree.insert_voxel(DVec3::new(10.0, 10.0, 10.0), 1);
        
        let result = octree.query_voxel(DVec3::new(10.0, 10.0, 10.0));
        assert_eq!(result, Some(1));
    }
    
    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c"); // Should evict 1
        
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&"b"));
    }
}
```

```bash
# Run tests
cargo test -p engine terrain
# Expected: All tests pass
```

### Level 3: Integration Test
```bash
# Full engine validation
just preflight

# Manual test - create a test scene with terrain
# game/assets/scenes/terrain_test.json
{
  "entities": [
    {
      "components": {
        "Name": { "name": "Test Terrain" },
        "VoxelTerrain": {
          "data_id": "test_terrain",
          "lod_bias": 1.0
        },
        "WorldTransform": {
          "position": [0.0, 0.0, 0.0],
          "rotation": [0.0, 0.0, 0.0, 1.0],
          "scale": [1.0, 1.0, 1.0]
        }
      }
    }
  ]
}
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Formatting correct: `cargo fmt --all -- --check`
- [ ] Documentation builds: `cargo doc --workspace --no-deps --document-private-items`
- [ ] Terrain renders correctly with LOD
- [ ] Memory usage stays within budget
- [ ] Streaming latency < 100ms
- [ ] Movement interpolation is smooth

## Error Handling Strategy
- Streaming failures: Fall back to low-detail placeholder
- GPU buffer allocation: Gracefully degrade quality
- Out of memory: Evict LRU chunks more aggressively
- Malformed data: Log error and skip chunk
- Large world precision: Automatic via camera-relative rendering

---

## Score: 8/10

### Confidence Assessment
- **Strengths**: 
  - Clear architecture based on engine patterns
  - Proven 64-tree approach with references
  - Leverages existing systems (large world coords, ECS)
  - Phased implementation reduces risk
  
- **Risks**:
  - Complex streaming system (mitigated by starting simple)
  - No compute shaders (fragment shader approach is proven)
  - Movement interpolation complexity (incremental approach)
  - WebGPU storage buffer limits (designed for multiple buffers)

The implementation path is well-defined with clear patterns to follow and comprehensive validation gates. The main complexity lies in the streaming system, but the phased approach and extensive testing should ensure success.