## FEATURE:
Refactor project structure to move assets to game crate and create minimal engine initialization API for cleaner separation of concerns

## EXAMPLES:
.claude/examples/console-output-before.txt – Shows current engine output patterns
.claude/examples/console-output-after.txt – Shows expected output after refactoring

## DOCUMENTATION:
https://docs.rs/wgpu/latest/wgpu/ – WebGPU API documentation for engine initialization patterns
https://github.com/bevyengine/bevy/tree/main/examples – Reference for engine/game separation patterns
https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html – Rust module paths for asset loading

## OTHER CONSIDERATIONS:
- Asset paths are currently relative to project root (e.g., "assets/scenes/demo_scene.json")
- Scene loading supports both JSON files and programmatic creation
- Engine uses feature flags (e.g., "editor" feature) that affect initialization
- Script engine (Rhai) needs access to asset paths for script loading
- Hot-reload functionality in engine/src/io/hot_reload.rs may need path updates
- Tests may reference asset paths that need updating
- Editor feature creates additional complexity with detached windows and shared state