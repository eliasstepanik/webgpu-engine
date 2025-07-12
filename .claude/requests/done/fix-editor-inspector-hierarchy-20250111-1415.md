## FEATURE:
Fix editor inspector and hierarchy panels not showing entity components and names

## DESCRIPTION:
The editor's Inspector and Hierarchy panels are not functioning correctly:
1. Inspector panel shows "No entity selected" or fails to display components when an entity is selected
2. Hierarchy panel shows all entities as "Entity ... [No Transform]" instead of showing proper names
3. Both panels should be displaying the Name components and other components that were added to entities

## CURRENT BEHAVIOR:
- Entities are created with Name, Transform, Material, Camera, and MeshId components in both demo scene and default scene
- Hierarchy panel shows entities but without proper names (displays as "Entity ... [No Transform]")
- Inspector panel either shows "No entity selected" or doesn't display any components when an entity is selected
- Debug logging shows that entities exist in the world but component checking appears to fail

## EXPECTED BEHAVIOR:
- Hierarchy panel should show entity names like "Main Camera", "Center Cube", "Ground Plane", etc.
- Inspector panel should show collapsible headers for each component (Name, Transform, Material, Camera, Mesh)
- Selecting an entity in hierarchy should populate the inspector with that entity's components
- Component values should be editable in the inspector

## TECHNICAL DETAILS:
- World access is done through `EditorSharedState` with `with_world_read` and `with_world_write` methods
- Component checking logic exists but may not be working correctly
- Recent fixes have been made to avoid nested world access but the core issue persists

## FILES MODIFIED:
- `game/src/main.rs` - Added Name components to demo scene entities
- `editor/src/scene_operations.rs` - Added Name components to default scene entities  
- `editor/src/panels/inspector.rs` - Fixed component checking logic to avoid nested world access
- `editor/src/panels/hierarchy.rs` - Added debug logging

## INVESTIGATION NEEDED:
1. Verify that entities are actually being created with the expected components
2. Check if world access through SharedState is working correctly
3. Verify that the hierarchy and inspector are reading from the same world instance
4. Check if there are any issues with the ECS component registration or queries

## EXAMPLES:
.claude/examples/ecs-component-query.rs – shows proper ECS component querying patterns
.claude/examples/imgui-debug-panel.rs – demonstrates debugging UI state

## DOCUMENTATION:
https://docs.rs/hecs/latest/hecs/ – ECS library documentation
https://github.com/hecrj/imgui-rs – ImGui Rust bindings

## OTHER CONSIDERATIONS:
- Thread safety with Arc<Mutex<World>> may be affecting performance but shouldn't break functionality
- Static mutable state warning in inspector.rs should be addressed but isn't blocking
- Debug logging has been added to track component existence and world state