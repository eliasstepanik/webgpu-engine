# Physics Scene Configuration Guide

This guide explains how to properly configure physics scenes to ensure collision detection works correctly.

## Quick Start

Use `physics_scene_template.json` as a starting point for new physics scenes. It includes properly positioned objects that will collide correctly.

## Key Concepts

### Transform Scale Effects on Collision

**CRITICAL**: The `scale` field in `Transform` components directly affects collision shape sizes!

```json
{
  "Transform": {
    "position": [0.0, -1.0, 0.0],
    "scale": [20.0, 0.2, 20.0]  // Scale affects collision!
  },
  "Collider": {
    "shape": {
      "Box": {
        "half_extents": [0.5, 0.5, 0.5]  // Base size before scale
      }
    }
  }
}
```

**Actual collision size** = `half_extents * scale`
- In the example above: Y half-extent becomes `0.5 * 0.2 = 0.1`
- Total height becomes `0.1 * 2 = 0.2` units
- This creates a very thin floor that objects can easily tunnel through!

### Physics Configuration

The engine uses these physics settings:
- Gravity: `-9.81` m/s² (downward)
- Fixed timestep: `1/120` seconds (120 Hz)
- Maximum velocity: `100` m/s
- Contact slop: `0.004` m (4mm allowed penetration)

## Positioning Guidelines

### Floor Placement

**Good floor configuration:**
```json
{
  "Transform": {
    "position": [0.0, -0.5, 0.0],  // Floor center at Y=-0.5
    "scale": [20.0, 1.0, 20.0]     // Normal Y scale for thickness
  },
  "Collider": {
    "shape": {
      "Box": {
        "half_extents": [0.5, 0.5, 0.5]
      }
    }
  }
}
```
- Floor top surface at: Y = -0.5 + (0.5 * 1.0) = Y = 0.0
- Floor thickness: 0.5 * 1.0 * 2 = 1.0 unit

**Bad floor configuration:**
```json
{
  "Transform": {
    "position": [0.0, -1.0, 0.0],   // Floor too low
    "scale": [20.0, 0.2, 20.0]      // Too thin!
  }
}
```
- Floor top surface at: Y = -1.0 + (0.5 * 0.2) = Y = -0.9
- Floor thickness: 0.5 * 0.2 * 2 = 0.2 units (very thin!)

### Object Placement

**For objects to collide with the floor:**
1. Floor top surface must be below object bottom
2. Gap should be reasonable (< 2 units for fast collision)
3. Avoid starting objects inside static geometry

**Example calculation:**
- Floor at Y=-0.5 with scale Y=1.0 → top surface at Y=0.0
- Object at Y=1.0 with half_extents Y=0.5 → bottom at Y=0.5
- Gap = 0.5 - 0.0 = 0.5 units ✅ Good

## Common Issues and Solutions

### Issue: "Objects fall through the floor"

**Diagnosis:**
1. Check floor top surface position: `floor_y + (half_extents_y * scale_y)`
2. Check object bottom position: `object_y - (half_extents_y * scale_y)`
3. Calculate gap distance

**Solutions:**
- Move floor up: increase `position.y`
- Use thicker floors: increase `scale.y` or use larger `half_extents.y`
- Lower objects: decrease `position.y`
- Reduce gaps to < 2 units

### Issue: "Objects start inside geometry"

**Symptoms:** Objects suddenly "pop" or teleport on scene start

**Solution:** Ensure no AABB overlap at scene start:
```bash
# Use the validator to check
cargo run --bin validate_physics_scene game/assets/scenes/your_scene.json
```

### Issue: "Very thin floors cause tunneling"

**Problem:** Floors with scale Y < 0.1 may be too thin for fast objects

**Solution:** Use minimum floor thickness of 0.1 units:
- Either: `scale.y = 0.2` with `half_extents.y = 0.5`
- Or: `scale.y = 1.0` with `half_extents.y = 0.05`

## Validation Tools

### CLI Validator
```bash
# Validate a single scene
cargo run --bin validate_physics_scene game/assets/scenes/physics_debug_test.json

# Validate all physics scenes
cargo run --bin validate_physics_scene game/assets/scenes/physics_*.json
```

### Debug Mode Validation
Physics scenes are automatically validated in debug builds when loaded. Check console output for warnings.

### Manual Testing
```bash
# Test a specific scene
SCENE=physics_debug_test cargo run

# Enable physics debug visualization
RUST_LOG=engine::physics=debug SCENE=physics_debug_test cargo run
```

## Scene Templates

### Basic Physics Scene
Use `physics_scene_template.json` - includes:
- Properly positioned floor at Y=-0.5
- Dynamic objects with reasonable gaps
- Various shapes (cube, sphere)
- Physics materials with different properties

### Working Examples
- `physics_working_test.json` - Minimal working setup
- `physics_debug_test.json` - Fixed version with proper positioning

### Broken Examples (for reference)
- Original `physics_debug_test.json` (before fix) - Shows gap problem

## Physics Material Properties

```json
"PhysicsMaterial": {
  "static_friction": 0.6,    // 0.0 = ice, 1.0 = rubber
  "dynamic_friction": 0.4,   // Usually < static_friction
  "restitution": 0.3         // 0.0 = no bounce, 1.0 = perfect bounce
}
```

## Best Practices

1. **Start with template**: Copy `physics_scene_template.json`
2. **Validate early**: Run validator before testing
3. **Use reasonable scales**: Avoid extreme values (< 0.1 or > 100)
4. **Test collisions**: Verify objects actually collide as expected
5. **Mind the gaps**: Keep vertical gaps under 2 units for predictable behavior
6. **Document assumptions**: Add comments explaining positioning choices

## Troubleshooting Checklist

- [ ] Floor thickness > 0.1 units?
- [ ] Gap between objects < 2 units?
- [ ] No initial overlaps between objects?
- [ ] Transform scales reasonable (0.1 to 100)?
- [ ] Gravity-affected objects have `use_gravity: true`?
- [ ] Static objects lack `Rigidbody` component?
- [ ] Scene validates without errors?
- [ ] Objects visibly collide when testing?

## Technical Details

### AABB Calculation
```
AABB = {
  min: position - (half_extents * scale),
  max: position + (half_extents * scale)
}
```

### Collision Detection Pipeline
1. **Broad Phase**: AABB overlap test
2. **Narrow Phase**: Detailed collision if AABBs overlap
3. **Constraint Generation**: Create contact constraints
4. **AVBD Solver**: Resolve collisions and apply forces

### GlobalTransform Requirement
Physics system requires `GlobalTransform` components, which are automatically created by the hierarchy system. This happens automatically when scenes load.