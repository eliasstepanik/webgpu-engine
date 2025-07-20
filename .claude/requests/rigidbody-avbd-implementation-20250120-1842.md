## FEATURE:
Implement rigidbody physics component using Augmented Vertex Block Descent (AVBD) algorithm from SIGGRAPH 2025 paper

## EXAMPLES:
.claude/examples/simple-rigidbody-scene.json – Basic rigidbody falling under gravity
.claude/examples/stacking-rigidbodies.json – Rigidbody stacking and collision test
.claude/examples/joint-constraints.json – Connected rigidbodies with constraints

## DOCUMENTATION:
.claude/documentation/Augmented_VBD-SIGGRAPH25.pdf – Original AVBD paper
.claude/documentation/Augmented_VBD-SIGGRAPH25_RTL.pdf – Ready-To-Learn version with clearer explanations
https://graphics.cs.utah.edu/research/projects/avbd/ – AVBD project page with examples
https://github.com/savant117/avbd-demo2d – 2D C++ reference implementation
https://docs.rs/glam/latest/glam/ – Math library already used in project
https://github.com/dimforge/rapier – Reference for physics engine design patterns
https://box2d.org/documentation/ – Reference for constraint solver architecture

## OTHER CONSIDERATIONS:
- AVBD algorithm excels at stiff constraints and stable stacking with only 3-5 iterations
- Requires vertex coloring for parallelization - can use rayon for parallel iteration
- Must integrate with existing Transform/WorldTransform component hierarchy
- Engine uses f32 for rendering but has f64 WorldTransform for large worlds - physics should support both
- No existing collision detection system - will need basic collision shapes (sphere, box, capsule)
- Component system requires registration in engine_derive hardcoded list
- Physics update should be called in EngineApp::update() before hierarchy update
- Use tracing crate for logging, not println!
- Key AVBD parameters: β=10, α=0.95, γ=0.99, k_start>0
- Warm-starting critical: scale previous stiffness/dual variables by γ=0.99
- Use LDLT decomposition for 6x6 linear solves (mass/inertia matrix)
- Contact constraints need special handling: normal force λ≥0, friction bounded by μ*λ_n
- Quaternion operations need special addition/subtraction (see paper equations 20-21)
- Progressive stiffness ramping prevents early divergence with stiff constraints
- Friction and restitution should be material properties, not per-rigidbody
- Reference implementation shows Taylor expansion for constraint approximation improves performance
- Use LDL decomposition for 6x6 systems (more stable than direct inversion)
- Implement constraint warmstarting with 0.99 decay factor between frames
- Consider GPU compute shaders for parallel primal updates using vertex coloring
- Contact persistence important for stable friction (track contact IDs across frames)
- Diagonal Hessian approximation prevents indefinite matrices while maintaining stability