# Comprehensive Physics System Bug Fixes

## Objective
Fix all critical physics bugs including the bypassed AVBD solver, incorrect contact calculations, jittering, and velocity issues to create a stable, production-ready physics simulation.

## Context and Research

### Critical Issues Identified
1. **AVBD Solver Bypassed**: `systems.rs:21-23` has TODO that routes to simple_physics instead
2. **Contact Points Wrong**: World positions calculated incorrectly (e.g., Vec3(-4.8, 0.4, -5.2) for origin object)
3. **Hardcoded Thresholds**: Velocity thresholds (0.1, 0.05) cause sticking/stopping
4. **Transform Confusion**: GlobalTransform vs Transform causing hierarchy issues
5. **Incomplete Correction**: 0.8 factor leaves objects interpenetrating
6. **No Configuration**: Physics parameters hardcoded throughout
7. **Debug Viz Disconnected**: Visualization exists but not integrated
8. **Missing Features**: No warmstarting, using Baumgarte instead of NGS

### Root Cause Analysis
The physics system was partially implemented with a sophisticated AVBD solver, but critical integration issues and a fallback to simple_physics have created a fragmented system where:
- Advanced features exist but aren't used
- Simple physics lacks proper constraint resolution
- Contact generation has fundamental math errors
- No unified configuration system

### Reference Implementation Analysis
From AVBD papers and reference implementations:
- Warmstarting with γ = 0.99 is critical for stability
- NGS (Nonlinear Gauss-Seidel) position correction prevents drift
- Contact points must be in world space, not local
- Simultaneous constraint resolution prevents jitter

## Implementation Blueprint

### Phase 1: Fix Contact Point Calculations

```rust
// Fix in narrow_phase.rs - contact points must be in world space
pub fn sphere_box_contact(
    sphere_pos: Vec3,
    sphere_radius: f32,
    box_pos: Vec3,
    box_rot: Quat,
    box_half_extents: Vec3,
) -> Option<Contact> {
    // Transform to box local space for calculation
    let local_sphere_pos = box_rot.conjugate() * (sphere_pos - box_pos);
    
    // Find closest point on box in local space
    let closest_local = local_sphere_pos.clamp(
        -box_half_extents,
        box_half_extents
    );
    
    // Transform back to world space
    let closest_world = box_pos + box_rot * closest_local;
    
    // Contact point is on sphere surface toward box
    let normal = (sphere_pos - closest_world).normalize();
    let contact_point = sphere_pos - normal * sphere_radius;
    
    let distance = (sphere_pos - closest_world).length();
    let penetration = sphere_radius - distance;
    
    if penetration > 0.0 {
        Some(Contact {
            point: contact_point, // Now correctly in world space
            normal,
            penetration,
            entity_a: /* ... */,
            entity_b: /* ... */,
        })
    } else {
        None
    }
}
```

### Phase 2: Enable AVBD Solver with Proper Integration

```rust
// Fix in systems.rs - remove the bypass
pub fn update_physics_system(world: &mut World, solver: &mut AVBDSolver, delta_time: f32) {
    trace!("Starting AVBD physics update, dt={}", delta_time);
    
    // Use fixed timestep with interpolation
    const PHYSICS_TIMESTEP: f32 = 1.0 / 120.0; // 120Hz for stability
    static mut ACCUMULATOR: f32 = 0.0;
    
    unsafe {
        ACCUMULATOR += delta_time;
        
        while ACCUMULATOR >= PHYSICS_TIMESTEP {
            // Run the actual AVBD physics
            update_physics_system_avbd(world, solver, PHYSICS_TIMESTEP);
            ACCUMULATOR -= PHYSICS_TIMESTEP;
        }
        
        // Interpolate positions for smooth rendering
        let alpha = ACCUMULATOR / PHYSICS_TIMESTEP;
        interpolate_transforms(world, alpha);
    }
}

// Add proper warmstarting to AVBDSolver
impl AVBDSolver {
    pub fn warmstart_constraints(&mut self) {
        for constraint in &mut self.constraints {
            constraint.lambda *= self.config.gamma; // 0.99 decay
            constraint.stiffness = constraint.stiffness * self.config.gamma 
                                   + self.config.k_start * (1.0 - self.config.gamma);
        }
    }
}
```

### Phase 3: Implement NGS Position Correction

```rust
// Replace Baumgarte stabilization with NGS
pub fn apply_position_correction(
    bodies: &mut [RigidbodyData],
    contacts: &[Contact],
    iterations: u32,
) {
    const SLOP: f32 = 0.004; // Allow 4mm penetration for stability
    const CORRECTION_RATE: f32 = 0.8;
    
    for _ in 0..iterations {
        for contact in contacts {
            if contact.penetration > SLOP {
                let correction = (contact.penetration - SLOP) * CORRECTION_RATE;
                
                // Get body indices
                let (body_a, body_b) = get_bodies_mut(bodies, contact);
                
                // Calculate impulse for position correction
                let total_mass = 1.0 / body_a.mass + 1.0 / body_b.mass;
                let impulse = correction / total_mass;
                
                // Apply corrections
                if !body_a.is_kinematic {
                    body_a.position -= contact.normal * (impulse / body_a.mass);
                }
                if !body_b.is_kinematic {
                    body_b.position += contact.normal * (impulse / body_b.mass);
                }
            }
        }
    }
}
```

### Phase 4: Configurable Physics Parameters

```rust
// Add to physics/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub fixed_timestep: f32,
    pub position_iterations: u32,
    pub velocity_iterations: u32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub restitution_threshold: f32,
    pub contact_slop: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_timestep: 1.0 / 120.0,
            position_iterations: 4,
            velocity_iterations: 8,
            linear_damping: 0.01,
            angular_damping: 0.01,
            restitution_threshold: 1.0, // m/s
            contact_slop: 0.004, // 4mm
            max_linear_velocity: 100.0,
            max_angular_velocity: 100.0,
        }
    }
}

// Use throughout physics system instead of hardcoded values
impl PhysicsSystem {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            solver: AVBDSolver::new(AVBDConfig {
                iterations: config.velocity_iterations,
                beta: 10.0,
                alpha: 0.98,
                gamma: 0.99,
                k_start: 5000.0,
                gravity: config.gravity,
            }),
        }
    }
}
```

### Phase 5: Fix Transform Hierarchy Integration

```rust
// Ensure physics uses GlobalTransform consistently
fn gather_physics_transforms(world: &World) -> Vec<PhysicsTransform> {
    world.query::<(&GlobalTransform, &Rigidbody, Option<&Transform>)>()
        .iter()
        .map(|(entity, (global, rb, local))| {
            let (scale, rotation, position) = global.matrix.to_scale_rotation_translation();
            
            PhysicsTransform {
                entity,
                world_position: position,
                world_rotation: rotation,
                world_scale: scale,
                local_transform: local.cloned(),
                rigidbody: rb.clone(),
            }
        })
        .collect()
}

// Update both local and global after physics
fn apply_physics_results(world: &mut World, results: &[PhysicsResult]) {
    for result in results {
        // Update local transform considering parent
        if let Ok((transform, parent)) = world.query_one_mut::<(&mut Transform, Option<&Parent>)>(result.entity) {
            if let Some(parent) = parent {
                // Convert world to local space
                if let Ok(parent_global) = world.get::<GlobalTransform>(parent.0) {
                    let parent_inv = parent_global.matrix.inverse();
                    let local_pos = parent_inv.transform_point3(result.world_position);
                    let (_, parent_rot, _) = parent_global.matrix.to_scale_rotation_translation();
                    let local_rot = parent_rot.conjugate() * result.world_rotation;
                    
                    transform.position = local_pos;
                    transform.rotation = local_rot;
                }
            } else {
                // No parent - world space is local space
                transform.position = result.world_position;
                transform.rotation = result.world_rotation;
            }
        }
    }
}
```

### Phase 6: Connect Debug Visualization

```rust
// Add debug system to physics update
pub fn debug_draw_physics(world: &World, debug_renderer: &mut DebugRenderer) {
    // Draw colliders
    for (entity, (collider, global_transform)) in world.query::<(&Collider, &GlobalTransform)>().iter() {
        let (scale, rotation, position) = global_transform.matrix.to_scale_rotation_translation();
        
        let color = if let Ok(rb) = world.get::<Rigidbody>(entity) {
            if rb.is_kinematic {
                [0.0, 1.0, 0.0, 0.7] // Green for kinematic
            } else {
                [0.0, 0.5, 1.0, 0.7] // Blue for dynamic
            }
        } else {
            [0.5, 0.5, 0.5, 0.7] // Gray for static
        };
        
        debug_renderer.draw_collider(&collider.shape, position, rotation, scale, color);
    }
    
    // Draw contact points
    for contact in &world.physics_contacts {
        debug_renderer.draw_sphere(contact.point, 0.05, [1.0, 1.0, 0.0, 1.0]); // Yellow
        debug_renderer.draw_line(
            contact.point,
            contact.point + contact.normal * 0.5,
            [1.0, 0.0, 0.0, 1.0], // Red normal
        );
    }
}
```

## Task List (In Order)

1. **Fix Contact Generation**
   - Fix sphere-box contact world space calculation
   - Fix box-box contact point generation
   - Add proper penetration depth calculation
   - Write comprehensive contact tests

2. **Enable AVBD Solver**
   - Remove simple_physics bypass in systems.rs
   - Implement fixed timestep with accumulator
   - Add transform interpolation for smooth rendering
   - Configure AVBD parameters properly

3. **Implement Warmstarting**
   - Store previous frame's lambda values
   - Apply gamma decay each frame
   - Update stiffness with proper ramping
   - Test constraint stability

4. **Add NGS Position Correction**
   - Replace Baumgarte with NGS solver
   - Implement contact slop (4mm)
   - Add position iteration loop
   - Test stacking stability

5. **Create Physics Configuration**
   - Define PhysicsConfig struct
   - Replace all hardcoded values
   - Add runtime configuration loading
   - Expose in editor UI

6. **Fix Transform Integration**
   - Use GlobalTransform for physics
   - Properly convert world<->local space
   - Handle transform hierarchy
   - Test with parented objects

7. **Connect Debug Visualization**
   - Integrate debug renderer with physics
   - Add UI toggles in editor
   - Draw colliders, contacts, velocities
   - Color-code by body type

8. **Comprehensive Testing**
   - Unit tests for each fix
   - Integration tests for full system
   - Performance benchmarks
   - Scene tests for validation

## Validation Gates

```bash
# After each task, run:
cargo test -p engine physics
cargo clippy --workspace -- -D warnings

# Full validation:
just preflight

# Scene validation:
# 1. physics_stacking.json - stable tower of boxes
# 2. physics_cube_tip.json - cube tips over naturally
# 3. physics_joints.json - constraints don't drift
# 4. physics_stress.json - 1000 bodies at 60fps
```

## Success Criteria

1. **Stability**: Objects rest without jitter or drift
2. **Accuracy**: Contact points correct in world space
3. **Performance**: 1000 bodies at 60 FPS
4. **Robustness**: High mass ratios (1000:1) work
5. **Configurability**: All physics parameters tunable
6. **Debuggability**: Visual debugging available
7. **Integration**: Works with transform hierarchy

## Error Recovery

1. **NaN Detection**: Check all positions/velocities each frame
2. **Explosion Recovery**: Clamp velocities to max values
3. **Constraint Violations**: Log and visualize violations
4. **Penetration Recovery**: Separate bodies over multiple frames

## Migration Strategy

1. Keep simple_physics.rs during transition
2. Add feature flag for AVBD vs simple
3. Migrate scenes one at a time
4. A/B test physics behaviors
5. Remove simple_physics after validation

## Code References

- AVBD bypass: `engine/src/physics/systems.rs:21-23`
- Contact generation: `engine/src/physics/collision/narrow_phase.rs`
- Simple physics: `engine/src/physics/simple_physics.rs`
- Transform hierarchy: `engine/src/core/entity/hierarchy.rs`
- Debug rendering: `engine/src/graphics/debug_renderer.rs`

## Confidence Score: 9/10

High confidence due to:
- Clear identification of all bugs
- Concrete fixes for each issue
- Reference implementations available
- Existing AVBD infrastructure

Point deducted for:
- Complex transform hierarchy integration
- Potential performance tuning iterations needed