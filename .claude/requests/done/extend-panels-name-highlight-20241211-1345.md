## FEATURE:
Extend editor panels with Name component for entities, display names in hierarchy, make inspector editable with component dropdown, and highlight selected entities in viewport

## EXAMPLES:
No examples available in .claude/examples/ directory

## DOCUMENTATION:
https://github.com/ocornut/imgui/issues/2193 - ImGui tree node selection highlighting
https://github.com/ocornut/imgui/issues/190 - Selected/Highlighted TreeNode discussion
https://docs.rs/imgui/latest/imgui/struct.ComboBox.html - imgui-rs ComboBox API documentation
https://github.com/imgui-rs/imgui-rs - imgui-rs Rust bindings for Dear ImGui
https://ameye.dev/notes/edge-detection-outlines/ - Edge detection outline techniques for 3D rendering
https://roystan.net/articles/outline-shader/ - Outline shader implementation tutorial
https://omar-shehata.medium.com/better-outline-rendering-using-surface-ids-with-webgl-e13cdab1fd94 - Surface ID-based outline rendering
https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/ - WGPU pipeline creation guide
https://whoisryosuke.com/blog/2022/render-pipelines-in-wgpu-and-rust - Render pipelines in wgpu and Rust

## OTHER CONSIDERATIONS:
- No Name component currently exists in the engine - entities are identified only by debug ID
- Inspector panel is read-only - needs conversion to use with_world_write for editing
- Component registry exists at engine/src/io/component_registry.rs with 6 registered components
- ImGui ComboBox requires begin()/end() pattern or build() with closure
- Need to filter already-attached components from dropdown list
- Component addition requires type-erased string matching due to Rust's static typing
- Consider using drag_float3 for Vec3 fields, drag_float for scalars, color picker for materials
- Quaternion rotation needs euler angle conversion for user-friendly editing
- No selection visualization exists in the viewport renderer
- Two main approaches for outline rendering: post-processing edge detection or multi-pass rendering
- Edge detection can use depth buffer, normal buffer, or surface IDs
- WGPU requires creating new render pipelines for additional render passes
- Selection state is already tracked in EditorSharedState but needs viewport integration
- Consider performance impact of additional render passes for outline effect
- May need to extend Material component or create SelectionMaterial for highlighted entities
- Component removal needs confirmation dialog to prevent accidental deletion