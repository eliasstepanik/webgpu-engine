## FEATURE:

Fix camera drift issue when parenting camera entities to other entities in the hierarchy system

## EXAMPLES:

.claude/examples/console-output-after.txt – shows debug output confirming math is correct but drift still occurs
.claude/examples/console-output-before.txt – baseline console output before parenting operations

## DOCUMENTATION:

https://www.scratchapixel.com/lessons/mathematics-physics-for-computer-graphics/geometry/row-major-vs-column-major-order
https://webgpufundamentals.org/webgpu/lessons/webgpu-cameras.html
https://www.khronos.org/opengl/wiki/GluLookAt_code
https://gamedev.stackexchange.com/questions/178643/decompose-matrix-to-translation-rotation-and-scale

## OTHER CONSIDERATIONS:

- Debug output shows transform math is correct: camera world pos remains (0.0, 2.0, 5.0) after parenting
- Issue is specific to camera entities - regular entities parent correctly without drift
- Camera view matrix is inverse of world transform, which may introduce precision issues
- Renderer uses camera-relative positioning for all entities to handle large world coordinates
- Matrix decomposition/reconstruction in renderer (lines 288-304) might introduce precision errors
- System supports mixed Transform (32-bit) and WorldTransform (64-bit) hierarchies
- Rotation quaternions are normalized after decomposition but drift still occurs
- Debug logging already exists in hierarchy.rs specifically for camera parenting issues