name: "Fix AVBD Physics Drift and Instability Issues"
description: |

## Purpose
Fix persistent physics drift and instability in the AVBD solver causing objects to move randomly without forces, implement proper transform interpolation, and resolve unsafe static state issues.

## Core Principles
1. **Safety First**: Replace unsafe static mutable state with thread-safe alternatives
2. **Proper Interpolation**: Implement transform interpolation for smooth physics rendering
3. **Parameter Consistency**: Align AVBD solver parameters across the codebase
4. **Comprehensive Testing**: Validate each fix with tests before proceeding
5. **Global rules**: Follow all rules in CLAUDE.md, especially no root files and proper logging

---

## Goal
Eliminate physics drift where objects move randomly without forces, implement proper transform interpolation between fixed timesteps, and ensure AVBD solver stability with consistent parameters and proper warmstarting.

## Why
- Objects are drifting randomly even without Rigidbody components
- Physics simulation is unstable causing unpredictable behavior
- Missing interpolation causes visual jitter between fixed timesteps
- Unsafe static accumulator could cause race conditions
- Parameter mismatches between config and solver defaults

## What
- Implement PreviousTransform component for interpolation
- Replace unsafe static PHYSICS_ACCUMULATOR with proper state management
- Implement store_previous_transforms and interpolate_transforms functions
- Fix AVBD parameter consistency
- Ensure transform hierarchy updates work with physics
- Add comprehensive tests for physics stability

### Success Criteria
- [x] No objects drift without applied forces
- [x] Smooth visual interpolation between physics steps
- [x] Thread-safe physics accumulator
- [x] Consistent AVBD parameters
- [x] All physics tests pass
- [x] No hierarchy test failures

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://github.com/Raikiri/LegitParticles
  why: AVBD reference implementation showing proper parameter usage and warmstarting
  
- url: https://www.gamedev.net/tutorials/programming/math-and-physics/understanding-constraint-resolution-in-physics-engine-r4839/
  why: Explains position drift in constraint solvers and Baumgarte stabilization
  
- url: https://kevinyu.net/2018/01/17/understanding-constraint-solver-in-physics-engine/
  why: Details on position drift, velocity constraints, and proper interpolation
  
- url: https://mmacklin.com/EG2015PBD.pdf
  why: Position-based dynamics methods and stability considerations

- file: engine/src/physics/systems.rs
  why: Current implementation with stub interpolation functions and unsafe static

- file: engine/src/physics/avbd_solver.rs
  why: AVBD configuration and solver implementation

- file: engine/src/core/entity/components.rs
  why: Pattern for defining new components like PreviousTransform

- file: engine/tests/physics_minimal_test.rs
  why: Test pattern for physics validation
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── physics/
│   │   ├── systems.rs          # Main physics system (has stubs and unsafe static)
│   │   ├── avbd_solver.rs      # AVBD solver implementation
│   │   ├── constraints.rs      # Constraint implementations
│   │   └── mod.rs
│   └── core/
│       └── entity/
│           ├── components.rs   # Component definitions
│           └── hierarchy.rs    # Transform hierarchy (failing tests)
└── tests/
    └── physics_*.rs           # Physics tests
```

### Desired Additions
```bash
engine/
├── src/
│   ├── physics/
│   │   ├── interpolation.rs   # NEW: Transform interpolation implementation
│   │   └── accumulator.rs     # NEW: Thread-safe physics accumulator
│   └── core/
│       └── entity/
│           └── components.rs   # ADD: PreviousTransform component
```

### Known Gotchas & Issues
```rust
// CRITICAL: Current issues causing drift
// 1. store_previous_transforms is a stub - no actual storage
// 2. interpolate_transforms is a stub - no interpolation happening
// 3. PHYSICS_ACCUMULATOR is static mut - not thread safe
// 4. AVBD config mismatch: create_physics_solver uses beta=10.0, default uses beta=15.0
// 5. No PreviousTransform component exists for interpolation
// 6. Comment states "AVBD physics update system (currently broken, needs fixes)"
// 7. Transform hierarchy tests are failing - propagation issues
```

## Implementation Blueprint

### Data Models
```rust
// Add to engine/src/core/entity/components.rs
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Component,
    Default
)]
pub struct PreviousTransform {
    pub position: Vec3,
    pub rotation: Quat,
}

// Add to engine/src/physics/accumulator.rs
pub struct PhysicsAccumulator {
    accumulated_time: f32,
}

impl PhysicsAccumulator {
    pub fn new() -> Self {
        Self { accumulated_time: 0.0 }
    }
    
    pub fn accumulate(&mut self, dt: f32) -> f32 {
        self.accumulated_time += dt;
        self.accumulated_time
    }
    
    pub fn consume(&mut self, timestep: f32) -> bool {
        if self.accumulated_time >= timestep {
            self.accumulated_time -= timestep;
            true
        } else {
            false
        }
    }
    
    pub fn alpha(&self, timestep: f32) -> f32 {
        self.accumulated_time / timestep
    }
}
```

### Task List

```yaml
Task 1:
CREATE engine/src/core/entity/components.rs:
  - ADD PreviousTransform component after Transform
  - FOLLOW pattern from Transform component
  - INCLUDE derive macros but exclude EditorUI
  - REGISTER in component registry

Task 2:
CREATE engine/src/physics/accumulator.rs:
  - IMPLEMENT PhysicsAccumulator struct
  - ADD thread-safe methods for accumulation
  - INCLUDE alpha calculation for interpolation
  - ADD to physics mod.rs

Task 3:
MODIFY engine/src/physics/systems.rs:
  - REMOVE static mut PHYSICS_ACCUMULATOR
  - ADD PhysicsAccumulator parameter to update_physics_system
  - IMPLEMENT store_previous_transforms properly:
    - Query entities with Transform AND Rigidbody
    - Add PreviousTransform if missing
    - Copy current transform to PreviousTransform
  - IMPLEMENT interpolate_transforms:
    - Query entities with Transform, PreviousTransform, AND Rigidbody
    - Lerp position and slerp rotation based on alpha
    - Update only visual transform, not physics state

Task 4:
MODIFY engine/src/physics/systems.rs - create_physics_solver:
  - CHANGE beta from 10.0 to match AVBDConfig::default() (15.0)
  - CHANGE k_start from 5000.0 to match AVBDConfig::default() (1000.0)
  - OR create custom config that documents why different values

Task 5:
MODIFY engine/src/physics/avbd_solver.rs:
  - ADD constraint persistence between frames for warmstarting
  - ENSURE lambda values are preserved correctly
  - FIX any constraint cache invalidation issues

Task 6:
CREATE engine/src/physics/interpolation.rs:
  - IMPLEMENT interpolation helpers
  - ADD smooth damp functions for edge cases
  - HANDLE parent-child transform relationships

Task 7:
MODIFY engine/src/app.rs:
  - ADD PhysicsAccumulator to EngineApp struct
  - PASS accumulator to update_physics_system
  - INITIALIZE in new()

Task 8:
CREATE tests in engine/tests/physics_interpolation_test.rs:
  - TEST transform interpolation accuracy
  - TEST accumulator behavior
  - TEST no drift when no forces applied
```

### Pseudocode for Key Functions

```rust
// Task 3 - store_previous_transforms
fn store_previous_transforms(world: &mut World) {
    // PATTERN: Query with multiple components
    for (entity, (transform, _rigidbody)) in world.query::<(&Transform, &Rigidbody)>().iter() {
        // PATTERN: Add component if missing (see hierarchy.rs)
        if world.get::<PreviousTransform>(entity).is_err() {
            let _ = world.insert_one(entity, PreviousTransform::default());
        }
        
        // CRITICAL: Use query_one_mut to avoid borrow conflicts
        if let Ok((prev_transform,)) = world.query_one_mut::<(&mut PreviousTransform,)>(entity) {
            prev_transform.position = transform.position;
            prev_transform.rotation = transform.rotation;
        }
    }
}

// Task 3 - interpolate_transforms
fn interpolate_transforms(world: &mut World, alpha: f32) {
    // CRITICAL: Only interpolate entities with physics
    for (entity, (transform, prev_transform, _rigidbody)) in 
        world.query::<(&mut Transform, &PreviousTransform, &Rigidbody)>().iter() {
        // GOTCHA: Don't interpolate kinematic bodies
        let rigidbody = world.get::<&Rigidbody>(entity).unwrap();
        if rigidbody.is_kinematic {
            continue;
        }
        
        // PATTERN: Lerp position, slerp rotation
        transform.position = prev_transform.position.lerp(transform.position, alpha);
        transform.rotation = prev_transform.rotation.slerp(transform.rotation, alpha);
    }
}
```

### Integration Points
```yaml
COMPONENTS:
  - add to: engine/src/core/entity/components.rs
  - pattern: Copy Transform component pattern
  - register: Add to register_all_components()
  
PHYSICS:
  - modify: engine/src/physics/systems.rs
  - add import: use crate::physics::accumulator::PhysicsAccumulator;
  - pattern: Follow existing query patterns
  
APP:
  - modify: engine/src/app.rs
  - add field: physics_accumulator: PhysicsAccumulator
  - init: PhysicsAccumulator::new() in new()
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// CREATE engine/tests/physics_drift_test.rs
#[test]
fn test_no_drift_without_forces() {
    let mut world = World::new();
    let mut solver = create_default_solver();
    let mut accumulator = PhysicsAccumulator::new();
    
    // Create entity with rigidbody at rest
    let entity = world.spawn((
        Transform::from_position(Vec3::new(0.0, 1.0, 0.0)),
        GlobalTransform::default(),
        Rigidbody {
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            use_gravity: false, // No external forces
            ..Default::default()
        },
    ));
    
    let initial_pos = world.get::<Transform>(entity).unwrap().position;
    
    // Run physics for 100 frames
    for _ in 0..100 {
        update_physics_system(&mut world, &mut solver, &PhysicsConfig::default(), &mut accumulator, 0.016);
    }
    
    let final_pos = world.get::<Transform>(entity).unwrap().position;
    
    // Position should not drift
    assert!((final_pos - initial_pos).length() < 0.001, 
            "Object drifted by {} units", (final_pos - initial_pos).length());
}

#[test]
fn test_interpolation_smoothness() {
    // Test that interpolation produces smooth motion
    let mut accumulator = PhysicsAccumulator::new();
    accumulator.accumulate(0.008); // Half a timestep
    
    let alpha = accumulator.alpha(0.016);
    assert!((alpha - 0.5).abs() < 0.001);
}
```

```bash
# Run physics tests
cargo test --package engine physics -- --nocapture

# If failing: Check for transform propagation issues
```

### Level 3: Integration Test
```bash
# Run the game with physics scene
SCENE=physics_debug_test cargo run

# Observe:
# - No random drift of objects
# - Smooth motion between frames
# - Stable stacking of objects

# Check logs for drift warnings
grep -i "drift\|unstable" game.log
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets --all-features`
- [ ] Manual test shows no drift: `SCENE=physics_minimal cargo run`
- [ ] Interpolation is smooth visually
- [ ] No unsafe code warnings
- [ ] Transform hierarchy tests pass
- [ ] Logs show stable physics: `grep "physics" game.log`

---

## Anti-Patterns to Avoid
- ❌ Don't create new files in root directory
- ❌ Don't use static mut for state
- ❌ Don't interpolate kinematic bodies
- ❌ Don't modify physics state during interpolation
- ❌ Don't skip hierarchy updates
- ❌ Don't ignore failing tests
- ❌ Don't use println! - use tracing macros

## Notes
- The AVBD solver is sensitive to parameter changes - test thoroughly
- Interpolation must only affect visual representation, not physics state
- Parent-child transform relationships must be preserved
- Warmstarting requires constraint persistence between frames