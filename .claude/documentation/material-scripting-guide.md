# Material Scripting Guide

This guide explains how to modify materials from Rhai scripts in the WebGPU engine.

## Overview

Materials in the engine have a `color` property that is an array of 4 float values `[r, g, b, a]` representing red, green, blue, and alpha channels. Values should be in the range 0.0 to 1.0.

## Material Access Pattern

All material modifications follow this pattern:
1. Get the material component from the entity
2. Modify the material's properties
3. Set the component back to apply changes

```rhai
// Get material
let material = world::get_component(entity, "Material");

if material != () {
    // Modify material
    // ... (see patterns below)
    
    // Apply changes
    world::set_component(entity, "Material", material);
}
```

## Modification Patterns

### Pattern 1: Array Element Assignment

Directly modify individual color channels:

```rhai
material.color[0] = 1.0;  // Red
material.color[1] = 0.5;  // Green
material.color[2] = 0.0;  // Blue
material.color[3] = 1.0;  // Alpha
```

### Pattern 2: Array Assignment

Replace the entire color array:

```rhai
material.color = [1.0, 0.0, 0.0, 1.0];  // Solid red
```

### Pattern 3: Using set_color Method

Set all RGBA values at once:

```rhai
material.set_color(1.0, 0.0, 0.0, 1.0);  // Red with full opacity
```

### Pattern 4: Using set_rgb Method

Set only RGB, preserving existing alpha:

```rhai
material.set_rgb(0.0, 1.0, 0.0);  // Green, keeps current alpha
```

## Working with Color Properties

When using script properties of type `color`, access components with string keys:

```rhai
let base_color = properties["base_color"];
let pulse_color = properties["pulse_color"];

// Access individual components
let red = base_color["r"];
let green = base_color["g"];
let blue = base_color["b"];
let alpha = base_color["a"];

// Apply to material
material.color[0] = base_color["r"];
material.color[1] = base_color["g"];
material.color[2] = base_color["b"];
material.color[3] = base_color["a"];
```

## Common Patterns

### Color Interpolation (Lerp)

Smoothly transition between two colors:

```rhai
fn lerp_color(material, color1, color2, t) {
    material.color[0] = color1["r"] + (color2["r"] - color1["r"]) * t;
    material.color[1] = color1["g"] + (color2["g"] - color1["g"]) * t;
    material.color[2] = color1["b"] + (color2["b"] - color1["b"]) * t;
    material.color[3] = color1["a"] + (color2["a"] - color1["a"]) * t;
}
```

### Color Pulsing

Create a pulsing effect using sine waves:

```rhai
let pulse_factor = (math::sin(time) + 1.0) * 0.5;  // 0.0 to 1.0
material.set_color(
    base_color["r"] * (1.0 - pulse_factor) + pulse_color["r"] * pulse_factor,
    base_color["g"] * (1.0 - pulse_factor) + pulse_color["g"] * pulse_factor,
    base_color["b"] * (1.0 - pulse_factor) + pulse_color["b"] * pulse_factor,
    base_color["a"]
);
```

### Rainbow Cycling

Cycle through colors smoothly:

```rhai
let r = (math::sin(time) + 1.0) * 0.5;
let g = (math::sin(time + 2.0) + 1.0) * 0.5;
let b = (math::sin(time + 4.0) + 1.0) * 0.5;
material.set_rgb(r, g, b);
```

## Material Constructors

Create new materials using the material module functions:

```rhai
// Predefined colors
let red_material = material::red();
let green_material = material::green();
let blue_material = material::blue();

// Custom gray
let gray_material = material::gray(0.5);  // 50% gray

// Custom RGB (alpha = 1.0)
let custom_material = material::from_rgb(0.8, 0.2, 0.4);

// Custom RGBA
let transparent_material = material::from_rgba(1.0, 1.0, 1.0, 0.5);
```

## Important Notes

1. **Value Clamping**: All color values are automatically clamped to the range 0.0-1.0
2. **Thread Safety**: Material updates are queued and applied after all scripts finish
3. **Performance**: Prefer set_color/set_rgb methods over individual array assignments for better performance
4. **Debugging**: Enable `RUST_LOG=engine::scripting=debug` to see material update logs

## Common Pitfalls

1. **Forgetting to set the component back**: Always call `world::set_component` after modifying
2. **Wrong property access**: Color properties use string keys ("r", "g", "b", "a"), not array indices
3. **Out of range values**: While values are clamped, using very large values might indicate a logic error

## Example Scripts

See these example scripts for reference:
- `color_pulse.rhai` - Smooth color pulsing between two colors
- `rotating_cube.rhai` - Applying a tint color to a rotating object
- `material_test.rhai` - Simple material color cycling for testing