## FEATURE:

**Transform Component & Hierarchy System**

* Add `Transform`, `GlobalTransform`, and `Parent` components.
* Implement a system that:

    * Traverses `Parent` relationships breadth-first.
    * Writes each entity’s world matrix into `GlobalTransform`.
* Provide helper API (`add_camera`, `add_with_requirements`) that auto-inserts missing mandatory components.

## DOCUMENTATION:

* `hecs` crate docs – entity creation, hierarchical queries. ([docs.rs](https://docs.rs/hecs/?utm_source=chatgpt.com))
* `glam` crate docs – matrices and quaternion math. ([docs.rs](https://docs.rs/glam/latest/glam/?utm_source=chatgpt.com))
* Future scripting integration will call Rhai; keep `GlobalTransform` `Send + Sync`. ([docs.rs](https://docs.rs/rhai/latest/rhai/?utm_source=chatgpt.com))

## OTHER CONSIDERATIONS:

* Detect and log cyclic parenting to avoid stack overflows.
* Store parent links as plain `hecs::Entity`; invalid on deserialization until remap – add a post-load fix-up step.
* Update order matters: run hierarchy before rendering and scripts each frame.
* Avoid allocating during traversal; reuse a `Vec<hecs::Entity>` per frame.
* Derive `Serialize`/`Deserialize` for components so scenes round-trip through JSON.
