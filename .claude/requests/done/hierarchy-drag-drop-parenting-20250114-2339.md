## FEATURE:

Add drag-and-drop functionality to the hierarchy panel for creating parent-child entity relationships in the editor

## EXAMPLES:

editor/src/panels/assets.rs – Shows existing drag-drop implementation pattern with static state storage
editor/src/panels/hierarchy.rs – Target file that needs drag-drop functionality added for entity parenting

## DOCUMENTATION:

https://docs.rs/imgui/latest/imgui/drag_drop/index.html – ImGui drag-drop API reference
https://github.com/imgui-rs/imgui-rs/blob/main/imgui-examples/examples/drag_drop.rs – ImGui-rs drag-drop examples
https://docs.rs/hecs/latest/hecs/ – HECS ECS documentation for entity management

## OTHER CONSIDERATIONS:

- Must use EditorSharedState's with_world_write() for safe World mutations
- Need to prevent cycles (dragging parent onto its own child)
- Static state pattern required for storing dragged entity between frames
- Should show visual feedback (hover rectangles) during drag operation
- Parent component removal needs world.inner.remove_one::<Parent>(entity) pattern
- Consider supporting multi-selection drag in future iterations
- Hierarchy updates are automatic via hierarchy system after Parent changes