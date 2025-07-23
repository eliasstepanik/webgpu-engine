## FEATURE:
Fix physics simulation stability issues and implement collider visualization in viewport

## EXAMPLES:
.claude/examples/console-output-before.txt – Shows physics log output with collision detection
.claude/examples/console-output-after.txt – Expected output after physics fixes
examples/scene_demo.rs – Basic scene rendering without physics

## DOCUMENTATION:
https://graphics.cs.utah.edu/research/projects/avbd/ – Official AVBD physics solver paper
https://github.com/Raikiri/LegitParticles – AVBD implementation reference
https://github.com/savant117/avbd-demo2d – 2D AVBD reference implementation
https://github.com/gfx-rs/wgpu/discussions/1818 – wgpu wireframe rendering techniques
https://github.com/m-schuetz/webgpu_wireframe_thicklines – WebGPU wireframe with thick lines
https://www.gijskaerts.com/wordpress/?p=190 – GPU-driven debug line renderer

## OTHER CONSIDERATIONS:
- AVBD solver has scene-dependent parameters (beta, stiffness) that require careful tuning to avoid explosions
- Fixed timestep with interpolation recommended over variable frame-based delta time
- Determinant threshold (1e-6) in matrix inversion may be too strict for some scales
- wgpu wireframe requires NON_FILL_POLYGON_MODE feature which isn't universally supported
- Vertex pulling technique allows efficient wireframe rendering without geometry duplication
- Debug rendering should use transparent/additive blending to see through objects
- Consider caching wireframe meshes for identical collision shapes
- Color coding recommended: green (static), blue (dynamic), red (triggers), yellow (colliding)