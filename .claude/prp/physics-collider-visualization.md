# Physics Simulation Fixes and Collider Visualization

## Objective
Fix physics simulation stability issues and implement visual debugging for collision shapes in the viewport.

## Context and Research

### Current Physics Issues
1. **Variable timestep**: Using raw frame delta time causes instability
2. **Parameter tuning**: AVBD solver parameters not optimized for stable stacking
3. **Collision response**: Contact constraints not generating sufficient separation forces
4. **Damping**: Over-damping or under-damping affecting simulation quality

### Current Rendering Limitations
- No line primitive support (only triangles)
- No debug visualization system
- Outline system only for selection highlighting

### Reference Documentation
- AVBD paper: https://graphics.cs.utah.edu/research/projects/avbd/
- AVBD implementation: https://github.com/savant117/avbd-demo2d
- wgpu wireframe: https://github.com/gfx-rs/wgpu/discussions/1818
- Debug lines: https://www.gijskaerts.com/wordpress/?p=190

## Implementation Blueprint

### Phase 1: Physics Stability Fixes

```rust
// 1. Fixed timestep implementation in app.rs
const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0; // 60 Hz
let mut physics_accumulator = 0.0;

// In update loop:
physics_accumulator += delta_time;
while physics_accumulator >= PHYSICS_TIMESTEP {
    update_physics_system(&mut world, &mut solver, PHYSICS_TIMESTEP);
    physics_accumulator -= PHYSICS_TIMESTEP;
}

// 2. Improved AVBD configuration
AVBDConfig {
    iterations: 20,              // Increase for stability
    beta: 5.0,                  // Moderate stiffness ramping
    alpha: 0.98,                // Higher error correction
    gamma: 0.95,                // Moderate warm-starting
    k_start: 5000.0,            // Higher initial stiffness
    gravity: Vec3::new(0.0, -9.81, 0.0),
}

// 3. Contact constraint improvements
// Add penetration recovery bias
let penetration_bias = -0.2 * penetration.max(0.0) / dt;
target_velocity += penetration_bias;

// 4. Improve damping calculation
// Apply exponential damping instead of linear
body.linear_velocity *= (1.0 - linear_damping).powf(dt);
body.angular_velocity *= (1.0 - angular_damping).powf(dt);
```

### Phase 2: Debug Line Rendering Pipeline

```rust
// 1. Create debug line shader (engine/src/shaders/debug_lines.wgsl)
struct DebugVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

// 2. Add debug pipeline to renderer.rs
pub struct Renderer {
    // ... existing fields
    debug_pipeline: RenderPipeline,
    debug_line_buffer: Buffer,
    debug_line_count: u32,
}

// 3. Debug pipeline creation
fn create_debug_pipeline(device: &Device) -> RenderPipeline {
    let pipeline_layout = /* ... */;
    
    device.create_render_pipeline(&RenderPipelineDescriptor {
        primitive: PrimitiveState {
            topology: PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: false, // Don't write depth
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        // ... rest of pipeline config
    })
}
```

### Phase 3: Collider Wireframe Generation

```rust
// In engine/src/physics/debug_visualization.rs
pub mod debug_visualization {
    use crate::physics::CollisionShape;
    use glam::Vec3;
    
    pub struct DebugLine {
        pub start: Vec3,
        pub end: Vec3,
        pub color: [f32; 4],
    }
    
    pub fn generate_collider_lines(shape: &CollisionShape) -> Vec<DebugLine> {
        match shape {
            CollisionShape::Box { half_extents } => {
                generate_box_lines(*half_extents)
            }
            CollisionShape::Sphere { radius } => {
                generate_sphere_lines(*radius, 16) // 16 segments
            }
            CollisionShape::Capsule { radius, half_height } => {
                generate_capsule_lines(*radius, *half_height, 16)
            }
        }
    }
    
    fn generate_box_lines(half_extents: Vec3) -> Vec<DebugLine> {
        let corners = [
            Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
            Vec3::new( half_extents.x, -half_extents.y, -half_extents.z),
            // ... all 8 corners
        ];
        
        // Generate 12 edges
        vec![
            DebugLine { start: corners[0], end: corners[1], color: [0.0, 1.0, 0.0, 1.0] },
            // ... remaining edges
        ]
    }
}
```

### Phase 4: Integration and UI

```rust
// 1. Add debug settings to EditorState
pub struct EditorState {
    // ... existing fields
    pub show_colliders: bool,
    pub show_contact_points: bool,
    pub show_aabbs: bool,
}

// 2. Update viewport rendering
fn render_viewport(&mut self, renderer: &mut Renderer, world: &World) {
    // ... existing rendering
    
    if self.show_colliders {
        renderer.render_debug_colliders(world);
    }
}

// 3. Add UI toggle in viewport panel
ui.checkbox("Show Colliders", &mut self.editor_state.show_colliders);
ui.checkbox("Show Contacts", &mut self.editor_state.show_contact_points);
ui.checkbox("Show AABBs", &mut self.editor_state.show_aabbs);

// 4. Color coding for collider states
fn get_collider_color(rigidbody: &Rigidbody, collider: &Collider) -> [f32; 4] {
    if collider.is_trigger {
        [1.0, 0.0, 0.0, 0.7] // Red for triggers
    } else if rigidbody.is_kinematic {
        [0.0, 1.0, 0.0, 0.7] // Green for kinematic
    } else {
        [0.0, 0.5, 1.0, 0.7] // Blue for dynamic
    }
}
```

## Task List (In Order)

1. **Fix Physics Timestep**
   - Implement fixed timestep with accumulator in `app.rs` and `game/src/main.rs`
   - Add interpolation for smooth rendering between physics steps
   - Test with various frame rates

2. **Tune AVBD Parameters**
   - Update `AVBDConfig::default()` with new values
   - Add penetration recovery bias to contact constraints
   - Fix damping to use exponential decay
   - Add debug logging for constraint violations

3. **Create Debug Line Shader**
   - Create `debug_lines.wgsl` with simple vertex/fragment shaders
   - Add shader to `shaders/mod.rs`
   - Support per-vertex colors

4. **Implement Debug Pipeline**
   - Add debug pipeline to `Renderer`
   - Create line buffer management system
   - Implement `render_debug_lines()` method

5. **Generate Collider Wireframes**
   - Create `debug_visualization.rs` module
   - Implement line generation for each collision shape
   - Cache generated lines per shape type/size

6. **Integrate Debug Rendering**
   - Add debug render pass after main rendering
   - Query entities with colliders and transform to world space
   - Batch debug lines for efficient rendering

7. **Add Editor UI Controls**
   - Add debug visualization toggles to viewport panel
   - Store settings in `EditorState`
   - Pass settings to renderer

8. **Write Tests**
   - Test fixed timestep accumulator
   - Test collider line generation
   - Test debug rendering doesn't affect main rendering
   - Physics stability tests with stacking

## Validation Gates

```bash
# Syntax and style check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run physics tests
cargo test -p engine physics

# Run graphics tests  
cargo test -p engine graphics

# Integration test
cargo test --test physics

# Full validation
just preflight

# Manual testing checklist:
# 1. Load physics_stacking.json - boxes should stack without interpenetration
# 2. Toggle collider visualization - wireframes should appear
# 3. Drop objects - should fall at realistic speed (~9.8 m/s²)
# 4. Check different collider types - box, sphere, capsule all render correctly
# 5. Performance - debug rendering shouldn't significantly impact FPS
```

## Error Handling Strategy

1. **Feature Detection**: Check for `NON_FILL_POLYGON_MODE` support, fall back to triangle-based lines
2. **Buffer Overflow**: Pre-allocate debug line buffer, warn if exceeded
3. **Shader Compilation**: Graceful fallback if debug shader fails
4. **Physics Divergence**: Clamp velocities and positions to prevent explosions

## Performance Considerations

- Cache wireframe meshes per shape type
- Use instanced rendering for identical shapes
- Cull debug lines outside viewport
- Limit debug line count (configurable)
- Only update debug lines when colliders change

## Code References

- Physics update: `engine/src/app.rs:242`
- AVBD solver: `engine/src/physics/avbd_solver.rs`
- Contact constraints: `engine/src/physics/constraints.rs:100-184`
- Renderer pipelines: `engine/src/graphics/pipeline.rs`
- Outline example: `engine/src/graphics/renderer.rs:render_with_outline()`
- Editor viewport: `editor/src/panels/viewport.rs`

## Success Criteria

1. Physics objects stack stably without jitter
2. Objects fall at realistic speeds
3. Colliders visible as colored wireframes
4. Debug visualization toggleable from UI
5. No significant performance impact
6. All physics tests pass

## Confidence Score: 8/10

The approach is well-researched with clear implementation steps. Points deducted for:
- Potential platform compatibility issues with line rendering
- AVBD parameter tuning may require iteration
- Integration complexity between physics and rendering systems