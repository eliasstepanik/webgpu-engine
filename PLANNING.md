# Project Plan

This outline explains the engine structure and the rules that keep the codebase consistent.

---

## 1. Purpose

Create a clean, minimal template for real‑time 3D applications in Rust on **wgpu**. A Cargo workspace separates reusable engine code from application‑specific game code.

---

## 2. High‑Level Goals

* **Modularity** – clear boundary between generic engine and game logic.
* **Idiomatic Rust** – formatted, Clippy‑clean, documented.
* **Extensibility** – new rendering techniques, components, or game mechanics drop in easily.
* **Clarity** – newcomers understand the layout fast.

---

## 3. Core Architecture

| Crate    | Type    | Responsibility                                                      |
| -------- | ------- | ------------------------------------------------------------------- |
| `engine` | library | Reusable systems: ECS, renderer, scene I/O, scripting.              |
| `editor` | library | ImGui‑based scene editor (compiled only in dev builds via feature). |
| `game`   | binary  | Application‑specific systems, main loop, windowing.                 |

### 3.1 Engine Modules (inside `engine` crate)

| Module      | Role                                                |
| ----------- | --------------------------------------------------- |
| `core`      | `hecs` wrapper, math helpers (`glam`), logging, entity system with transform hierarchy.     |
| `graphics`  | WebGPU pipelines, depth, frame/render pass setup.   |
| `io`        | Asset loading, scene JSON load/save (`serde_json`). |
| `scripting` | Rhai engine, script registration helpers.           |
| `windowing` | Multi-window management, surface handling for viewports. |

---

## 4. Renderer Architecture

* **Per‑object rendering** – separate passes allow per‑entity transforms.
* **Uniform buffers** – model and camera matrices.
* **Depth testing** – 24‑bit depth buffer.
* **Lighting** – ambient + single directional light (simple shader).
* **Vertex buffer cache** – one buffer per mesh.
* **UI overlay** – editor draws with ImGui over the same surface.
* **Input** – Tab toggles UI versus game focus, Esc locks/unlocks mouse.

---

## 5. Directory Layout (top level)

```
.
├── assets/             # Models, textures, scripts
├── engine/             # Engine library
├── editor/             # Optional editor library
├── game/               # Game binary
├── justfile            # Command runner recipes
├── Cargo.toml          # Workspace definition
└── PLANNING.md         # This file
```

---

## 6. Build & Tooling

* Workspace managed by Cargo.
* `just run` – builds `game` (dev), links `editor` feature automatically.
* `just preflight` – `fmt`, `clippy -D warnings`, tests, docs.
* Release build excludes the editor: `cargo build --release`.

### 6.1 Build Modes

| Command                 | Linked crates            | Editor UI |
| ----------------------- | ------------------------ | --------- |
| `cargo run` (dev)       | engine, game, **editor** | ✔         |
| `cargo build --release` | engine, game             | ✖         |

---

## 7. Entity‑Component Basics

* **Entity** – opaque `hecs::Entity` ID.
* **Core components**

    * `Transform` – local position/rotation/scale. ✅ **Implemented**
    * `GlobalTransform` – world matrix (calculated each frame). ✅ **Implemented**
    * `Parent(Entity)` – establishes hierarchy. ✅ **Implemented**
    * `Camera` – projection parameters. ⏳ **Placeholder created**
    * `ScriptRef` – Rhai script path. ✅ **Implemented** (Basic support)
* Helper functions add mandatory dependencies (e.g. adding `Camera` auto‑adds `Transform`).
* **Hierarchy system** – Breadth-first traversal updates `GlobalTransform` each frame. ✅ **Implemented**
* **Cycle detection** – Prevents infinite loops in parent-child relationships. ✅ **Implemented**

### 7.1 Scene JSON example

```json
{
  "entities": [
    {
      "components": {
        "Transform": {"pos": [0,0,5], "rot": [0,0,0,1], "scale": [1,1,1]},
        "Camera":    {"fov": 60.0, "near": 0.1, "far": 500.0},
        "ScriptRef": {"name": "fly_camera"}
      }
    }
  ]
}
```

---

## 8. Code Style

* Format with `rustfmt` (`cargo fmt --all`).
* Zero Clippy warnings (`cargo clippy --workspace --all-targets --all-features -D warnings`).
* Naming

    * Modules/files: `snake_case`.
    * Types/traits: `PascalCase`.
    * Functions/vars: `snake_case`.
    * Constants: `UPPER_SNAKE_CASE`.
* Document all public items.

---

## 9. Testing

* Each feature ships unit tests.
* Tests live in a `tests` module in the same file.
* Run all tests with `cargo test --workspace`.

---

## 10. Key Decisions & Risks

* **ECS** – using `hecs` for simplicity; can migrate to `legion` or `bevy_ecs` if required.
* **Renderer abstraction** – currently tightly coupled to `wgpu`; may wrap in a `RenderContext` later.
* **Asset management** – direct file loads; may introduce an asset graph for larger projects.

---