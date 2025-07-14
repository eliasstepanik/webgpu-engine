## FEATURE:

Fix script properties resetting on each execution and clean up the script system by removing unused/unnecessary files and functions

## EXAMPLES:

.claude/examples/console-output-before.txt – Shows script property values resetting between executions
.claude/examples/console-output-after.txt – Expected behavior with properties persisting across frames

## DOCUMENTATION:

https://rhai.rs/book/engine/scope.html – Rhai documentation on maintaining state between executions using Scope
https://docs.rs/rhai/latest/rhai/struct.Engine.html#method.call_fn_with_options – Rhai Engine call options for state control
https://github.com/SanderMertens/ecs-faq – ECS best practices for component persistence
https://gamedev.stackexchange.com/questions/189268/ways-to-persist-entities-and-components-in-an-ecs – Entity persistence strategies in ECS

## OTHER CONSIDERATIONS:

### Property Reset Issue
- Properties are passed to scripts as read-only copies via `to_rhai_map()` in engine/src/scripting/system.rs:85
- No write-back mechanism exists to persist script-modified properties back to the ScriptProperties component
- Scope is recreated each frame and discarded after execution, losing all property changes
- Component cache is cleared after each frame (line 174 in system.rs)
- Missing ScriptCommand variant for updating properties in commands.rs

### Files to Remove (unused debug/experimental code)
- debug_property_system.rs – Uses unsafe static mut, extensive debug logging
- debug_script_init.rs – Debug version of initialization system
- focused_debug.rs – Debug system with frame counter and property history
- simple_init.rs – Simplified init only used by focused_debug.rs
- property_preservation_system.rs – Alternative init system never referenced

### Implementation Notes
- Only script_init_system.rs is actually used in production (called in app.rs)
- Consider implementing a SetProperties command and write-back mechanism after script execution
- May need to track which properties were modified to avoid unnecessary updates
- Consider using Rhai's Scope persistence features with careful size management
- Follow ECS patterns: separate persistent IDs from runtime IDs for serialization