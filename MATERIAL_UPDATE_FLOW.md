# Material Update Flow in Scripts

## Overview
This document traces the complete flow of material updates from scripts to the ECS system.

## The Flow

### 1. Script Execution (`script_execution_system`)
- Scripts are executed for each entity with a `ScriptRef` component
- Component cache is populated with current world state before script execution
- Scripts run in a sandboxed Rhai environment with access to world, input, and math modules

### 2. Material Access in Scripts
Scripts can access materials through the world module:
```rhai
let material = world::get_component(entity, "Material");
```
This retrieves a copy of the material from the component cache.

### 3. Material Modification
The Material type is registered with Rhai with:
- **Getter**: `color` property returns an array `[r, g, b, a]`
- **Setter**: `color` property can be set with an array (NEW)
- **Methods**: 
  - `set_color(r, g, b, a)` - Set all color components
  - `set_rgb(r, g, b)` - Set RGB, keep existing alpha
  - `clone()` - Create a copy

Example usage:
```rhai
// Method 1: Using setter
material.set_color(1.0, 0.0, 0.0, 1.0); // Red

// Method 2: Direct array modification (requires setter)
material.color = [1.0, 0.0, 0.0, 1.0];
```

### 4. Command Queue
After modifying the material, scripts must update the world:
```rhai
world::set_component(entity, "Material", material);
```

This creates a `ScriptCommand::SetMaterial` command and adds it to the command queue.

### 5. Command Application
After all scripts finish execution:
1. All queued commands are drained from the queue
2. Each command is applied to the ECS world
3. For `SetMaterial`, this calls `world.insert_one(entity, material)`

### 6. Rendering
The renderer queries entities with `Material` components each frame and uses the updated color values.

## Key Points

1. **Thread Safety**: Commands are queued and applied after all scripts finish to prevent race conditions
2. **Component Cache**: Components are cached before script execution for performance
3. **Material is Copy**: The Material type implements Copy, so scripts work with value copies
4. **Debug Logging**: Enable with `RUST_LOG=engine::scripting=debug`

## Testing

Test scenes:
- `test_material_simple.json` - Basic material color cycling
- `test_color_pulse.json` - Color pulsing effect
- `test_color_pulse_fixed.json` - Fixed version using new methods

Test scripts:
- `material_test.rhai` - Simple color cycling test
- `color_pulse.rhai` - Original color pulse (may have issues)
- `color_pulse_fixed.rhai` - Fixed version using set_color method

## Common Issues

1. **Color modifications not persisting**: The original implementation only had a getter for the color property. Scripts could read but not write back changes.
2. **Array element assignment**: `material.color[0] = value` doesn't work without a proper setter
3. **Solution**: Added setter and convenience methods for color modification