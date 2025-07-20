## FEATURE:
Implement a high-performance voxel terrain component for the engine using sparse voxel octrees with disk streaming, raymarching renderer, LOD support, movable terrain sections with interpolation, and full integration with the engine's large world coordinate system for rendering massive planetary terrains and asteroids.

## EXAMPLES:
.claude/examples/console-output-before.txt – Shows logging output before fix was applied
.claude/examples/console-output-after.txt – Shows improved logging after fix

## DOCUMENTATION:
- https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/ – Guide to fast voxel ray tracing using sparse 64-trees
- https://github.com/expenses/tree64 – Sparse 64-tree implementation based on October 2024 guide
- https://github.com/davids91/shocovox – WGSL sparse voxel octree with GPU raytracing
- https://research.nvidia.com/publication/efficient-sparse-voxel-octrees – NVIDIA's foundational ESVO paper
- https://www.w3.org/TR/webgpu/ – WebGPU specification for compute shader requirements
- https://www.w3.org/TR/WGSL/ – WGSL specification for shader implementation

## OTHER CONSIDERATIONS:
- **Disk Streaming Requirements**:
  - Implement virtual voxel system similar to virtual texturing
  - Use memory-mapped files (mmap) for efficient disk access
  - Implement LRU cache for loaded octree nodes with configurable memory budget
  - Async loading system using Rust's tokio or similar for non-blocking I/O
  - Consider using a custom file format optimized for spatial queries (e.g., Morton encoding)
  - Page size should align with octree node boundaries (e.g., 4KB pages for 64-tree nodes)
- **Streaming Architecture**:
  - Frustum culling to determine visible octree nodes
  - Predictive loading based on camera movement direction
  - Multiple resolution levels stored on disk for quick LOD switching
  - Background thread pool for decompression if using compressed voxel data
- Consider using 64-trees (4³ branching) instead of traditional octrees (2³) for up to 60% performance improvement
- WebGPU compute shaders require explicit feature request in device creation
- Optimal workgroup size is 64 threads (or 4x4x4 for 3D operations)
- Storage buffer size limited to 2GB in WebGPU, may need multiple buffers for large worlds
- Implement LOD with 4-5 levels using hierarchical chunk reduction (64³ → 32³ → 16³ → 8³ → 4³)
- **Large World Coordinate Integration**:
  - Voxel positions must use f64 precision (DVec3) for world-space coordinates
  - Integrate with existing camera-relative rendering system (already converts to f32 for GPU)
  - Support for voxel worlds up to 10^21 meters (galaxy scale) using hierarchical coordinates
  - Octree root nodes should align with GalaxySector boundaries for extreme scales
  - Use WorldTransform component for voxel chunk entities to leverage automatic precision handling
  - Streaming system must handle coordinate precision when loading distant chunks
- Consider hybrid approach with voxel bricks (8 voxels per leaf) for better memory efficiency
- Raymarching in fragment shader vs compute shader tradeoffs based on target resolution
- Engine currently lacks compute shader infrastructure - will need to extend RenderContext and create ComputePipeline
- **Movable Voxel Objects Support**:
  - Implement "voxel entity" system where entire octree branches can be attached to Transform components
  - Support for hierarchical movement (e.g., cities on rotating planets, asteroids in orbit)
  - Dual-position interpolation: store previous and current transform for smooth transitions
  - Velocity-based prediction for reducing perceived lag during movement
  - Separate static world octree from dynamic object octrees for optimization
  - Instance-based rendering for repeated voxel objects (asteroids, space debris)
- **Movement Interpolation System**:
  - Frame-rate independent interpolation using delta time
  - Quaternion slerp for rotation interpolation
  - Hermite spline interpolation for smooth acceleration/deceleration
  - LOD-aware interpolation (less precision needed for distant objects)
  - Temporal upsampling for 120Hz+ displays
  - Motion blur support through velocity buffers
- **Dynamic Object Optimization**:
  - Bounding volume hierarchies (BVH) for moving voxel objects
  - Spatial hashing for broad-phase collision detection
  - Predictive streaming based on object trajectories
  - Delta compression for networked voxel object updates
- **Engine Integration as Terrain Component**:
  - Add as `engine/src/terrain/` module following existing module patterns
  - Create `VoxelTerrain` component for ECS integration
  - Extend renderer with terrain-specific rendering pipeline
  - Integrate with existing Transform/WorldTransform hierarchy
  - Follow engine's component registration patterns (see io/component_registry.rs)
- Integration points: new terrain module under engine/src/, VoxelTerrain component in ECS, terrain pipeline in renderer