name: "Large World Coordinate Support"
description: |

## Purpose
Implement large world coordinate support to handle game worlds beyond the precision limits of single-precision floating point (f32). This will enable the engine to support worlds of planetary scale or larger without floating-point precision artifacts.

## Core Principles
1. **Camera-Relative Rendering**: Use double precision on CPU, single precision on GPU
2. **Minimal Breaking Changes**: Preserve existing API where possible
3. **Performance Aware**: Avoid double precision on GPU due to performance costs
4. **Flexible Architecture**: Support both origin shifting and hierarchical coordinates
5. **Future-Proof**: Design for potential multiplayer support

---

## Goal
Enable the WebGPU engine to handle large game worlds (>1 million units) without precision loss, z-fighting, or jittering artifacts by implementing a camera-relative rendering system with optional origin shifting.

## Why
- **Precision Loss**: Current f32 coordinates lose centimeter precision at ~16,777,216 units
- **Z-Fighting**: Near/far plane ratio of 10,000:1 causes depth buffer precision issues
- **Game Types**: Enable space games, flight sims, open-world games with realistic scales
- **Industry Standard**: Modern engines (UE5, Unity) have large world support
- **Future Multiplayer**: Foundation for server-authoritative large worlds

## What
Implement a dual-coordinate system where gameplay uses f64 world coordinates while rendering uses f32 camera-relative coordinates. Add optional origin shifting for single-player scenarios.

### Success Criteria
- [ ] Objects remain stable at 100 million units from origin
- [ ] No visible jittering when camera moves at large distances
- [ ] Existing scenes continue to work without modification
- [ ] Performance impact <5% for typical scenes
- [ ] Z-fighting reduced with logarithmic depth buffer option

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.godotengine.org/en/stable/tutorials/physics/large_world_coordinates.html
  why: Godot's implementation shows origin shifting patterns and common pitfalls
  critical: Origin shifting breaks in multiplayer - need camera-relative approach
  
- url: https://dev.epicgames.com/documentation/en-us/unreal-engine/large-world-coordinates-in-unreal-engine-5
  why: UE5's Large World Coordinates (LWC) system design decisions
  critical: They use 64-bit doubles throughout, with GPU conversion
  
- url: https://gamedevtricks.com/post/origin-rebasing-space/
  why: Deep dive into spatial rebasing techniques and implementation
  critical: Shows the complexity of origin shifting in practice

- url: https://github.com/bevyengine/bevy/issues/3068
  why: Bevy's discussion on double precision transforms
  critical: Shows Rust/ECS-specific implementation challenges
  
- url: https://github.com/gpuweb/gpuweb/issues/2805
  why: WebGPU/WGSL double precision support status
  critical: Confirms no GPU f64 support - must use f32 on GPU

- file: engine/src/core/entity/components.rs
  why: Current Transform and GlobalTransform components using Vec3 (f32)
  critical: Need to understand existing transform system
  
- file: engine/src/core/entity/hierarchy.rs
  why: Transform hierarchy propagation system
  critical: Must update for large world coordinates
  
- file: engine/src/core/camera.rs
  why: Camera projection and view matrix calculations
  critical: Central to camera-relative rendering
  
- file: engine/src/graphics/renderer.rs
  why: Rendering pipeline and uniform buffer updates
  critical: Where coordinate conversion happens
  
- file: engine/src/shaders/basic.wgsl
  why: Shader coordinate transformations
  critical: Must remain f32 for GPU compatibility
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── core/
│   │   ├── entity/
│   │   │   ├── components.rs      # Transform, GlobalTransform (f32)
│   │   │   ├── hierarchy.rs       # Transform propagation
│   │   │   └── world.rs          # ECS world
│   │   └── camera.rs             # Camera component
│   ├── graphics/
│   │   ├── renderer.rs           # Main rendering loop
│   │   ├── uniform.rs            # GPU uniform buffers
│   │   └── mesh.rs               # Vertex data
│   └── shaders/
│       └── basic.wgsl            # Vertex/fragment shaders
```

### Desired Codebase Structure
```bash
engine/
├── src/
│   ├── core/
│   │   ├── entity/
│   │   │   ├── components.rs      # Transform (f32), WorldTransform (f64)
│   │   │   ├── hierarchy.rs       # Updated for dual coordinates
│   │   │   └── world.rs
│   │   ├── camera.rs             # Enhanced with origin management
│   │   └── coordinates/          # NEW
│   │       ├── mod.rs            # Coordinate system types
│   │       ├── world_transform.rs # f64 world position component
│   │       └── origin_manager.rs  # Origin shifting logic
│   ├── graphics/
│   │   ├── renderer.rs           # Camera-relative conversion
│   │   ├── uniform.rs            # Same (f32)
│   │   └── depth_buffer.rs       # NEW: Logarithmic depth
│   └── shaders/
│       ├── basic.wgsl            # Same (f32)
│       └── logarithmic_depth.wgsl # NEW: Better depth precision
```

### Known Gotchas & Constraints
```rust
// CRITICAL: wgpu/WGSL only supports f32 for vertex data and uniforms
// No f64 support on GPU - must convert on CPU

// GOTCHA: glam uses f32 by default, need glam::DVec3 for f64
use glam::DVec3; // 64-bit vectors
use glam::DMat4; // 64-bit matrices

// CONSTRAINT: Performance - f64 math is slower on CPU
// Minimize f64 operations in hot paths

// GOTCHA: Serialization - serde handles f64 differently
// May need custom serialization for scene files

// CRITICAL: Origin shifting breaks multiplayer
// Server needs precision for ALL players, not just one

// PATTERN: Camera-relative rendering avoids most issues
// Convert world -> camera space on CPU before GPU upload
```

## Implementation Blueprint

### Core Components

```rust
// Task 1: World Transform Component
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldTransform {
    pub position: DVec3,  // f64 world position
    pub rotation: Quat,   // Keep as f32 quaternion
    pub scale: Vec3,      // Keep as f32 scale
}

// Task 2: Coordinate System Manager
pub struct CoordinateSystem {
    camera_origin: DVec3,     // Current camera world position
    origin_threshold: f64,    // When to shift origin (e.g., 10,000 units)
    enable_origin_shift: bool, // Toggle for single/multiplayer
}

// Task 3: Transform Conversion
impl WorldTransform {
    pub fn to_camera_relative(&self, camera_origin: DVec3) -> Transform {
        Transform {
            position: ((self.position - camera_origin) as Vec3),
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}
```

### List of Tasks

```yaml
Task 1 - Create World Coordinate Types:
CREATE engine/src/core/coordinates/mod.rs:
  - EXPORT: WorldTransform, CoordinateSystem types
  - PATTERN: Follow component pattern from components.rs
  
CREATE engine/src/core/coordinates/world_transform.rs:
  - IMPLEMENT: WorldTransform component with DVec3 position
  - ADD: Conversion methods to/from Transform
  - INCLUDE: Tests for precision at large distances

Task 2 - Update Transform System:
MODIFY engine/src/core/entity/components.rs:
  - ADD: Import WorldTransform
  - KEEP: Transform as-is for backward compatibility
  - DOCUMENT: When to use Transform vs WorldTransform

Task 3 - Implement Coordinate System Manager:
CREATE engine/src/core/coordinates/origin_manager.rs:
  - IMPLEMENT: CoordinateSystem struct
  - ADD: update_camera_origin() method
  - ADD: should_shift_origin() logic
  - PATTERN: Use singleton pattern like in app.rs

Task 4 - Update Hierarchy System:
MODIFY engine/src/core/entity/hierarchy.rs:
  - FIND: update_hierarchy_system function
  - ADD: Support for WorldTransform propagation
  - PARALLEL: Keep existing Transform logic
  - PATTERN: Use compound queries for (Transform, Option<WorldTransform>)

Task 5 - Enhance Camera System:
MODIFY engine/src/core/camera.rs:
  - ADD: logarithmic_depth: bool field to Camera
  - ADD: world_position: DVec3 to track camera in world space
  - UPDATE: projection_matrix() to support logarithmic depth
  - REFERENCE: https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html

Task 6 - Update Renderer for Camera-Relative:
MODIFY engine/src/graphics/renderer.rs:
  - FIND: render_with_selection function
  - BEFORE: Creating uniform buffers
  - ADD: Camera-relative transform conversion
  - PATTERN: Query for (GlobalTransform, Option<WorldTransform>)
  - CONVERT: World to camera-relative before GPU upload

Task 7 - Add Logarithmic Depth Shader:
CREATE engine/src/shaders/logarithmic_depth.wgsl:
  - COPY: basic.wgsl as starting point
  - MODIFY: Vertex shader output for logarithmic depth
  - FORMULA: gl_Position.z = log2(max(1e-6, 1.0 + gl_Position.w)) * Fcoef - 1.0
  - ADD: Fcoef uniform for depth coefficient

Task 8 - Scene Serialization Updates:
MODIFY engine/src/io/scene.rs:
  - FIND: Component serialization section
  - ADD: WorldTransform to component_value_to_dynamic
  - ADD: WorldTransform to spawn_entity_from_json
  - TEST: Existing scenes still load correctly

Task 9 - Add Configuration:
MODIFY engine/src/app.rs:
  - ADD: large_world_config to EngineConfig
  - OPTIONS: enable_large_world, origin_shift_threshold, use_logarithmic_depth
  - DEFAULT: All disabled for backward compatibility

Task 10 - Create Integration Tests:
CREATE engine/src/core/coordinates/tests.rs:
  - TEST: Precision at 100 million units
  - TEST: Hierarchy with mixed Transform/WorldTransform
  - TEST: Camera-relative conversion accuracy
  - TEST: Origin shifting behavior
```

### Per Task Implementation Details

```rust
// Task 1: WorldTransform Component
use glam::{DVec3, Quat, Vec3};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct WorldTransform {
    pub position: DVec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl WorldTransform {
    pub fn from_transform(transform: &Transform, offset: DVec3) -> Self {
        Self {
            position: DVec3::new(
                transform.position.x as f64,
                transform.position.y as f64,
                transform.position.z as f64,
            ) + offset,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
    
    pub fn to_camera_relative(&self, camera_origin: DVec3) -> Transform {
        let relative_pos = self.position - camera_origin;
        Transform {
            position: Vec3::new(
                relative_pos.x as f32,
                relative_pos.y as f32,
                relative_pos.z as f32,
            ),
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}

// Task 4: Hierarchy Update Pattern
// In hierarchy.rs, handle dual coordinate systems:
for (entity, (transform, world_transform)) in world
    .query::<(&Transform, Option<&WorldTransform>)>()
    .iter()
{
    if let Some(world_trans) = world_transform {
        // Use world transform for large world entities
        // Convert to camera-relative later in renderer
    } else {
        // Use regular transform for normal entities
        // Existing logic remains unchanged
    }
}

// Task 6: Renderer Camera-Relative Conversion
// Before uploading to GPU:
let camera_world_pos = camera_world_transform.position;
let model_matrix = if let Some(world_trans) = world_transform {
    // Convert world to camera-relative
    let relative_transform = world_trans.to_camera_relative(camera_world_pos);
    relative_transform.to_matrix()
} else {
    // Use existing transform directly
    transform.to_matrix()
};
```

### Integration Points
```yaml
COMPONENTS:
  - WorldTransform: New component for large world entities
  - Transform: Remains for backward compatibility
  - CoordinateSystem: Singleton managing origin

SYSTEMS:
  - hierarchy: Updated to handle dual coordinates
  - renderer: Converts world -> camera space
  - camera: Tracks world position in f64

CONFIG:
  - engine_config.large_world.enable: Toggle feature
  - engine_config.large_world.origin_threshold: When to shift
  - engine_config.large_world.logarithmic_depth: Better Z precision
```

## Validation Loop

### Level 1: Compilation and Formatting
```bash
# Ensure new modules compile
cargo check --workspace
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// CREATE engine/src/core/coordinates/tests.rs
#[test]
fn test_precision_at_large_distances() {
    let world_pos = DVec3::new(100_000_000.0, 0.0, 0.0);
    let camera_pos = DVec3::new(99_999_999.0, 0.0, 0.0);
    
    let world_transform = WorldTransform {
        position: world_pos,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    let relative = world_transform.to_camera_relative(camera_pos);
    assert!((relative.position.x - 1.0).abs() < 0.001);
}

#[test]
fn test_origin_shifting() {
    let mut coord_system = CoordinateSystem::new();
    coord_system.enable_origin_shift = true;
    coord_system.origin_threshold = 10_000.0;
    
    coord_system.update_camera_origin(DVec3::new(15_000.0, 0.0, 0.0));
    assert!(coord_system.camera_origin.x > 0.0);
}
```

```bash
# Run tests
cargo test coordinates --workspace
```

### Level 3: Visual Validation
```bash
# Create test scene with distant objects
cat > game/assets/scenes/large_world_test.json << 'EOF'
{
  "entities": [
    {
      "components": {
        "WorldTransform": {
          "position": [100000000.0, 0.0, 0.0],
          "rotation": [0.0, 0.0, 0.0, 1.0],
          "scale": [1.0, 1.0, 1.0]
        },
        "MeshRenderer": { "mesh": "Cube" },
        "Material": { "color": [1.0, 0.0, 0.0, 1.0] }
      }
    }
  ]
}
EOF

# Run with large world enabled
RUST_LOG=engine::core::coordinates=debug cargo run -p game -- --large-world

# Expected: Red cube visible without jittering
# Camera can fly to distant object smoothly
```

### Level 4: Performance Validation
```bash
# Benchmark regular vs large world
cargo bench --bench transform_benchmark

# Expected: <5% performance impact for typical scenes
# Profile with: cargo flamegraph
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Large world test scene works: distant objects stable
- [ ] Existing scenes unaffected when feature disabled
- [ ] Performance impact acceptable (<5%)
- [ ] Z-fighting reduced with logarithmic depth
- [ ] Multiplayer considerations documented

---

## Anti-Patterns to Avoid
- ❌ Don't use f64 in shaders - not supported by WebGPU
- ❌ Don't break existing Transform API - ensure compatibility
- ❌ Don't origin shift in multiplayer - breaks server authority
- ❌ Don't convert to f64 unnecessarily - performance cost
- ❌ Don't forget to handle serialization - f64 needs care
- ❌ Don't assume all entities need WorldTransform - optional

## Success Confidence Score: 8/10

**High confidence** because:
- Clear technical approach (camera-relative rendering)
- No GPU-side changes needed (remains f32)
- Similar systems proven in other engines
- Backward compatibility maintained
- Comprehensive test coverage planned

**Moderate risks**:
- Performance impact needs careful profiling
- Integration with existing systems requires care
- Serialization format changes need migration path