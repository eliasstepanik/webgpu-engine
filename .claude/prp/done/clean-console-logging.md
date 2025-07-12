name: "Clean Console Logging - Replace println!/eprintln! with Tracing"
description: |

## Purpose
Replace all 37 println!/eprintln! statements with proper tracing-based logging to ensure consistent, filterable, and structured console output as mandated by CLAUDE.md section 11.

## Core Principles
1. **Zero println!/eprintln!**: CLAUDE.md section 11 explicitly forbids these
2. **Structured Logging**: Use key=value syntax for all contextual data
3. **Appropriate Levels**: Match log level to message importance
4. **Performance Aware**: Avoid trace! in render loops
5. **Follow Existing Patterns**: Many files already use tracing correctly

---

## Goal
Transform messy console output with mixed formats, emojis, and unstructured text into clean, structured, filterable tracing-based logs that follow the established patterns in the codebase.

## Why
- **CLAUDE.md Compliance**: Section 11 mandates tracing ecosystem usage - NO println!/eprintln!
- **Filtering**: Users can control log verbosity via RUST_LOG environment variable
- **Consistency**: Standardized format with timestamps, module paths, and levels
- **Performance**: Tracing can be optimized out at compile time for release builds
- **Debugging**: Structured fields make it easier to search and analyze logs

## What
Replace all println!/eprintln! statements in 4 files with appropriate tracing macros, following the established patterns already present in the codebase.

### Success Criteria
- [ ] Zero println!/eprintln! statements remain in the codebase
- [ ] All logs use structured field syntax where applicable
- [ ] Appropriate log levels used (debug, info, warn, error)
- [ ] `just preflight` passes without errors
- [ ] Logs can be filtered using RUST_LOG environment variable

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/tracing
  why: Official tracing crate documentation for macro syntax and best practices
  
- url: https://github.com/tokio-rs/tracing
  why: Examples of structured logging patterns and field syntax
  
- url: https://blog.logrocket.com/comparing-logging-tracing-rust/
  why: Explains differences between log levels and when to use each
  
- url: https://www.shuttle.dev/blog/2024/01/09/getting-started-tracing-rust
  why: Practical examples of migrating from println! to tracing

- file: engine/src/lib.rs:40-50
  why: Shows existing init_logging() configuration - DO NOT MODIFY
  
- file: engine/src/io/scene.rs:90-107
  why: Example of correct structured field usage with error = %e pattern
  
- file: editor/src/panels/hierarchy.rs:173
  why: Example of correct debug! usage in editor context

- file: CLAUDE.md:120-137
  why: Section 11 - Mandatory logging requirements and patterns
```

### Current Codebase Violations
```bash
# Files with println!/eprintln! violations:
game/src/main.rs - 2 violations (lines 348, 351)
examples/scene_demo.rs - 27 violations (heavy emoji usage)
editor/src/panels/hierarchy.rs - 5 violations (but also has correct usage)
editor/src/panels/inspector.rs - 3 violations (lines 75, 79, 102)
```

### Existing Correct Patterns
```rust
// From engine/src/io/scene.rs:90
debug!(
    entity_count = entity_to_id.len(),
    "Serializing scene with entities"
);

// From engine/src/io/scene.rs:106
error!(error = %e, "Failed to serialize Transform");

// From editor/src/panels/hierarchy.rs:173
debug!("Selected entity: {:?}", entity);

// From engine/src/scripting/system.rs
warn!(script_path = ?script_path, error = %e, "Failed to load script");
```

### Known Gotchas & Requirements
```rust
// CRITICAL: CLAUDE.md Section 11 Requirements
// 1. Import pattern: use tracing::{debug, error, info, warn, trace};
// 2. Structured fields: info!(entity_id = id, "Message");
// 3. Log levels:
//    - error!() - Critical errors that may cause failure
//    - warn!() - Warnings about potentially problematic situations  
//    - info!() - General information about application flow (default level)
//    - debug!() - Detailed debugging information
//    - trace!() - Very verbose tracing (avoid in render loops)
// 4. Field syntax:
//    - Simple: info!("Application started");
//    - With data: debug!(key = ?value, state = ?state, "Event occurred");
//    - Error context: error!(error = %e, "Operation failed");
// 5. Performance: Avoid trace!() in graphics render loops
// 6. Default filter: "info,wgpu_core=warn,wgpu_hal=warn"
```

## Implementation Blueprint

### Import Updates
```yaml
game/src/main.rs:
  - Already has: use tracing::{debug, info};
  - No change needed

examples/scene_demo.rs:
  - Add at top: use tracing::{debug, info};

editor/src/panels/hierarchy.rs:
  - Already has: use tracing::debug;
  - No change needed

editor/src/panels/inspector.rs:
  - Already has: use tracing::debug;
  - Change to: use tracing::{debug, warn};
```

### List of tasks to be completed in order

```yaml
Task 1: Fix game/src/main.rs
MODIFY game/src/main.rs:
  - FIND: eprintln!("MAIN DEBUG: Processing scene operation: {operation:?}");
  - REPLACE: debug!(operation = ?operation, "Processing scene operation");
  
  - FIND: eprintln!("MAIN DEBUG: Creating new default scene (this will clear existing entities)");
  - REPLACE: debug!("Creating new default scene (this will clear existing entities)");

Task 2: Fix editor/src/panels/inspector.rs  
MODIFY editor/src/panels/inspector.rs:
  - UPDATE import: use tracing::{debug, warn};
  
  - FIND: eprintln!("INSPECTOR DEBUG: Entity {entity:?} components: Name={}, Transform={}, Camera={}, Material={}, Mesh={}",
  - REPLACE: debug!(entity = ?entity, has_name = components.0, has_transform = components.1, has_camera = components.2, has_material = components.3, has_mesh = components.4, "Entity components");
  
  - FIND: eprintln!("WARNING: Failed to access world for entity {entity:?}");
  - REPLACE: warn!(entity = ?entity, "Failed to access world for entity");
  
  - FIND: eprintln!("WARNING: Entity {entity:?} missing Name component");
  - REPLACE: warn!(entity = ?entity, "Entity missing Name component");

Task 3: Fix examples/scene_demo.rs
MODIFY examples/scene_demo.rs:
  - ADD import: use tracing::{debug, info};
  
  - REPLACE ALL println! statements with appropriate info! calls
  - REMOVE emojis from log messages
  - Convert multi-line statistics to separate info! calls
  
  Examples:
  - println!("🏗️  Building demo scene..."); → info!("Building demo scene...");
  - println!("📊 Scene statistics:"); → info!("Scene statistics:");
  - println!("   Entities: {}", count); → info!(entity_count = count, "Entities in scene");
  - println!("   {old_id} -> {new_entity:?}"); → info!(old_id = %old_id, new_entity = ?new_entity, "Entity ID mapping");

Task 4: Verify and test
RUN validation:
  - cargo fmt --all
  - cargo clippy --workspace --all-targets --all-features -- -D warnings
  - rg "println!|eprintln!" --type rust  # Should return empty
  - just preflight  # Run full validation suite
```

### Conversion Patterns
```rust
// Pattern 1: Simple message
// OLD: println!("Building scene...");
// NEW: info!("Building scene...");

// Pattern 2: Variable interpolation
// OLD: eprintln!("DEBUG: Processing operation: {operation:?}");
// NEW: debug!(operation = ?operation, "Processing operation");

// Pattern 3: Multiple values
// OLD: eprintln!("Entity {} has components: {}, {}", id, comp1, comp2);
// NEW: debug!(entity_id = %id, component1 = comp1, component2 = comp2, "Entity components");

// Pattern 4: Warning/Error
// OLD: eprintln!("WARNING: Failed to access entity {entity:?}");
// NEW: warn!(entity = ?entity, "Failed to access entity");

// Pattern 5: Multi-line output
// OLD: println!("Stats:\n  Count: {}\n  Size: {}", count, size);
// NEW: 
//   info!("Stats:");
//   info!(count, "Total count");
//   info!(size, "Total size");

// Pattern 6: Conditional debug
// OLD: if debug_mode { println!("Debug: {}", value); }
// NEW: debug!(value, "Debug information"); // Controlled by RUST_LOG
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Verify no println!/eprintln! remain
rg "println!|eprintln!" --type rust

# Expected: No matches found
```

### Level 2: Functionality Test
```bash
# Test with different log levels
RUST_LOG=debug cargo run  # Should show all debug messages
RUST_LOG=info cargo run   # Should hide debug messages (default)
RUST_LOG=warn cargo run   # Should only show warnings and errors

# Run the scene demo example
RUST_LOG=info cargo run --example scene_demo

# Expected: Clean structured output with timestamps and module paths
```

### Level 3: Integration Test
```bash
# Run full validation suite
just preflight

# Expected: All checks pass
# - cargo fmt --all
# - cargo clippy with no warnings
# - cargo test --workspace
# - cargo doc builds successfully
```

## Final Validation Checklist
- [ ] No println!/eprintln! in codebase: `rg "println!|eprintln!" --type rust` returns empty
- [ ] All files have correct tracing imports
- [ ] Structured fields used for all contextual data
- [ ] Appropriate log levels (debug for development, info for user-facing, warn for issues)
- [ ] No emojis in log messages (unless explicitly requested)
- [ ] `just preflight` passes all checks
- [ ] Logs properly filtered by RUST_LOG environment variable
- [ ] Editor functionality unchanged (entity selection, inspector updates work)

---

## Anti-Patterns to Avoid
- ❌ Don't use println!/eprintln! - CLAUDE.md forbids it
- ❌ Don't use string interpolation in log messages - use structured fields
- ❌ Don't include \n in log messages - use separate log calls
- ❌ Don't use trace!() in render loops - performance impact
- ❌ Don't modify init_logging() in engine/src/lib.rs
- ❌ Don't add emojis to log messages
- ❌ Don't use info!() for debug information - use debug!()
- ❌ Don't forget the ? for Debug format or % for Display format in fields

## Score
**Confidence Level: 9/10**

This is a straightforward refactoring task with clear patterns to follow. The codebase already uses tracing correctly in many places, providing excellent examples. The only reason it's not 10/10 is the large number of changes in scene_demo.rs that need careful conversion to maintain the same information structure.