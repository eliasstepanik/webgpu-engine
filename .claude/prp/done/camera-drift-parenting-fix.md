name: "Fix Camera Drift When Parenting Entities"
description: |

## Purpose
Fix the camera-specific drift issue that occurs when parenting camera entities to other entities in the hierarchy system, despite mathematically correct transform calculations.

## Core Principles
1. **Precision Preservation**: Minimize floating-point errors in transform calculations
2. **Single Update Principle**: Ensure hierarchy updates happen exactly once per frame
3. **Camera-Aware Handling**: Recognize cameras need special treatment due to view matrix inversion
4. **Validation Through Testing**: Add comprehensive tests to prevent regression
5. **Global rules**: Follow all rules in CLAUDE.md, especially regarding logging with `tracing`

---

## Goal
Eliminate camera position drift when camera entities are parented to other entities, ensuring cameras maintain their exact world position after parenting operations.

## Why
- **User Experience**: Camera drift is highly noticeable and disrupts gameplay/editing experience
- **System Integrity**: Current debug output shows correct math but visual drift still occurs
- **Feature Parity**: Regular entities parent correctly; cameras should behave the same way
- **Large World Support**: Critical for games using camera-relative rendering at extreme scales

## What
When a camera entity is drag-dropped onto another entity in the hierarchy panel to make it a child, the camera should maintain its exact world position without any drift or "jumping". The local transform should be adjusted to compensate for the parent's transform, and this adjustment should be stable across frames.

### Success Criteria
- [ ] Camera maintains exact world position after parenting (within 0.0001 units tolerance)
- [ ] No visual drift or jumping when camera is parented/unparented
- [ ] Hierarchy updates occur exactly once per frame
- [ ] Solution works with both Transform and WorldTransform components
- [ ] All existing hierarchy tests continue to pass
- [ ] New tests verify camera-specific parenting behavior

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html
  why: Understanding floating-point arithmetic and precision loss

- url: https://floating-point-gui.de/errors/propagation/
  why: How errors propagate through multiple operations like matrix multiplication

- url: https://www.scratchapixel.com/lessons/mathematics-physics-for-computer-graphics/geometry/row-major-vs-column-major-order
  why: Matrix operation order and precision considerations

- file: engine/src/core/entity/hierarchy.rs
  why: Current hierarchy update system - needs modification to prevent double updates

- file: editor/src/panels/hierarchy.rs
  why: Drag-drop parenting code with existing camera debug output

- file: engine/src/core/camera.rs
  why: Camera view matrix calculation - understand why cameras are special

- file: engine/src/graphics/renderer.rs
  why: Camera-relative rendering system that might amplify small errors

- file: game/src/main.rs (line 342)
  why: Shows where hierarchy update is called in game loop

- file: engine/src/app.rs (line 229)
  why: Shows where hierarchy update is called in engine update
```

### Current Debug Output Analysis
```
=== CAMERA PARENTING DEBUG ===
Old local pos: Vec3(0.0, 2.0, 5.0)
New local pos: Vec3(0.0, 3.0, 5.0)
Child world matrix: Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 0.9284767, -0.3713906, 0.0), z_axis: Vec4(0.0, 0.3713906, 0.9284767, 0.0), w_axis: Vec4(0.0, 2.0, 5.0, 1.0) }
Parent world matrix: Mat4 { x_axis: Vec4(1.0, 0.0, 0.0, 0.0), y_axis: Vec4(0.0, 1.0, 0.0, 0.0), z_axis: Vec4(0.0, 0.0, 1.0, 0.0), w_axis: Vec4(0.0, -1.0, 0.0, 1.0) }
After hierarchy update - Camera world pos: Vec3(0.0, 2.0, 5.0)
Expected world pos was: Some(Vec3(0.0, 2.0, 5.0))
```

Math is correct but visual drift still occurs!

### Known Gotchas & Critical Insights
```rust
// CRITICAL: Hierarchy system is called multiple times per frame:
// 1. editor/src/panels/hierarchy.rs after parenting (lines 424, 598)
// 2. game/src/main.rs in update loop (line 342)  
// 3. engine/src/app.rs in engine update (line 229)
// Each call recalculates transforms, accumulating floating-point errors

// CRITICAL: Camera view matrix is INVERSE of world transform
// Small errors in position are magnified when inverted for view matrix

// CRITICAL: Matrix decomposition loses precision
// to_scale_rotation_translation() and from_scale_rotation_translation()
// are not perfectly reversible due to floating-point representation

// CRITICAL: Must use tracing crate for logging, never println!
use tracing::{debug, error, info, warn, trace};
```

## Implementation Blueprint

### Core Issue: Multiple Hierarchy Updates Per Frame
The hierarchy system is being called 3 times per frame when parenting occurs:
1. Immediately after parenting in editor
2. In the game's main update loop
3. In the engine's update method

Each update recalculates global transforms, and small floating-point errors accumulate.

### Solution Architecture
1. **Add frame-based update tracking** to prevent multiple hierarchy updates
2. **Store exact world position** before/after parenting for validation
3. **Use higher precision** for camera transform calculations
4. **Add epsilon-based comparison** for transform changes

### List of Tasks

```yaml
Task 1: Add Hierarchy Update Tracking
MODIFY engine/src/core/entity/hierarchy.rs:
  - ADD field to track last update frame/time
  - ADD method to check if update needed this frame
  - MODIFY update_hierarchy_system to skip if already updated

Task 2: Improve Camera Transform Precision
MODIFY editor/src/panels/hierarchy.rs:
  - STORE exact world position before any operations
  - USE f64 precision for intermediate calculations
  - VALIDATE world position after operations
  - ADD warning if drift detected

Task 3: Add Camera-Specific Tests
CREATE engine/src/core/entity/hierarchy.rs (test module):
  - ADD test_camera_parenting_no_drift
  - ADD test_camera_hierarchy_precision
  - ADD test_multiple_update_prevention

Task 4: Fix Transform Decomposition Precision
MODIFY engine/src/core/entity/components.rs:
  - ADD epsilon tolerance to transform comparisons
  - IMPROVE GlobalTransform::to_camera_relative precision

Task 5: Optimize Update Order
MODIFY game/src/main.rs and engine/src/app.rs:
  - COORDINATE hierarchy updates to happen once
  - ADD debug tracking for update frequency
```

### Task 1: Add Hierarchy Update Tracking
```rust
// In engine/src/core/entity/hierarchy.rs

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// Add at module level
static LAST_HIERARCHY_UPDATE_FRAME: AtomicU64 = AtomicU64::new(0);
static CURRENT_FRAME: AtomicU64 = AtomicU64::new(0);

pub fn advance_frame() {
    CURRENT_FRAME.fetch_add(1, Ordering::SeqCst);
}

pub fn update_hierarchy_system(world: &mut World) {
    let current_frame = CURRENT_FRAME.load(Ordering::SeqCst);
    let last_update = LAST_HIERARCHY_UPDATE_FRAME.load(Ordering::SeqCst);
    
    if current_frame == last_update {
        trace!("Skipping hierarchy update - already updated this frame");
        return;
    }
    
    LAST_HIERARCHY_UPDATE_FRAME.store(current_frame, Ordering::SeqCst);
    
    // Existing update logic...
    update_regular_hierarchy(world);
    update_world_hierarchy(world);
}
```

### Task 2: Improve Camera Transform Precision
```rust
// In editor/src/panels/hierarchy.rs

// Store world position with full precision
let original_world_pos = child_world_matrix
    .map(|m| {
        let (_, _, translation) = m.to_scale_rotation_translation();
        glam::DVec3::new(
            translation.x as f64,
            translation.y as f64,
            translation.z as f64
        )
    });

// After all operations, validate position hasn't drifted
if let (Some(original_pos), Some(final_matrix)) = (original_world_pos, final_world_matrix) {
    let final_pos = // extract position from final_matrix
    let drift = (final_pos - original_pos).length();
    
    if drift > 0.0001 {
        warn!(
            entity = ?dragged,
            drift = drift,
            "Camera position drifted after parenting"
        );
        
        // Force correction if needed
        if is_camera && drift > 0.001 {
            // Reconstruct transform with exact position
        }
    }
}
```

### Task 3: Camera-Specific Tests
```rust
#[cfg(test)]
mod camera_tests {
    use super::*;
    
    #[test]
    fn test_camera_parenting_no_drift() {
        let mut world = World::new();
        
        // Create camera with specific position and rotation
        let camera_pos = Vec3::new(10.5, 25.3, -15.7);
        let camera_rot = Quat::from_rotation_y(0.7854); // 45 degrees
        
        let camera = world.spawn((
            Transform::from_position_rotation(camera_pos, camera_rot),
            GlobalTransform::default(),
            Camera::default(),
        ));
        
        // Create parent at different position
        let parent = world.spawn((
            Transform::from_position(Vec3::new(-5.0, 10.0, 20.0)),
            GlobalTransform::default(),
        ));
        
        // Update hierarchy to establish initial state
        update_hierarchy_system(&mut world);
        
        // Store original world position
        let original_world_pos = world.get::<GlobalTransform>(camera)
            .unwrap()
            .position();
        
        // Parent the camera
        world.insert_one(camera, Parent(parent)).unwrap();
        
        // Update hierarchy
        update_hierarchy_system(&mut world);
        
        // Verify world position hasn't changed
        let new_world_pos = world.get::<GlobalTransform>(camera)
            .unwrap()
            .position();
        
        assert!((new_world_pos - original_world_pos).length() < 0.0001,
            "Camera drifted by {} units", (new_world_pos - original_world_pos).length());
    }
    
    #[test]
    fn test_multiple_hierarchy_updates_no_accumulation() {
        // Test that calling update_hierarchy_system multiple times
        // doesn't accumulate errors
    }
}
```

## Validation Loop

### Level 1: Syntax & Formatting
```bash
# Fix formatting issues
cargo fmt --all

# Check for common issues
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Unit Tests
```bash
# Run hierarchy tests specifically
cargo test --package engine hierarchy

# Run new camera parenting tests
cargo test --package engine camera_parenting

# Expected: All tests pass
```

### Level 3: Integration Test
```bash
# Run the game with debug logging
RUST_LOG=engine::core::entity::hierarchy=debug cargo run

# Test camera parenting:
# 1. Create a scene with camera and cube
# 2. Select camera in hierarchy panel
# 3. Drag camera onto cube to parent it
# 4. Check console for drift warnings
# 5. Verify camera view doesn't jump

# Expected: No "Camera position drifted" warnings
```

### Level 4: Performance Validation
```bash
# Verify hierarchy updates happen once per frame
RUST_LOG=engine::core::entity::hierarchy=trace cargo run

# Expected: Should see "Skipping hierarchy update" messages
# when multiple systems try to update in same frame
```

## Final Validation Checklist
- [ ] All existing tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --all`
- [ ] Camera parenting shows no visual drift
- [ ] Debug output shows no drift warnings
- [ ] Hierarchy updates occur exactly once per frame
- [ ] Performance is not impacted (measure frame times)
- [ ] Solution works at extreme scales (test at 1 million units from origin)

## Anti-Patterns to Avoid
- ❌ Don't use println! for debugging - use tracing crate
- ❌ Don't skip the frame tracking - multiple updates cause drift
- ❌ Don't ignore small drift values - they accumulate over time
- ❌ Don't modify camera behavior for non-parenting operations
- ❌ Don't break existing entity parenting functionality
- ❌ Don't use f32 for world-scale position calculations

---

## Implementation Notes

The root cause is multiple hierarchy updates per frame combined with floating-point precision loss during matrix operations. Cameras are especially sensitive because:

1. Their view matrix is the inverse of their world transform
2. Small position errors are visually magnified
3. The camera-relative rendering system depends on precise camera positions

By ensuring single updates per frame and using higher precision for critical calculations, we can eliminate the drift while maintaining performance.

## Confidence Score: 8/10

High confidence because:
- Root cause is clearly identified (multiple updates)
- Solution is straightforward (frame tracking)
- Similar patterns exist in other engines
- Debug infrastructure already in place

Deductions for:
- Complex matrix math might have edge cases
- Testing at extreme scales needs careful validation