name: "Fix Physics Scene Configuration - Collision Detection"
description: |

## Purpose
Fix physics collision detection failures caused by improper scene configuration where objects are positioned with large gaps preventing any physical overlap. The collision detection system is working correctly, but scene files have geometry that prevents objects from ever colliding.

## Core Principles
1. **Context is King**: Document all physics scene requirements and validation rules
2. **Validation Loops**: Add scene validation to catch configuration errors early
3. **Information Dense**: Include working examples and anti-patterns
4. **Progressive Success**: Fix existing scenes, then add validation
5. **Global rules**: Follow all rules in CLAUDE.md

---

## Goal
Ensure physics scenes are configured correctly so that objects can actually collide, with proper validation to prevent future configuration errors. Create scene validation tools and update all physics test scenes.

## Why
- **Business value**: Physics simulation is a core engine feature; broken physics scenes create poor user experience
- **Integration**: Physics affects gameplay, editor workflow, and demo quality
- **Problems solved**: Prevents "physics not working" bug reports when it's actually scene configuration

## What
Fix scene configurations where floor and falling objects have no physical overlap, add scene validation, and provide clear documentation.

### Success Criteria
- [ ] All physics test scenes have correct geometry placement
- [ ] Scene validator detects and reports configuration issues
- [ ] Physics demo scenes show proper collision behavior
- [ ] Documentation explains proper physics scene setup

## All Needed Context

### Documentation & References
```yaml
- file: /engine/src/physics/mod.rs
  why: Physics config shows gravity=-9.81, timestep=1/120s, max_velocity=100
  
- file: /game/assets/scenes/physics_debug_test.json
  why: Floor at Y=-1.0 with scale 0.2 puts top at Y=-0.9, objects start at Y=5.0
  critical: 5.4 unit gap prevents any collision!

- file: /game/assets/scenes/physics_working_test.json  
  why: Working example - floor at Y=-0.5 with scale Y=1.0, top at Y=0.0
  pattern: Proper floor placement for collision

- file: /engine/src/physics/systems.rs
  why: Shows how GlobalTransform scale is applied to collision shapes
  line: 210 - scale_collider applies transform scale correctly

- url: https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection
  section: AABB overlap testing
  critical: Objects must have overlapping bounding boxes to collide

- file: /engine/src/bin/debug_collision_test.rs
  why: Standalone test showing collision detection works when objects overlap
```

### Current Codebase tree
```bash
engine/
├── src/
│   ├── physics/
│   │   ├── collision/
│   │   │   ├── mod.rs          # AABB overlap test
│   │   │   ├── shapes.rs       # world_aabb calculation
│   │   │   └── narrow_phase.rs # Contact generation
│   │   ├── systems.rs          # scale_collider function
│   │   └── mod.rs              # PhysicsConfig (gravity=-9.81)
│   └── bin/
│       └── debug_collision_test.rs # Collision testing tool
game/
└── assets/
    └── scenes/
        ├── physics_debug_test.json     # BROKEN: floor too low
        ├── physics_working_test.json   # WORKING: correct placement
        └── [other physics scenes]      # Need validation
```

### Desired Codebase tree with files to be added
```bash
engine/
├── src/
│   ├── physics/
│   │   ├── scene_validator.rs  # NEW: Validate physics scene configuration
│   │   └── mod.rs              # Export validator
│   └── bin/
│       └── validate_physics_scene.rs # NEW: CLI tool for scene validation
game/
└── assets/
    └── scenes/
        ├── physics_debug_test.json     # FIXED: proper floor placement
        ├── physics_scene_template.json # NEW: Template for physics scenes
        └── README_PHYSICS_SCENES.md    # NEW: Documentation
```

### Known Gotchas
```rust
// CRITICAL: Scale is applied to collision shapes!
// Floor with scale Y=0.2 and half_extents Y=0.5 → actual height = 0.1
// This creates very thin floors that objects can miss

// CRITICAL: With gravity=-9.81 and 120Hz physics:
// Objects fall 9.81/120 = 0.08175 units per step initially
// Need ~66 steps (0.55s) to fall 5.4 units with acceleration

// CRITICAL: GlobalTransform required for physics
// Hierarchy system must run before physics to create GlobalTransform
```

## Implementation Blueprint

### Data models and structure

```rust
// Scene validation result
#[derive(Debug)]
pub struct SceneValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub entity_name: String,
    pub error_type: ErrorType,
    pub details: String,
}

#[derive(Debug)]
pub enum ErrorType {
    NoOverlap { gap_distance: f32 },
    MissingCollider,
    InvalidScale { scale: Vec3 },
    FloatingObject { height: f32 },
}

// Scene analysis data
#[derive(Debug)]
pub struct PhysicsSceneAnalysis {
    pub static_colliders: Vec<ColliderInfo>,
    pub dynamic_bodies: Vec<RigidbodyInfo>,
    pub potential_collisions: Vec<(String, String, f32)>, // entity1, entity2, distance
}
```

### List of tasks

```yaml
Task 1: Create physics scene validator module
MODIFY engine/src/physics/mod.rs:
  - FIND pattern: "pub mod systems;"
  - INJECT after: "pub mod scene_validator;"
  - ADD export: "pub use scene_validator::validate_physics_scene;"

CREATE engine/src/physics/scene_validator.rs:
  - MIRROR pattern from: engine/src/physics/debug_visualization.rs (structure)
  - IMPLEMENT: Scene loading and validation logic
  - VALIDATE: Check for floating objects, collision gaps, scale issues

Task 2: Fix physics_debug_test.json scene configuration  
MODIFY game/assets/scenes/physics_debug_test.json:
  - FIND floor entity with position Y=-1.0
  - CHANGE position to [0.0, -0.5, 0.0]
  - CHANGE scale to [20.0, 1.0, 20.0]
  - PRESERVE all other properties

Task 3: Create CLI validation tool
CREATE engine/src/bin/validate_physics_scene.rs:
  - MIRROR pattern from: engine/src/bin/debug_collision_test.rs
  - USE: scene_validator module
  - OUTPUT: Detailed validation report

Task 4: Add validation to physics system startup
MODIFY engine/src/physics/systems.rs:
  - FIND pattern: "pub fn update_physics_system"
  - INJECT validation check at start (debug builds only)
  - LOG warnings for scene configuration issues

Task 5: Create physics scene template
CREATE game/assets/scenes/physics_scene_template.json:
  - INCLUDE: Properly positioned floor
  - INCLUDE: Example dynamic objects
  - INCLUDE: Comments explaining positioning

Task 6: Document physics scene requirements
CREATE game/assets/scenes/README_PHYSICS_SCENES.md:
  - EXPLAIN: Collision shape scaling
  - PROVIDE: Positioning guidelines
  - INCLUDE: Common pitfalls and solutions
```

### Per task pseudocode

```rust
// Task 1: Scene validator core logic
pub fn validate_physics_scene(scene: &Scene) -> SceneValidationResult {
    let mut result = SceneValidationResult::default();
    
    // Extract colliders and rigidbodies with transforms
    let static_colliders = collect_static_colliders(scene);
    let dynamic_bodies = collect_dynamic_bodies(scene);
    
    // Check 1: Floating objects
    for body in &dynamic_bodies {
        let bottom_y = body.position.y - body.scaled_half_extents.y;
        let nearest_floor = find_nearest_floor(&static_colliders, body.position);
        
        if let Some((floor, distance)) = nearest_floor {
            if distance > 2.0 {  // More than 2 units gap
                result.warnings.push(ValidationWarning {
                    entity: body.name.clone(),
                    warning: format!("Object starts {:.1}m above nearest floor", distance),
                });
            }
        } else {
            result.errors.push(ValidationError {
                entity_name: body.name.clone(),
                error_type: ErrorType::FloatingObject { height: bottom_y },
                details: "No floor found below object".to_string(),
            });
        }
    }
    
    // Check 2: Floor configuration
    for floor in &static_colliders {
        let actual_height = floor.half_extents.y * floor.scale.y;
        if actual_height < 0.05 {  // Less than 5cm thick
            result.warnings.push(ValidationWarning {
                entity: floor.name.clone(),
                warning: format!("Floor only {:.3}m thick - may cause tunneling", actual_height),
            });
        }
    }
    
    // Check 3: Initial overlaps
    for (i, body) in dynamic_bodies.iter().enumerate() {
        let body_aabb = compute_aabb(body);
        
        for floor in &static_colliders {
            let floor_aabb = compute_aabb(floor);
            if body_aabb.overlaps(&floor_aabb) {
                result.errors.push(ValidationError {
                    entity_name: body.name.clone(),
                    error_type: ErrorType::InitialPenetration,
                    details: format!("Overlaps with {} at start", floor.name),
                });
            }
        }
    }
    
    result
}

// Task 3: CLI tool
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <scene.json>", args[0]);
        return;
    }
    
    // Load scene
    let scene = load_scene(&args[1]).expect("Failed to load scene");
    
    // Validate
    let result = validate_physics_scene(&scene);
    
    // Report
    println!("=== Physics Scene Validation Report ===");
    println!("Scene: {}", args[1]);
    println!("Valid: {}", result.is_valid);
    
    if !result.errors.is_empty() {
        println!("\nERRORS:");
        for error in &result.errors {
            println!("  - {}: {}", error.entity_name, error.details);
        }
    }
    
    if !result.warnings.is_empty() {
        println!("\nWARNINGS:");
        for warning in &result.warnings {
            println!("  - {}: {}", warning.entity, warning.warning);
        }
    }
    
    if !result.suggestions.is_empty() {
        println!("\nSUGGESTIONS:");
        for suggestion in &result.suggestions {
            println!("  - {}", suggestion);
        }
    }
}
```

### Integration Points
```yaml
BUILD:
  - add to: Cargo.toml
  - binary: "[[bin]] name = 'validate_physics_scene'"
  
PHYSICS:
  - modify: engine/src/physics/mod.rs
  - export: scene_validator module
  
CI:
  - add to: .github/workflows/test.yml  
  - step: "Validate physics scenes"
  - command: "cargo run --bin validate_physics_scene game/assets/scenes/physics_*.json"
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Check new validator module
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings

# Expected: No errors
```

### Level 2: Unit Tests
```rust
// CREATE engine/src/physics/scene_validator.rs tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_floating_objects() {
        let scene = create_test_scene_with_gap();
        let result = validate_physics_scene(&scene);
        
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        matches!(result.errors[0].error_type, ErrorType::NoOverlap { .. });
    }
    
    #[test]
    fn test_valid_scene() {
        let scene = create_valid_test_scene();
        let result = validate_physics_scene(&scene);
        
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }
    
    #[test]
    fn test_thin_floor_warning() {
        let scene = create_scene_with_thin_floor();
        let result = validate_physics_scene(&scene);
        
        assert!(result.warnings.iter().any(|w| w.warning.contains("thin")));
    }
}
```

### Level 3: Integration Test
```bash
# Test the validator on actual scenes
cargo build --bin validate_physics_scene
./target/debug/validate_physics_scene game/assets/scenes/physics_debug_test.json

# Expected output should show configuration issues

# After fixing scenes:
./target/debug/validate_physics_scene game/assets/scenes/physics_debug_test.json
# Expected: "Valid: true"

# Run the game with fixed scene
just run
# Select physics_debug_test scene
# Verify: Objects should collide with floor properly
```

## Final validation Checklist
- [ ] All physics test scenes validate successfully
- [ ] Validator detects common configuration errors
- [ ] Fixed scenes show proper collision behavior
- [ ] No regression in working physics scenes
- [ ] Documentation clearly explains requirements
- [ ] CI validates all physics scenes
- [ ] Template scene works out of the box

---

## Anti-Patterns to Avoid
- ❌ Don't place floors too low with small scale factors
- ❌ Don't assume collision detection is broken - check scene first
- ❌ Don't create paper-thin collision geometry  
- ❌ Don't start dynamic objects inside static geometry
- ❌ Don't forget that scale affects collision shapes
- ❌ Don't skip scene validation in CI