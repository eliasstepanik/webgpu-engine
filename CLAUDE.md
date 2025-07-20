# Agent Development Guide (v2.1)

This guide defines how AI agents must work within this repository. Follow every **MANDATORY** rule. Deviations fail the task.

---

## 1. **MANDATORY** Core Directives

* **Safety First**: Never touch system files. Operate only inside the project directory.
* **Analyze, Then Act**: Inspect existing code and docs with `read_file` and `list_directory` before changing anything.
* **Follow Conventions**: Match the established coding style and architecture. Check `PLANNING.md` first.
* **Preserve Working Code**: Do not delete or overwrite functional code unless it is provably wrong or obsolete.
* **Verify Changes**: After each modification run `just preflight` (format, clippy, tests, docs) and fix all failures.

---

---

## 2. **MANDATORY** Testing Protocol

* **Write Unit Tests** for every new function or feature.

    * Happy path
    * Edge cases (empty, zero, etc.)
    * Error conditions (invalid inputs, missing files, etc.)
* **Run Tests** with `cargo test --workspace` and ensure green status.

---

## 3. Project Interaction

### 3.1. Running the Project

* `just run` executes the `game` crate.
* `just preflight` runs formatting, linting, tests, and docs.

### 3.2. Modifying the Engine

1. Pick the correct module (`core`, `graphics`, `io`, etc.).
2. Add new files; update the parent `mod.rs`.
3. Mark public items `pub`.
4. Update `engine/src/lib.rs` for new top‑level modules.
5. Reflect the change in `PLANNING.md`.

### 3.3. Adding New Components

1. Create the struct in `engine/src/core/entity/components/mod.rs`.
2. No extra steps for `World` (works with any `'static` struct).
3. Document the component in `PLANNING.md`.

---

## **4. Documentation**

* The `.claude/documentation` folder contains useful resources and documentation for the project. Consult it before starting any new task.
* https://www.shadertoy.com/view/X3XfRM
* https://github.com/gfx-rs/wgpu
* https://github.com/eliasstepanik/Pathtracer
* All public‑facing items must be documented with clear and concise doc comments.

---

## 5. Code Style

* Run `cargo fmt --all` for formatting.
* Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and resolve all warnings.
* **Module Structure**: To avoid module naming conflicts, never create a `mod.rs` file that declares a module with the same name as its parent directory. Instead, name the file containing the module's code after the module itself (e.g., for a `renderer` module, create `renderer.rs`, not `renderer/mod.rs`).
* **Trait Imports**: When using methods from a trait, ensure the trait is explicitly imported into the current scope. For example, to use the `mul` method on a `Mat4`, you must include `use std::ops::Mul;`.
* Follow TigerStyle: explicit APIs, minimal scope, safe defaults, fast‑fail logic.

---

## 6. Version‑Control Workflow (Git)

* Work in feature branches named `feat/<ticket-id>-<slug>`.
* Rebase on `main` before opening a PR; resolve conflicts locally.
* Require at least **one** approving review.
* CI (`just preflight`) must pass before merging.

---

## 7. Commit Message Conventions

* Format: `<type>: <short summary>`
  Examples: `feat: add frustum culling`, `fix: handle null input`.
* Use these **types**: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `ci`, `chore`.
* Add a blank line, then body if needed.
* Include a `BREAKING CHANGE:` footer when public APIs change.
* **IMPORTANT**: Never mention Claude, Anthropic, AI, or similar in commit messages
* **IMPORTANT**: Never create git commits automatically - always let the user create them
* Git author: Elias Stepanik <eliasstepanik@proton.me>
* GitHub: https://github.com/eliasstepanik

---

## 8. Continuous Integration

* Every PR runs `just preflight` plus integration tests.
* Fail fast: any red step blocks merge.
* Docs must build (`cargo doc --workspace --no-deps --document-private-items`).

---

## 9. Security & Privacy Checks

* No hard‑coded secrets; use environment variables or secrets manager.
* Run `cargo audit` weekly; treat high‑severity findings as merge‑blocking.

---

## 10. Performance Budget

* A PR increasing binary size > 5 % or slowing benchmarks > 3 % must include:

    * Justification
    * Plan to optimize or roll back

---

## 11. Logging & Observability

* **MANDATORY**: Use the `tracing` ecosystem for all logging - NO `println!` or `eprintln!` statements.
* **Import Pattern**: `use tracing::{debug, error, info, warn, trace};`
* **Initialization**: Already configured in `engine::logging::init_logging()` - do not modify.
* **Structured Fields**: Always use `key = value` syntax: `info!(entity_id = id, "Message");`
* **Log Levels**:
  - `error!()` - Critical errors that may cause failure
  - `warn!()` - Warnings about potentially problematic situations  
  - `info!()` - General information about application flow (default level)
  - `debug!()` - Detailed debugging information
  - `trace!()` - Very verbose tracing (avoid in render loops)
* **Module Filtering**: Users control via `RUST_LOG="warn,engine::input=debug" cargo run`
* **Field Syntax**: 
  - Simple: `info!("Application started");`
  - With data: `debug!(key = ?value, state = ?state, "Event occurred");`
  - Error context: `error!(error = %e, "Operation failed");`
* **Performance**: Avoid `trace!()` in graphics render loops - use `debug!()` sparingly.

---

## 12. Dependency Management

* Prefer crates with active maintenance (≥ 1 release/year, issues triaged).
* Pin versions in `Cargo.toml`.
* Run `cargo update -p <crate>` only after tests pass.

---

## 13. Code‑Review Checklist

* Tests cover new paths.
* No panics on error paths; use `Result`.
* Documentation and examples compile (`cargo test --doc`).
* Public API surface minimal and clear.

---

## 14. Release Procedure

1. Tag with semantic version (`vX.Y.Z`).
2. Generate changelog from merged PR titles.
3. Push tag; CI publishes artifacts.

---

## 15. Incident Rollback

* Keep the last **two** release tags deployable.
* Provide `just rollback <tag>` script to redeploy a stable tag quickly.

---

## 16. UI Annotation System

The engine provides an automatic UI generation system for the editor inspector using derive macros and annotations.

### 16.1. Basic Usage

To enable automatic UI generation for a component:

```rust
#[derive(Component, EditorUI)]
pub struct MyComponent {
    #[ui(range = 0.0..100.0, speed = 0.1, tooltip = "Speed in units/second")]
    pub speed: f32,
    
    #[ui(tooltip = "Object name")]
    pub name: String,
    
    #[ui(hidden)]
    pub internal_state: u32,
}
```

### 16.2. Supported UI Attributes

* `range = min..max` - Sets min/max values for numeric fields
* `speed = value` - Sets drag speed for numeric inputs  
* `step = value` - Sets step size for numeric inputs
* `tooltip = "text"` - Adds hover tooltip
* `label = "text"` - Custom label (defaults to field name)
* `format = "%.2f"` - Printf-style format string
* `hidden` - Hides field from inspector
* `readonly` - Makes field non-editable
* `multiline` - For string fields, enables multiline input
* `color_mode = "rgb"/"rgba"` - For color fields
* `custom = "function_name"` - Use custom UI function

### 16.3. Automatic Widget Selection

The system automatically selects appropriate widgets based on field types:
* `f32/f64` → DragFloat
* `i32/u32/etc` → DragInt  
* `bool` → Checkbox
* `String` → InputText
* `Vec3` → Vec3Input (3 drag floats)
* `Quat` → QuatInput (euler angles)
* `[f32; 3]` → ColorEdit (RGB)
* `[f32; 4]` → ColorEdit (RGBA)

### 16.4. Implementation Notes

* Both `#[derive(Component)]` and `#[derive(EditorUI)]` must be present
* The Component derive macro has a hardcoded list of components with UI support
* UI metadata is generated at compile time and stored in the component registry
* The inspector uses metadata to render UI dynamically

### 16.5. Adding UI Support to New Components

1. Add both derives: `#[derive(Component, EditorUI)]`
2. Add the component name to the hardcoded list in `engine_derive/src/lib.rs`
3. Add UI annotations to fields as needed
4. Register the component using `Component::register()`

---

Adhere strictly to this guide. Challenge poor ideas, keep code safe, and maintain project integrity.
