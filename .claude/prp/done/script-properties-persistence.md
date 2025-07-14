name: "Script Properties Persistence and System Cleanup"
description: |

## Purpose
Fix script properties resetting on each execution and clean up the script system by removing unused debug/experimental files. This PRP provides comprehensive context for an AI agent to implement property persistence following existing patterns and safely remove unnecessary code.

## Core Principles
1. **Context is King**: Include ALL necessary documentation, examples, and caveats
2. **Validation Loops**: Provide executable tests/lints the AI can run and fix
3. **Information Dense**: Use keywords and patterns from the codebase
4. **Progressive Success**: Start simple, validate, then enhance
5. **Global rules**: Be sure to follow all rules in CLAUDE.md

---

## Goal
Enable scripts to persist property changes across frames and clean up the script system by removing unused debug files. Currently, any property modifications made by scripts are lost after each frame execution.

## Why
- Scripts cannot maintain state between frames (e.g., counters, accumulated values)
- Debug files create confusion and maintenance burden
- Users expect property changes in scripts to persist like component changes do
- Cleaner codebase improves maintainability

## What
- Scripts can modify properties and changes persist across frames
- Unused debug/experimental files are removed
- Property persistence follows the existing command queue pattern
- Type safety is maintained through existing PropertyValue conversion

### Success Criteria
- [ ] Script-modified properties persist between frames
- [ ] All unused debug files removed
- [ ] Existing scripts continue to work
- [ ] `just preflight` passes (format, clippy, tests, docs)
- [ ] No performance regression from property updates

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://rhai.rs/book/engine/scope.html
  why: Rhai documentation on maintaining state between executions using Scope
  
- url: https://docs.rs/rhai/latest/rhai/struct.Engine.html#method.call_fn_with_options
  why: CallFnOptions to control scope rewind behavior
  
- file: engine/src/scripting/system.rs
  why: Script execution system - where properties are passed and scope is managed
  
- file: engine/src/scripting/commands.rs
  why: Command pattern for thread-safe component updates - pattern to follow
  
- file: engine/src/scripting/property_types.rs
  why: PropertyValue types and conversion methods (to_dynamic, from_dynamic)
  
- file: engine/src/scripting/mod.rs
  why: Module exports - need to update after removing files
```

### Current Codebase Structure
```bash
engine/src/scripting/
├── commands.rs              # ScriptCommand enum and apply logic
├── component_access.rs      # Component cache population
├── components.rs            # Script component definitions
├── debug_property_system.rs # REMOVE - unused debug with unsafe static
├── debug_script_init.rs     # REMOVE - unused debug initialization
├── engine.rs                # Script engine core
├── focused_debug.rs         # REMOVE - unused selective debug logging
├── lifecycle_tracker.rs     # Tracks script lifecycle events
├── mod.rs                   # Module exports
├── modules/                 # Rhai module implementations
├── property_parser.rs       # Property definition parsing
├── property_preservation_system.rs # REMOVE - unused alternative init
├── property_types.rs        # PropertyValue and ScriptProperties types
├── script.rs                # Script loading and caching
├── script_init_system.rs    # ACTIVE - property initialization system
├── simple_init.rs           # REMOVE - unused simplified init
├── system.rs                # Script execution system
└── tests/                   # Unit tests
```

### Known Gotchas & Critical Information
```rust
// CRITICAL: Properties are passed as read-only copies
// engine/src/scripting/system.rs:85-93
let props_map = properties.to_rhai_map(); // Creates a COPY
scope.push("properties", props_map);

// CRITICAL: Scope is discarded after execution
// No mechanism to read back modified properties

// CRITICAL: Component cache is cleared after each frame
// engine/src/scripting/system.rs:174
component_cache.write().unwrap().clear();

// PATTERN: Commands are queued and applied after all scripts run
// engine/src/scripting/system.rs:163-171
let commands = command_queue.write().unwrap().drain(..).collect::<Vec<_>>();
for command in commands {
    command.apply(world.inner_mut())?;
}

// GOTCHA: Only script_init_system.rs is used in production
// All other init systems are unused experiments
```

## Implementation Blueprint

### Data Models and Structure

Add SetProperties command variant:
```rust
// In engine/src/scripting/commands.rs
pub enum ScriptCommand {
    SetTransform { entity: u64, transform: Transform },
    SetMaterial { entity: u64, material: Material },
    CreateEntity { components: Vec<ComponentData> },
    DestroyEntity { entity: u64 },
    SetProperties { entity: u64, properties: ScriptProperties }, // NEW
}
```

### List of Tasks

```yaml
Task 1 - Remove Unused Debug Files:
DELETE engine/src/scripting/debug_property_system.rs
DELETE engine/src/scripting/debug_script_init.rs
DELETE engine/src/scripting/focused_debug.rs
DELETE engine/src/scripting/simple_init.rs
DELETE engine/src/scripting/property_preservation_system.rs

MODIFY engine/src/scripting/mod.rs:
  - REMOVE module declarations for deleted files
  - KEEP only active systems

Task 2 - Add SetProperties Command:
MODIFY engine/src/scripting/commands.rs:
  - ADD SetProperties variant to ScriptCommand enum
  - ADD ScriptProperties to use statements
  - IMPLEMENT apply logic following SetTransform pattern

Task 3 - Implement Property Persistence:
MODIFY engine/src/scripting/system.rs:
  - AFTER script execution, retrieve properties from scope
  - COMPARE with original properties
  - QUEUE SetProperties command if changed
  - PRESERVE existing error handling

Task 4 - Add Property Write-Back Logic:
MODIFY engine/src/scripting/system.rs:
  - FIND: "if let Err(e) = script_engine.call_on_update"
  - INJECT AFTER: Property comparison and update logic
  - USE: PropertyValue::from_dynamic for type conversion
  - HANDLE: Type conversion failures gracefully

Task 5 - Update Tests:
CREATE engine/src/scripting/tests/property_persistence_test.rs:
  - TEST property values persist between frames
  - TEST type conversion for all PropertyValue types
  - TEST invalid property updates are rejected
```

### Pseudocode for Key Changes

```rust
// Task 3 - After script execution in system.rs
if let Some(ref original_properties) = script_properties {
    // Try to get modified properties from scope
    if let Some(modified_props) = scope.get_value::<rhai::Map>("properties") {
        let mut changed = false;
        let mut updated_properties = original_properties.clone();
        
        // Check each property for changes
        for (name, original_value) in &original_properties.values {
            if let Some(new_dynamic) = modified_props.get(name) {
                // Get expected type from property definitions
                if let Some(prop_type) = get_property_type(name) {
                    // Try to convert back to PropertyValue
                    if let Some(new_value) = PropertyValue::from_dynamic(new_dynamic, prop_type) {
                        if new_value != *original_value {
                            updated_properties.values.insert(name.clone(), new_value);
                            changed = true;
                        }
                    }
                }
            }
        }
        
        // Queue update command if properties changed
        if changed {
            command_queue.write().unwrap().push(ScriptCommand::SetProperties {
                entity: entity.to_bits().get(),
                properties: updated_properties,
            });
        }
    }
}
```

### Integration Points
```yaml
COMMANDS:
  - location: engine/src/scripting/commands.rs
  - pattern: Match existing command patterns for consistency
  
PROPERTY TYPES:
  - use: PropertyValue::from_dynamic for type-safe conversion
  - use: ScriptProperties::values HashMap for storage
  
LOGGING:
  - use: tracing::{debug, trace} for property updates
  - pattern: debug!(entity = ?entity, "Updated script properties")
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run after each file modification
just preflight

# Individual checks if needed:
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Level 2: Unit Tests
```rust
// Test property persistence
#[test]
fn test_script_property_persistence() {
    let mut world = World::new();
    let entity = world.spawn((
        ScriptRef::new("test_script"),
        ScriptProperties::new("test_script", vec![
            ("counter", PropertyValue::Integer(0)),
        ])
    ));
    
    // Execute script that increments counter
    script_execution_system(&mut world, &mut engine, &input, 0.016);
    
    // Verify counter was incremented
    let props = world.get::<&ScriptProperties>(entity).unwrap();
    assert_eq!(props.values["counter"], PropertyValue::Integer(1));
}
```

### Level 3: Integration Test
```bash
# Run the game with a test scene
just run

# Load a scene with scripts that modify properties
# Verify properties persist across frames in the inspector UI
```

## Final Validation Checklist
- [ ] All unused debug files removed
- [ ] SetProperties command implemented and tested
- [ ] Property persistence works for all PropertyValue types
- [ ] `just preflight` passes without warnings
- [ ] Existing scripts continue to work unchanged
- [ ] Performance impact is negligible
- [ ] No unsafe code or global statics remain

---

## Anti-Patterns to Avoid
- ❌ Don't use global statics for state (remove them)
- ❌ Don't skip type conversion validation
- ❌ Don't modify properties outside the command queue
- ❌ Don't clear scope before reading properties back
- ❌ Don't ignore conversion failures - log them
- ❌ Don't break existing script functionality

## Confidence Score: 9/10

The implementation path is clear with existing patterns to follow. The only complexity is in the property comparison logic, but the existing PropertyValue conversion methods handle the type safety. The command queue pattern ensures thread safety.