name: "Rigidbody Physics Implementation with AVBD Algorithm"
description: |

## Purpose
Implement a physics system for the WebGPU engine using the Augmented Vertex Block Descent (AVBD) algorithm, providing stable rigidbody dynamics, collision detection, and constraint solving suitable for real-time applications.

## Core Principles
1. **AVBD Algorithm**: Implement the state-of-the-art AVBD solver for unconditional stability
2. **Component-Based**: Follow existing component patterns with automatic UI generation
3. **GPU-Ready**: Design with future GPU compute acceleration in mind
4. **Large World Support**: Integrate with existing f64 WorldTransform system
5. **Progressive Implementation**: Start with basic rigidbodies, validate, then add constraints

---

## Goal
Implement a complete physics system that enables rigidbody simulation with collisions, joints, and constraints using the AVBD algorithm. The system should handle complex scenarios like stacking, high mass ratios, and stiff constraints while maintaining real-time performance.

## Why
- Enable physics-based gameplay and simulations in the engine
- Provide stable physics that works at various scales (from small objects to planets)
- Support complex mechanical systems with joints and constraints
- Leverage modern GPU-friendly algorithms for future performance scaling

## What
A physics system that includes:
- Rigidbody component with mass, inertia, velocity
- Basic collision shapes (sphere, box, capsule)
- AVBD solver with constraint support
- Contact constraints with friction
- Joint constraints (ball, hinge, fixed)
- Integration with transform hierarchy
- Editor UI support for all physics components

### Success Criteria
- [ ] Rigidbody objects fall under gravity and collide
- [ ] Stable stacking of multiple objects
- [ ] Joints connect bodies without drift
- [ ] Performance: 1000 bodies at 60 FPS
- [ ] Editor can create/modify physics components
- [ ] Tests pass for all physics functionality

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- file: .claude/documentation/Augmented_VBD-SIGGRAPH25.pdf
  why: Original AVBD algorithm paper with mathematical formulation
  
- file: .claude/documentation/Augmented_VBD-SIGGRAPH25_RTL.pdf
  why: Ready-to-learn version with clearer implementation details
  
- url: https://github.com/savant117/avbd-demo2d
  why: Reference C++ implementation showing practical AVBD usage
  
- url: https://docs.rs/glam/latest/glam/
  why: Math library documentation for Vec3, Quat, Mat4 operations
  
- file: engine/src/core/entity/components.rs
  why: Pattern for component definitions with UI annotations
  
- file: engine_derive/src/lib.rs
  why: Hardcoded component list that needs updating (line 38)
  
- file: engine/src/core/coordinates/world_transform.rs
  why: Large world coordinate system integration
  
- file: engine/src/app.rs
  why: Main update loop where physics system integrates
  
- file: engine/src/scripting/tests/
  why: Test patterns for component systems
```

### Current Codebase Structure
```bash
engine/
├── src/
│   ├── core/
│   │   ├── entity/
│   │   │   ├── components.rs      # Component definitions
│   │   │   └── hierarchy.rs       # Transform hierarchy
│   │   └── coordinates/           # Large world support
│   ├── graphics/                  # Rendering systems
│   ├── scripting/                 # Script system
│   └── app.rs                     # Main update loop
└── tests/
```

### Desired Codebase Structure
```bash
engine/
├── src/
│   ├── physics/                   # NEW: Physics module
│   │   ├── mod.rs                 # Module exports
│   │   ├── components.rs          # Rigidbody, Collider, PhysicsMaterial
│   │   ├── avbd_solver.rs         # AVBD algorithm implementation
│   │   ├── constraints.rs         # Constraint types and handling
│   │   ├── collision/             # Collision detection
│   │   │   ├── mod.rs            
│   │   │   ├── shapes.rs          # Sphere, Box, Capsule
│   │   │   ├── broad_phase.rs    # Spatial partitioning
│   │   │   └── narrow_phase.rs   # Contact generation
│   │   └── systems.rs             # Physics update system
│   └── lib.rs                     # Export physics module
└── tests/
    └── physics/                   # Physics tests
        ├── mod.rs
        ├── rigidbody_tests.rs
        ├── collision_tests.rs
        └── constraint_tests.rs
```

### Known Gotchas & Implementation Notes
```rust
// CRITICAL: AVBD parameters must be tuned correctly
// β = 10.0 (stiffness ramping speed)
// α = 0.95 (error correction factor)
// γ = 0.99 (warmstart decay)
// k_start > 0 (initial stiffness)

// GOTCHA: Quaternion operations need special handling
// Addition: q + 0.5 * omega * q (see paper eq. 20-21)
// Subtraction: 2 * (q1 * q2.conjugate()).xyz

// PATTERN: Use tracing for logging, never println!
use tracing::{debug, info, warn, error};

// CRITICAL: Components need registration in engine_derive
// Add to COMPONENT_NAMES in engine_derive/src/lib.rs:38

// GOTCHA: Contact constraints need special clamping
// Normal force: λ_min = 0, λ_max = ∞ (no pulling)
// Friction: ||λ_tangent|| ≤ μ * λ_normal

// PATTERN: Use LDLT decomposition for 6x6 linear systems
// More stable than direct inversion for mass/inertia matrices

// CRITICAL: Warm-starting between frames is essential
// Scale previous λ and k by γ = 0.99 each frame
```

## Implementation Blueprint

### Data Models and Structure

```rust
// Core physics components following engine patterns
#[derive(Component, EditorUI, Debug, Clone, Serialize, Deserialize)]
#[component(name = "Rigidbody")]
pub struct Rigidbody {
    #[ui(range = 0.1..1000.0, speed = 0.1, tooltip = "Mass in kg")]
    pub mass: f32,
    
    #[ui(range = 0.0..10.0, speed = 0.01, tooltip = "Linear damping")]
    pub linear_damping: f32,
    
    #[ui(range = 0.0..10.0, speed = 0.01, tooltip = "Angular damping")]
    pub angular_damping: f32,
    
    #[ui(tooltip = "Linear velocity")]
    pub linear_velocity: Vec3,
    
    #[ui(tooltip = "Angular velocity")]  
    pub angular_velocity: Vec3,
    
    #[ui(hidden)]
    pub inertia_tensor: Mat3,
    
    #[ui(tooltip = "Is affected by gravity")]
    pub use_gravity: bool,
    
    #[ui(tooltip = "Prevents all movement")]
    pub is_kinematic: bool,
}

#[derive(Component, EditorUI, Debug, Clone, Serialize, Deserialize)]
#[component(name = "Collider")]
pub struct Collider {
    #[ui(tooltip = "Collision shape type")]
    pub shape: CollisionShape,
    
    #[ui(tooltip = "Is trigger (no collision response)")]
    pub is_trigger: bool,
    
    #[ui(hidden)]
    pub material_id: Option<Entity>, // Reference to PhysicsMaterial
}

#[derive(Component, EditorUI, Debug, Clone, Serialize, Deserialize)]
#[component(name = "PhysicsMaterial")]
pub struct PhysicsMaterial {
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Static friction")]
    pub static_friction: f32,
    
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Dynamic friction")]
    pub dynamic_friction: f32,
    
    #[ui(range = 0.0..1.0, speed = 0.01, tooltip = "Restitution (bounciness)")]
    pub restitution: f32,
}

// AVBD solver data structures
pub struct AVBDSolver {
    constraints: Vec<Box<dyn Constraint>>,
    vertex_colors: VertexColoring,
    config: AVBDConfig,
}

pub struct AVBDConfig {
    pub iterations: u32,
    pub beta: f32,          // Stiffness ramping
    pub alpha: f32,         // Error correction  
    pub gamma: f32,         // Warmstart decay
    pub k_start: f32,       // Initial stiffness
}
```

### List of Tasks

```yaml
Task 1: Create Physics Module Structure
CREATE engine/src/physics/mod.rs:
  - Export all physics types and systems
  - Follow pattern from engine/src/graphics/mod.rs

CREATE engine/src/physics/components.rs:
  - Define Rigidbody, Collider, PhysicsMaterial components
  - Use derive macros: Component, EditorUI
  - Add UI annotations for inspector

MODIFY engine_derive/src/lib.rs:
  - Add "Rigidbody", "Collider", "PhysicsMaterial" to COMPONENT_NAMES
  - Maintain alphabetical order

MODIFY engine/src/io/component_registry.rs:
  - Register new physics components in with_default_components()
  - Follow existing registration pattern

Task 2: Implement Collision Shapes
CREATE engine/src/physics/collision/shapes.rs:
  - Define CollisionShape enum (Sphere, Box, Capsule)
  - Implement support functions (inertia calculation, bounds)
  - Use glam math types consistently

CREATE engine/src/physics/collision/mod.rs:
  - Export collision types
  - Define Contact struct with position, normal, penetration

Task 3: Implement AVBD Solver Core
CREATE engine/src/physics/avbd_solver.rs:
  - Implement AVBDSolver struct with config
  - Core solver loop with primal-dual updates
  - Vertex coloring for parallelization
  - Use rayon for parallel iteration

CREATE engine/src/physics/constraints.rs:
  - Define Constraint trait
  - Implement ContactConstraint with friction
  - Add BallJoint, HingeJoint, FixedJoint
  - Handle force clamping correctly

Task 4: Implement Collision Detection
CREATE engine/src/physics/collision/broad_phase.rs:
  - Simple AABB sweep-and-prune
  - Return potential collision pairs

CREATE engine/src/physics/collision/narrow_phase.rs:
  - Sphere-sphere, box-box collision tests
  - Generate contact points with normals
  - Use GJK/EPA or simple geometric tests

Task 5: Create Physics System
CREATE engine/src/physics/systems.rs:
  - Define update_physics_system() function
  - Query entities with Rigidbody + Transform
  - Run collision detection
  - Execute AVBD solver
  - Update Transform/WorldTransform components

MODIFY engine/src/app.rs:
  - Add physics system update before hierarchy update
  - Pass delta_time to physics system
  - Ensure correct update order

Task 6: Add Tests
CREATE engine/tests/physics/mod.rs:
  - Module setup for physics tests

CREATE engine/tests/physics/rigidbody_tests.rs:
  - Test gravity integration
  - Test velocity damping
  - Test kinematic bodies

CREATE engine/tests/physics/collision_tests.rs:
  - Test shape-shape collisions
  - Test contact generation
  - Test trigger behavior

CREATE engine/tests/physics/constraint_tests.rs:
  - Test joint stability
  - Test constraint solving
  - Test warmstarting

Task 7: Create Example Scenes
CREATE game/assets/scenes/physics_test.json:
  - Simple falling cubes scene
  - Demonstrate basic physics

CREATE game/assets/scenes/physics_stacking.json:
  - Stacking test with multiple boxes
  - Test stability under load

CREATE game/assets/scenes/physics_joints.json:
  - Connected bodies with joints
  - Demonstrate constraint types
```

### Per-Task Implementation Details

```rust
// Task 1: Component Setup
// Follow existing component patterns exactly
#[derive(Component, EditorUI, Debug, Clone, Serialize, Deserialize, Default)]
#[component(name = "Rigidbody")]
pub struct Rigidbody {
    // Implementation as shown above
}

// Task 3: AVBD Solver Core Algorithm
impl AVBDSolver {
    pub fn step(&mut self, bodies: &mut [RigidbodyData], dt: f32) {
        // 1. Compute inertial positions
        // y = x + dt*v + dt²*a_ext
        
        // 2. Initialize/warmstart dual variables
        // λ *= self.config.gamma
        // k = max(k * self.config.gamma, self.config.k_start)
        
        for _ in 0..self.config.iterations {
            // 3. Primal update (per color, parallel)
            for color in &self.vertex_colors.colors {
                bodies.par_iter_mut()
                    .filter(|b| b.color == *color)
                    .for_each(|body| {
                        let (f, H) = compute_forces_and_hessian(body, &self.constraints);
                        body.delta_x = ldlt_solve_6x6(H, f);
                    });
                
                // Apply position updates
                apply_updates(bodies);
            }
            
            // 4. Dual update (parallel)
            self.constraints.par_iter_mut().for_each(|constraint| {
                constraint.update_dual_variables(&bodies, self.config.beta);
            });
        }
        
        // 5. Update velocities
        // v = (x - x_old) / dt
    }
}

// Task 4: Collision Detection Pattern
pub fn detect_collisions(colliders: &[(Entity, &Collider, &GlobalTransform)]) -> Vec<Contact> {
    let mut contacts = Vec::new();
    
    // Broad phase
    let pairs = broad_phase_sweep_and_prune(colliders);
    
    // Narrow phase
    for (a, b) in pairs {
        if let Some(contact) = narrow_phase_test(
            &colliders[a].1.shape,
            &colliders[a].2,
            &colliders[b].1.shape, 
            &colliders[b].2
        ) {
            contacts.push(contact);
        }
    }
    
    contacts
}

// Task 5: System Integration
pub fn update_physics_system(
    world: &mut World,
    solver: &mut AVBDSolver,
    delta_time: f32,
) {
    // 1. Gather physics entities
    let mut bodies = query_rigidbodies(world);
    let colliders = query_colliders(world);
    
    // 2. Apply gravity and external forces
    apply_external_forces(&mut bodies, delta_time);
    
    // 3. Detect collisions
    let contacts = detect_collisions(&colliders);
    
    // 4. Create contact constraints
    solver.constraints.clear();
    for contact in contacts {
        solver.constraints.push(Box::new(ContactConstraint::new(contact)));
    }
    
    // 5. Run AVBD solver
    solver.step(&mut bodies, delta_time);
    
    // 6. Write back to components
    update_transforms(world, &bodies);
}
```

### Integration Points
```yaml
ENGINE:
  - file: engine/src/lib.rs
  - add: pub mod physics;
  - export: pub use physics::{Rigidbody, Collider, PhysicsMaterial};

APP_UPDATE:
  - file: engine/src/app.rs  
  - add: use crate::physics::systems::update_physics_system;
  - location: After script execution, before hierarchy update
  - pattern: |
      // Physics simulation
      update_physics_system(&mut self.world, &mut self.physics_solver, dt);
      
      // Update transform hierarchy
      update_hierarchy_system(&mut self.world);

COMPONENT_DERIVE:
  - file: engine_derive/src/lib.rs
  - location: COMPONENT_NAMES constant (line 38)
  - add: "Rigidbody", "Collider", "PhysicsMaterial"

COMPONENT_REGISTRY:
  - file: engine/src/io/component_registry.rs
  - location: with_default_components() method
  - add: |
      Rigidbody::register(&mut registry);
      Collider::register(&mut registry);  
      PhysicsMaterial::register(&mut registry);
```

## Validation Loop

### Level 1: Compilation and Lints
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all                    # Format code
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace            # Quick compilation check

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```rust
// Test basic rigidbody behavior
#[test]
fn test_rigidbody_gravity() {
    let mut world = World::new();
    let entity = world.spawn((
        Transform::default(),
        Rigidbody {
            mass: 1.0,
            use_gravity: true,
            ..Default::default()
        },
    ));
    
    // Step physics
    update_physics_system(&mut world, &mut solver, 0.016);
    
    // Check velocity increased downward
    let rb = world.get::<Rigidbody>(entity).unwrap();
    assert!(rb.linear_velocity.y < 0.0);
}

#[test]
fn test_collision_detection() {
    // Create two overlapping spheres
    let contacts = detect_collisions(&[
        (entity_a, &sphere_collider, &transform_a),
        (entity_b, &sphere_collider, &transform_b),
    ]);
    
    assert_eq!(contacts.len(), 1);
    assert!(contacts[0].penetration > 0.0);
}

#[test]
fn test_avbd_convergence() {
    // Test solver converges for stiff constraints
    let mut solver = AVBDSolver::new(AVBDConfig {
        iterations: 5,
        beta: 10.0,
        alpha: 0.95,
        gamma: 0.99,
        k_start: 100.0,
    });
    
    // Run solver
    solver.step(&mut bodies, 0.016);
    
    // Check constraints are satisfied
    for constraint in &solver.constraints {
        assert!(constraint.evaluate(&bodies).abs() < 0.01);
    }
}
```

### Level 3: Integration Tests
```bash
# Run all tests
cargo test --workspace

# Run physics tests specifically  
cargo test --package engine physics

# Run with logging to debug
RUST_LOG=debug cargo test --package engine physics -- --nocapture
```

### Level 4: Visual Testing
```bash
# Run the editor with physics test scene
just run -- --scene game/assets/scenes/physics_test.json

# Expected behavior:
# - Cubes fall under gravity
# - Collisions prevent interpenetration
# - Stacking remains stable
# - Joints maintain constraints
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Documentation builds: `cargo doc --workspace --no-deps`
- [ ] Physics scenes run at 60 FPS with 1000 bodies
- [ ] Editor can create/modify physics components
- [ ] Rigidbodies integrate with transform hierarchy
- [ ] Constraints remain stable over time
- [ ] Large world coordinates work correctly

---

## Anti-Patterns to Avoid
- ❌ Don't use println! - use tracing macros
- ❌ Don't forget to register components in derive macro
- ❌ Don't skip warmstarting - it's critical for stability
- ❌ Don't use direct matrix inversion - use LDLT
- ❌ Don't ignore quaternion special operations
- ❌ Don't create new patterns - follow existing ones
- ❌ Don't skip tests - physics needs validation

## Confidence Score: 8/10

The PRP provides comprehensive context including:
- Complete algorithm details from papers
- Existing codebase patterns to follow
- Specific implementation guidance
- Clear integration points
- Extensive validation approach

Points deducted for:
- Collision detection algorithms not fully specified (GJK/EPA vs simple)
- GPU compute shader implementation deferred
- Some complex constraint types (motors, limits) not detailed