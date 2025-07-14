name: "Fix Color Pulse Material Updates"
description: |

## Purpose
Fix the color pulse and rotating cube scripts by properly implementing material color modifications. The scripts were attempting to modify material colors through array indexing, but the Material type registration in Rhai only provided a getter without a setter, causing modifications to be lost.

## Core Principles
1. **Proper Type Registration**: Ensure Rhai types have both getters and setters when modification is needed
2. **Clear API Design**: Provide convenient methods for common operations
3. **Backward Compatibility**: Maintain existing script interfaces while fixing the issue
4. **Thorough Testing**: Verify all material modification patterns work correctly

---

## Goal
Enable Rhai scripts to successfully modify material colors at runtime by:
- Adding proper setters to the Material type registration
- Providing convenient color modification methods
- Updating affected scripts to use correct patterns
- Creating test scripts to verify functionality

## Why
- **Visual Feedback**: Dynamic material changes are essential for visual effects and game feedback
- **Script Capability**: Material modification is a core feature that many scripts need
- **Developer Experience**: The current behavior is confusing - scripts appear to work but have no visual effect
- **Feature Completeness**: Color pulsing and tinting are common game effects that should work out of the box

## What
Fix the Material type registration to support color modifications and update all affected scripts to use the correct patterns for material updates.

### Success Criteria
- [ ] Color pulse script successfully animates material colors
- [ ] Rotating cube script correctly applies tint colors
- [ ] Material modifications persist and are visible in the renderer
- [ ] All material modification patterns are documented
- [ ] Test scripts demonstrate various material update techniques

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://rhai.rs/book/rust/custom-types.html#getters-setters-and-indexers
  why: Understanding how to register getters and setters for custom types in Rhai
  critical: Shows the register_get_set pattern needed for mutable properties
  
- url: https://rhai.rs/book/rust/register-raw.html#fallible-getters-setters-and-indexers
  why: Advanced registration patterns for error handling in setters
  
- file: engine/src/scripting/modules/world.rs
  why: Current Material type registration showing only getter, no setter
  critical: Lines 289-314 show the registration that needs modification
  
- file: engine/src/graphics/material.rs
  why: Material struct definition with color: [f32; 4] field
  
- file: game/assets/scripts/color_pulse.rhai
  why: Script attempting to modify colors that currently fails
  critical: Lines 41-44 show the array modification pattern
  
- file: game/assets/scripts/rotating_cube.rhai
  why: Another script with material tinting that needs fixing
  critical: Lines 76-79 show the same failing pattern
  
- file: engine/src/scripting/property_types.rs
  why: Shows how color properties are converted to Rhai maps
  critical: PropertyValue::Color to Dynamic conversion uses "r","g","b","a" keys
```

### Current Implementation Analysis
```rust
// PROBLEM: Current Material registration (world.rs:289-299)
engine
    .register_type_with_name::<Material>("Material")
    .register_get("color", |m: &mut Material| {
        vec![
            Dynamic::from(m.color[0] as f64),
            Dynamic::from(m.color[1] as f64),
            Dynamic::from(m.color[2] as f64),
            Dynamic::from(m.color[3] as f64),
        ]
    })
    .register_fn("clone", |m: &mut Material| *m);

// ISSUE: No register_set for color property!
// Scripts can read material.color but modifications to the array are lost
```

### Script Pattern Analysis
```rhai
// Current pattern that FAILS:
let material = world::get_component(entity, "Material");
material.color[0] = new_value;  // Modifies temporary array copy
material.color[1] = new_value;  // These changes are lost!
world::set_component(entity, "Material", material);

// Why it fails: material.color returns a NEW array each time
// Modifications don't affect the actual material struct
```

### Known Gotchas & Requirements
```rust
// CRITICAL: Rhai array elements are Dynamic, not f64
// Setter must handle Dynamic array and convert to [f32; 4]

// CRITICAL: Color property values come as maps with "r","g","b","a" keys
// Example: base_color["r"] not base_color[0]

// GOTCHA: Material uses f32 internally but Rhai uses f64
// All numeric conversions must handle this

// PATTERN: Scripts expect 0.0-1.0 color range
// No clamping in Material struct, so setter should validate
```

## Implementation Blueprint

### Material Type Registration Enhancement
```rust
// Task 1: Add setter for color property
.register_get_set(
    "color",
    |m: &mut Material| -> Vec<Dynamic> {
        // Existing getter code
    },
    |m: &mut Material, color: Dynamic| -> Result<(), Box<EvalAltResult>> {
        // Convert Dynamic array to [f32; 4]
        // Handle both array and map formats
        // Validate and clamp values
    }
)

// Task 2: Add convenience methods
.register_fn("set_color", |m: &mut Material, r: f64, g: f64, b: f64, a: f64| {
    // Direct color setting method
})
.register_fn("set_rgb", |m: &mut Material, r: f64, g: f64, b: f64| {
    // RGB with alpha=1.0
})
```

### List of Tasks

```yaml
Task 1 - Enhance Material Type Registration:
MODIFY engine/src/scripting/modules/world.rs:
  - FIND: register_get("color", |m: &mut Material| {
  - REPLACE: with register_get_set that includes setter
  - IMPLEMENT: Dynamic array to [f32; 4] conversion
  - HANDLE: Both Vec<Dynamic> and rhai::Array types
  - VALIDATE: Color values between 0.0-1.0

Task 2 - Add Material Helper Methods:
MODIFY engine/src/scripting/modules/world.rs:
  - ADD: register_fn("set_color", ...) for direct RGBA setting
  - ADD: register_fn("set_rgb", ...) for RGB with alpha=1.0
  - PATTERN: Follow existing Material constructor patterns

Task 3 - Create Test Script:
CREATE game/assets/scripts/material_color_test.rhai:
  - TEST: Array modification pattern
  - TEST: set_color method pattern
  - TEST: Color property interpolation
  - VERIFY: Changes are visible

Task 4 - Update Color Pulse Script:
VERIFY game/assets/scripts/color_pulse.rhai:
  - CHECK: If current pattern now works with setter
  - OR: Update to use set_color method if cleaner
  - TEST: Smooth color pulsing animation

Task 5 - Update Rotating Cube Script:
MODIFY game/assets/scripts/rotating_cube.rhai:
  - FIND: Lines 76-79 material color assignment
  - UPDATE: Use working pattern (array or set_color)
  - TEST: Tint color properly applied

Task 6 - Create Test Scenes:
CREATE game/assets/scenes/test_material_updates.json:
  - INCLUDE: Entity with material_color_test script
  - INCLUDE: Entity with fixed color_pulse script
  - INCLUDE: Entity with fixed rotating_cube script
  - PURPOSE: Easy testing of all patterns

Task 7 - Document Material Patterns:
CREATE .claude/documentation/material-scripting-guide.md:
  - DOCUMENT: All ways to modify materials from scripts
  - INCLUDE: Code examples for each pattern
  - EXPLAIN: When to use each approach
  - WARN: About common pitfalls
```

### Per Task Implementation Details

```rust
// Task 1 - Setter Implementation
|m: &mut Material, color: Dynamic| -> Result<(), Box<EvalAltResult>> {
    // Handle array input
    if let Ok(array) = color.into_typed_array::<f64>() {
        if array.len() >= 3 {
            m.color[0] = array[0].clamp(0.0, 1.0) as f32;
            m.color[1] = array[1].clamp(0.0, 1.0) as f32;
            m.color[2] = array[2].clamp(0.0, 1.0) as f32;
            m.color[3] = if array.len() > 3 { array[3].clamp(0.0, 1.0) as f32 } else { 1.0 };
            return Ok(());
        }
    }
    
    // Handle Vec<Dynamic> from getter
    if let Ok(vec) = color.into_typed_array::<Dynamic>() {
        if vec.len() >= 3 {
            m.color[0] = vec[0].as_float()?.clamp(0.0, 1.0) as f32;
            m.color[1] = vec[1].as_float()?.clamp(0.0, 1.0) as f32;
            m.color[2] = vec[2].as_float()?.clamp(0.0, 1.0) as f32;
            m.color[3] = if vec.len() > 3 { vec[3].as_float()?.clamp(0.0, 1.0) as f32 } else { 1.0 };
            return Ok(());
        }
    }
    
    Err("color must be an array with at least 3 elements".into())
}
```

### Integration Points
```yaml
SCRIPTING:
  - location: engine/src/scripting/modules/world.rs
  - function: register_material_type()
  - ensure: Backward compatibility with existing scripts
  
TESTING:
  - manual: Run test scenes and verify visual changes
  - automated: Could add script tests if framework exists
  
EXAMPLES:
  - update: Any example scripts using materials
  - document: New patterns in comments
```

## Validation Loop

### Level 1: Compilation and Formatting
```bash
# Ensure Rust code compiles and is formatted
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors or warnings
```

### Level 2: Script Syntax Validation
```bash
# Test script loading (if the engine has a script validator)
cargo run --bin game -- --validate-scripts

# Or manually load test scene
RUST_LOG=engine::scripting=debug cargo run --bin game

# Expected: All scripts load without syntax errors
```

### Level 3: Visual Validation
```bash
# Run the test scene with debug logging
RUST_LOG=engine::scripting=debug cargo run --bin game

# Load test scene: game/assets/scenes/test_material_updates.json
# Expected:
# - Color pulse entity smoothly transitions between white and red
# - Rotating cube shows proper tint color
# - Debug logs show material updates being applied
# - No script errors in console
```

### Level 4: Pattern Testing
```rhai
// Test all material modification patterns work:

// Pattern 1: Array modification (with new setter)
material.color[0] = 1.0;
material.color[1] = 0.0;
material.color[2] = 0.0;
material.color[3] = 1.0;

// Pattern 2: Array assignment
material.color = [1.0, 0.0, 0.0, 1.0];

// Pattern 3: Helper methods
material.set_color(1.0, 0.0, 0.0, 1.0);
material.set_rgb(1.0, 0.0, 0.0);

// All should result in red material
```

## Final Validation Checklist
- [ ] All scripts compile without errors
- [ ] Color pulse animation is visually smooth
- [ ] Rotating cube tint is applied correctly
- [ ] No performance regression from setter overhead
- [ ] All material modification patterns documented
- [ ] Test scenes demonstrate each pattern
- [ ] No script errors in console during runtime

---

## Anti-Patterns to Avoid
- ❌ Don't forget to handle Dynamic type conversions in setter
- ❌ Don't assume array will always have 4 elements (RGB vs RGBA)
- ❌ Don't skip value clamping (0.0-1.0 range)
- ❌ Don't break existing Material constructor functions
- ❌ Don't modify Material struct itself (only registration)
- ❌ Don't forget to test with actual visual output

## Success Confidence Score: 9/10

**High confidence** because:
- Root cause is clearly identified (missing setter)
- Solution pattern is well-documented in Rhai docs
- Similar getter/setter patterns exist in codebase
- Testing can be done visually for immediate feedback
- No complex architectural changes required

**Minor risks**:
- Dynamic type handling in setter might have edge cases
- Performance impact of setter validation needs testing