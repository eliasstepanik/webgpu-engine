## FEATURE:
Replace all println!/eprintln! statements with proper tracing-based logging

## EXAMPLES:
.claude/examples/console-output-before.txt – shows current messy console output
.claude/examples/console-output-after.txt – demonstrates clean structured logging

## DOCUMENTATION:
https://docs.rs/tracing
https://github.com/tokio-rs/tracing
https://blog.logrocket.com/comparing-logging-tracing-rust/
https://www.shuttle.dev/blog/2024/01/09/getting-started-tracing-rust

## OTHER CONSIDERATIONS:
- Engine already has init_logging() configured in engine/src/lib.rs:40-50
- CLAUDE.md section 11 mandates using tracing ecosystem - NO println!/eprintln!
- Found 37 violations across 4 files (game/src/main.rs, examples/scene_demo.rs, editor/src/panels/hierarchy.rs, editor/src/panels/inspector.rs)
- Must import: use tracing::{debug, error, info, warn, trace};
- Use structured fields: info!(entity_id = id, "Message");
- Default log level is "info,wgpu_core=warn,wgpu_hal=warn" 
- Users control filtering via RUST_LOG env var
- Avoid trace!() in render loops for performance
- Some existing code already uses debug! macro correctly (hierarchy.rs:173)