name: "Fix Script Lifecycle Tracker Borrow Conflict"
description: |

## Purpose
Eliminate HECS component borrow conflicts in the script execution system that cause `on_start` to be called repeatedly every frame instead of once per entity, restoring proper script lifecycle management.

## Core Principles
1. **Borrow Safety**: Eliminate overlapping component access that creates false positive entity removals
2. **Consistency**: Apply collection-first patterns throughout the script system
3. **Validation**: Ensure lifecycle events occur exactly once per entity state change
4. **Performance**: Minimize component queries and borrow overhead

---

## Goal
Fix the script lifecycle tracker borrow conflict where entities are incorrectly marked as "no longer having ScriptRef" due to HECS query/get access patterns, causing `on_start` to be called every frame instead of once per entity.

## Why
- **Functional Integrity**: Scripts should only initialize once, not every frame
- **Performance**: Repeated initialization causes unnecessary overhead and memory churn
- **Developer Experience**: Current behavior makes script debugging nearly impossible
- **System Reliability**: Lifecycle tracking is fundamental to the scripting architecture

## What
Refactor the script execution system to eliminate borrow conflicts by:
- Consolidating all component access into collection phases
- Using compound queries instead of mixed query/get patterns
- Implementing proper borrow lifetime management
- Ensuring destruction checks don't create false positives

### Success Criteria
- [ ] Scripts call `on_start` exactly once per entity spawn/restart
- [ ] Lifecycle tracker maintains accurate entity state across frames
- [ ] No borrow conflicts during script execution or lifecycle management
- [ ] All existing script functionality preserved
- [ ] Performance maintained or improved

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/hecs/latest/hecs/struct.World.html
  why: Core HECS World API and component querying methods
  critical: Understanding QueryBorrow lifetimes and when borrows are released
  
- url: https://docs.rs/hecs/latest/hecs/struct.QueryBorrow.html
  why: QueryBorrow lifetime management for dynamic borrow checking
  critical: References cannot outlive the QueryBorrow that created them

- url: https://github.com/Ralith/hecs
  section: Examples of proper query patterns and borrow management
  critical: Official patterns for avoiding simultaneous borrows

- url: https://users.rust-lang.org/t/best-way-to-solve-a-it-is-already-borrowed-error/126666
  why: Community solutions for borrow conflicts in ECS systems
  critical: Collection-first patterns and query lifetime management

- url: https://ianjk.com/ecs-in-rust/
  section: ECS borrowing patterns in Rust
  critical: Understanding when queries conflict with individual component access

- file: engine/src/scripting/system.rs
  why: Current problematic implementation showing exact borrow conflict points
  critical: Lines 52-66 (collection), 272-297 (destruction check) create conflict

- file: engine/src/scripting/lifecycle_tracker.rs  
  why: Global lifecycle state management working correctly
  critical: Issue is false positive removal, not tracker logic

- file: engine/src/core/entity/world.rs
  why: World wrapper implementation with get() method returning hecs::Ref<T>
  critical: Line 40 creates active borrows that can conflict with queries

- file: engine/src/scripting/tests/
  why: Existing test patterns for script system validation
  critical: Follow established test structure and naming conventions
```

### Current Codebase Tree
```bash
engine/src/scripting/
├── commands.rs                 # Command queue system (working correctly)
├── component_access.rs         # Component cache population (working correctly)  
├── components.rs              # ScriptRef and component definitions
├── engine.rs                  # Script compilation and execution (working correctly)
├── lifecycle_tracker.rs       # Global state tracker (working correctly)
├── mod.rs                     # Module declarations and re-exports
├── modules/                   # Script modules (input, math, world)
├── property_parser.rs         # Property definition parsing (working correctly)
├── property_types.rs          # Property value types (working correctly)
├── script.rs                  # Script loading and caching (working correctly)
├── script_init_system.rs      # Script initialization (similar borrow issues)
├── system.rs                  # MAIN PROBLEM: Script execution with borrow conflicts
└── tests/                     # Unit tests (need lifecycle tracker tests)
```

### Desired Codebase Tree (files to modify/add)
```bash
engine/src/scripting/
├── system.rs                  # MODIFY: Fix borrow patterns in script execution
├── script_init_system.rs      # MODIFY: Apply same borrow fixes  
├── lifecycle_tracker.rs       # MODIFY: Add debugging/validation methods
└── tests/
    ├── lifecycle_tests.rs     # CREATE: Tests for lifecycle tracker behavior
    └── borrow_safety_tests.rs # CREATE: Tests ensuring no borrow conflicts
```

### Known Gotchas & Library Quirks
```rust
// CRITICAL: HECS QueryBorrow lifetimes extend beyond apparent scope
// Example: This creates a long-lived borrow even after loop ends
for (entity, script) in world.query::<&ScriptRef>().iter() {
    // This can fail due to existing borrow from query iterator
    world.get::<&ScriptRef>(entity)  // BORROW CONFLICT
}

// CRITICAL: hecs::Ref<T> holds active borrows until dropped
let component_ref = world.get::<&Transform>(entity)?;  // Active borrow
// component_ref must be dropped before new queries on Transform

// CRITICAL: Mixed query patterns create conflicts
world.query::<&ScriptRef>().iter()     // Creates query borrow
world.get::<&ScriptRef>(entity)        // Individual access conflicts

// SOLUTION: Use collection-first pattern consistently
let entities: Vec<_> = world.query::<&ScriptRef>().iter().collect();
// Now safe to use world.get() or other operations

// GOTCHA: Entity IDs remain valid even when components conflict
// Error from world.get() != missing component, might be borrow conflict

// GOTCHA: Thread-local storage used for CommandQueue and ComponentCache
// These patterns work and should be preserved

// GOTCHA: Tracing crate used exclusively - no println!/eprintln!
use tracing::{debug, error, info, warn, trace};
```

## Implementation Blueprint

### Core Problem Analysis
The borrow conflict occurs because:
1. `script_execution_system()` queries entities with `ScriptRef` components
2. Processes them (creating potential borrows through command queue operations)  
3. Later checks same entities individually with `world.get::<ScriptRef>()` 
4. HECS query borrow lifetime overlaps with individual component access
5. Individual access fails → incorrectly marked as "entity destroyed"
6. Next frame: entity not in tracker → `on_start` called again

### Solution Strategy
**Consolidate all component access into distinct, non-overlapping phases:**
- **Phase 1**: Collect all needed entity data in compound queries
- **Phase 2**: Process collected data without further world queries  
- **Phase 3**: Apply commands and update lifecycle state
- **Eliminate Phase 4**: Remove or defer destruction checking

### List of Tasks (Implementation Order)

```yaml
Task 1 - Add Lifecycle Tracker Tests:
MODIFY engine/src/scripting/tests/mod.rs:
  - ADD: mod lifecycle_tests;
  - ADD: mod borrow_safety_tests;

CREATE engine/src/scripting/tests/lifecycle_tests.rs:
  - PATTERN: Follow property_tests.rs structure
  - TEST: Entity marked started only once
  - TEST: Entity stays in tracker across frames
  - TEST: Entity properly removed when actually destroyed
  - VERIFY: No false positive removals

CREATE engine/src/scripting/tests/borrow_safety_tests.rs:
  - TEST: Multiple entity queries don't conflict
  - TEST: Collection-first pattern works
  - TEST: Component access after collection is safe
  - VERIFY: No panic conditions under normal operation

Task 2 - Fix Core System Borrow Patterns:
MODIFY engine/src/scripting/system.rs:
  - FIND: Lines 52-66 entity collection loop
  - REPLACE: Use compound query for all component types at once
  - PATTERN: world.query::<(&ScriptRef, Option<&ScriptProperties>)>()
  - ELIMINATE: Individual world.get() calls within query scope

  - FIND: Lines 272-297 destruction check
  - ELIMINATE: This entire phase creates the borrow conflict  
  - ALTERNATIVE: Track destruction during processing or defer to separate system

  - PRESERVE: Thread-local CommandQueue and ComponentCache patterns
  - PRESERVE: Existing tracing log messages and structure
  - PRESERVE: Command application and lifecycle tracking calls

Task 3 - Apply Fixes to Script Init System:
MODIFY engine/src/scripting/script_init_system.rs:
  - FIND: Similar query + world.get() patterns
  - APPLY: Same collection-first fixes as main system
  - PATTERN: Use compound queries for ScriptRef + ScriptProperties
  - VERIFY: No borrow conflicts in initialization

Task 4 - Add Lifecycle Tracker Validation:
MODIFY engine/src/scripting/lifecycle_tracker.rs:
  - ADD: Debug validation methods
  - ADD: Consistency checking for started vs active entities
  - ADD: Detailed logging for state transitions
  - PATTERN: Follow existing tracing patterns

Task 5 - Update World Wrapper (if needed):
EVALUATE engine/src/core/entity/world.rs:
  - ANALYZE: Whether get() method causes extended borrows
  - CONSIDER: Alternative access patterns if needed
  - PRESERVE: Existing API compatibility
```

### Per Task Pseudocode

```rust
// Task 1 - Lifecycle Tests
#[test]
fn test_entity_started_once_per_lifecycle() {
    let mut world = World::new();
    let mut tracker = ScriptLifecycleTracker::default();
    
    let entity = world.spawn((ScriptRef::new("test"), Transform::default()));
    
    // First call should mark as started
    assert!(!tracker.has_started(entity));
    tracker.mark_started(entity);
    assert!(tracker.has_started(entity));
    
    // Subsequent calls should not change state
    tracker.mark_started(entity);  // Should be no-op
    assert!(tracker.has_started(entity));
    assert_eq!(tracker.started_count(), 1);
}

// Task 2 - Fixed System Pattern  
pub fn script_execution_system(
    world: &mut World,
    script_engine: &mut ScriptEngine,
    input_state: &ScriptInputState,
    delta_time: f32,
) {
    // FIXED: Single compound query collects all needed data
    let mut entity_data = Vec::new();
    for (entity, script_ref, script_properties) in 
        world.query::<(&ScriptRef, Option<&ScriptProperties>)>().iter() 
    {
        entity_data.push((
            entity, 
            script_ref.clone(), 
            script_properties.cloned()
        ));
    }
    // Query borrow automatically dropped here
    
    // Process collected data - safe to access world mutably
    for (entity, script_ref, script_properties) in entity_data {
        // Check lifecycle state
        let needs_start = {
            let tracker = get_tracker().lock().unwrap();
            !tracker.has_started(entity)
        };
        
        if needs_start {
            // Call on_start and mark started
            // ... existing logic
        }
        
        // Call on_update
        // ... existing logic
        
        // Handle property persistence
        // ... existing logic
    }
    
    // Apply commands
    // ... existing logic
    
    // ELIMINATED: Destruction check phase that caused conflicts
    // Alternative: Track destruction during processing or separate system
}

// Task 4 - Enhanced Lifecycle Tracker
impl ScriptLifecycleTracker {
    pub fn validate_consistency(&self) -> bool {
        // All started entities should be in active set
        self.started_entities.is_subset(&self.active_entities)
    }
    
    pub fn debug_state(&self) {
        debug!(
            started_count = self.started_entities.len(),
            active_count = self.active_entities.len(),
            consistent = self.validate_consistency(),
            "Lifecycle tracker state"
        );
    }
}
```

### Integration Points
```yaml
TESTING:
  - command: "cargo test --package engine scripting::tests"
  - validate: "All lifecycle tests pass"
  - pattern: "Follow existing test naming conventions"

BUILDING:
  - command: "just preflight"  
  - includes: "fmt, clippy, tests, docs"
  - validate: "No warnings or errors"

LOGGING:
  - use: "tracing crate exclusively"
  - levels: "debug for detailed info, trace for verbose"
  - structured: "Use key = value syntax for context"

MODULE_INTEGRATION:
  - preserve: "Existing module structure and exports"
  - maintain: "API compatibility for ScriptRef, ScriptProperties"
  - extend: "Add validation methods to lifecycle tracker"
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run formatting and linting
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Expected: No errors. Auto-formatting applied, no clippy warnings.
```

### Level 2: Unit Tests
```bash
# Test the specific scripting module
cargo test --package engine scripting::tests --verbose

# Expected: All tests pass including new lifecycle and borrow safety tests
# Focus tests:
# - test_entity_started_once_per_lifecycle  
# - test_no_borrow_conflicts_during_execution
# - test_compound_query_collection_pattern
```

### Level 3: Integration Test
```bash
# Run the game and verify script behavior
RUST_LOG=engine::scripting=debug cargo run --bin game

# Expected output:
# - Scripts call "on_start" only once per entity
# - No "🔄 Entity X not in started_entities" repeated warnings
# - Lifecycle tracker maintains consistent state
# - Debug logs show entities properly tracked

# Validation criteria:
# grep "Calling on_start" logs.txt | wc -l  # Should be minimal
# grep "Added entity.*started_entities" logs.txt  # Should see additions
# grep "Removed entity.*was_started: true" logs.txt  # Should be rare
```

## Final Validation Checklist
- [ ] All tests pass: `cargo test --workspace`
- [ ] No linting errors: `cargo clippy --workspace --all-features -- -D warnings`
- [ ] No formatting issues: `cargo fmt --all -- --check`
- [ ] Script `on_start` called exactly once per entity lifecycle
- [ ] Lifecycle tracker state remains consistent across frames
- [ ] No borrow panic conditions during normal script execution
- [ ] Performance maintained or improved
- [ ] Existing script functionality preserved
- [ ] Debug logging provides clear insight into lifecycle events

---

## Anti-Patterns to Avoid
- ❌ Don't mix query iteration with individual component access in same scope
- ❌ Don't ignore borrow errors - they indicate real architectural issues
- ❌ Don't use println!/eprintln! - use tracing crate exclusively  
- ❌ Don't skip the collection phase - always collect first, process second
- ❌ Don't assume component access failure means missing component
- ❌ Don't create new patterns when collection-first pattern exists
- ❌ Don't modify World wrapper without understanding hecs borrow semantics

## Success Confidence Score: 9/10

**High confidence** because:
- Root cause clearly identified through detailed analysis
- Solution follows established patterns already present in codebase
- HECS documentation provides clear guidance on borrow management
- Comprehensive test strategy ensures validation at each step
- Implementation preserves all existing functionality while fixing core issue

**Potential risks:**
- Edge cases in compound query patterns not fully explored
- Performance impact of collection-first approach needs validation

The solution directly addresses the core borrow conflict without changing fundamental architecture, following established patterns and providing thorough validation.