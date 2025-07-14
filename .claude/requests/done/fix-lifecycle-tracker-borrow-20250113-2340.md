## FEATURE:

Fix script lifecycle tracker borrow conflict causing on_start to be called repeatedly every frame instead of once per entity

## EXAMPLES:

.claude/examples/console-output-before.txt – shows mixed emoji/plain text logging without proper levels
.claude/examples/console-output-after.txt – demonstrates proper structured logging with tracing crate

## DOCUMENTATION:

https://docs.rs/hecs/latest/hecs/struct.World.html – HECS World API and component querying methods
https://docs.rs/hecs/latest/hecs/struct.QueryBorrow.html – QueryBorrow lifetime management for dynamic borrow checking
https://github.com/Ralith/hecs – Official HECS repository with borrowing examples
https://users.rust-lang.org/t/best-way-to-solve-a-it-is-already-borrowed-error/126666 – Community solutions for borrow conflicts
https://ianjk.com/ecs-in-rust/ – Tutorial on ECS borrowing patterns in Rust

## OTHER CONSIDERATIONS:

- HECS runtime borrow checker panics when attempting simultaneous access to same component types on entities
- The World wrapper in `engine/src/core/entity/world.rs` line 40 uses `get::<&T>` but may need `get::<T>` for proper component queries
- Script execution system collects entities at start but checks them again at end - this creates borrow conflict window
- Component cache clearing happens between script execution and entity destruction check - timing issue
- QueryBorrow lifetimes must be properly managed - references can't outlive the QueryBorrow that created them
- Current pattern: query → process → check same entities again fails due to active borrows from processing phase
- Solution requires either: deferring destruction check until after all borrows released, or using consistent query patterns throughout
- Entity IDs remain valid (4294967297 = 1v1) but component access fails due to borrowing, not actual component removal
- Debug logs show entities successfully added to tracker but immediately removed due to false positive "no ScriptRef" detection