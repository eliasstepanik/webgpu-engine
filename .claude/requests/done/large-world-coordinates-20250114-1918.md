## FEATURE:

Implement large world coordinate support to handle game worlds beyond the precision limits of single-precision floating point.

## EXAMPLES:

.claude/examples/transform-hierarchy.rs - Shows current transform system using Vec3 (f32)
.claude/examples/camera-projection.rs - Demonstrates camera setup with near/far planes

## DOCUMENTATION:

https://docs.godotengine.org/en/stable/tutorials/physics/large_world_coordinates.html - Godot's implementation of origin shifting and double precision
https://dev.epicgames.com/documentation/en-us/unreal-engine/large-world-coordinates-in-unreal-engine-5 - UE5's Large World Coordinates (LWC) system
https://docs.flaxengine.com/manual/editor/large-worlds/index.html - Flax Engine's double precision approach
https://gamedev.stackexchange.com/questions/104139/how-can-i-represent-location-in-a-massive-world - Overview of techniques including origin shifting
https://gamedevtricks.com/post/origin-rebasing-space/ - Deep dive into spatial rebasing in Unreal
https://docs.rs/wgpu/latest/wgpu/enum.VertexFormat.html - wgpu vertex format limitations (no double precision vertex formats)

## OTHER CONSIDERATIONS:

- Current implementation uses glam::Vec3 (f32) with precision loss at ~16,777,216 units
- Camera near=0.1, far=1000.0 limits usable range and causes z-fighting
- wgpu primarily supports single precision vertex data - double precision requires workarounds
- GPU performance: double precision can be 2-32x slower than single precision on consumer GPUs
- Origin shifting breaks in multiplayer where server needs precision for all players
- Transform hierarchy system would need updates to handle origin shifts
- Shader modifications required: vertex positions are vec3<f32> in WGSL
- Consider hybrid approach: double precision on CPU, single precision on GPU with camera-relative rendering
- Physics engines (if added later) typically use single precision only
- Existing scenes and scripts assume absolute world coordinates