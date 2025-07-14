# Debug Instructions for Script Properties

I've added comprehensive debug logging to help track down the property reset issue. 

## To test:

1. Run the project with: `cargo run --features editor`
2. In the editor, go to File > Load Scene
3. Load: `game/assets/scenes/test_property_persistence.json`
4. Select either "Fast Rotating Cube" or "Slow Rotating Cube" in the hierarchy
5. Try to modify the "rotation_speed" property in the inspector
6. Watch the console output for debug messages

## What to look for:

The debug system logs with these symbols:
- 🔍 = Debug system frame boundaries
- 📋 = Entity state information
- ➕ = Creating new properties
- 🔧 = Modifying existing properties
- ⚠️ = Warnings about mismatches or changes
- 🎯 = Script execution with properties

Look for messages like:
- "⚠️ PROPERTY VALUE CHANGED BETWEEN FRAMES!" - indicates values are being reset
- "Script name mismatch" - indicates reinitialization trigger
- "Property value changed in inspector" - confirms your edits are registered

The logs will show exactly when and why properties are being modified or reset.