## FEATURE:
Fix critical physics system bugs including disabled AVBD solver, incorrect contact points, penetration jitter, and velocity damping issues

## EXAMPLES:
.claude/prp/rigidbody-avbd-physics.md – AVBD physics implementation plan showing intended solver design
.claude/prp/physics-collider-visualization.md – Debug visualization plan for troubleshooting physics issues
game/assets/scenes/physics_stacking.json – Demonstrates unstable stacking and interpenetration bugs
game/assets/scenes/physics_cube_tip.json – Shows rotation physics not working correctly
engine/tests/physics_rotation_test.rs – Failing test for angular dynamics
engine/tests/physics_rest_test.rs – Test showing objects not settling at correct positions

## DOCUMENTATION:
https://github.com/Raikiri/LegitParticles – AVBD implementation reference
https://www.franksworld.com/2025/07/15/how-roblox-solved-the-physics-problem-that-stumped-everyone/ – AVBD algorithm overview
https://gamedev.stackexchange.com/questions/131219/rigid-body-physics-resolution-causing-never-ending-bouncing-and-jittering – Common jitter solutions
https://www.gamedev.net/tutorials/programming/math-and-physics/understanding-constraint-resolution-in-physics-engine-r4839/ – Constraint resolution fundamentals

## OTHER CONSIDERATIONS:
- AVBD solver completely bypassed in systems.rs with TODO comment – entire advanced solver unused
- Contact point calculation returns incorrect world positions (e.g., Vec3(-4.8, 0.4, -5.2) for object at origin)
- Multiple hardcoded velocity thresholds (0.1, 0.05) causing objects to stick or stop unexpectedly
- Transform vs GlobalTransform inconsistency causes wrong positions for hierarchical objects
- Penetration correction uses partial factor (0.8) leaving objects slightly interpenetrating
- No configurable physics parameters – gravity, restitution, damping all hardcoded
- Debug visualization exists but not connected to physics loop
- Broad phase using partial_cmp could hide NaN values in positions
- AVBD has "magical parameters" requiring careful tuning to avoid explosions
- Need simultaneous contact resolution to prevent jitter
- Allow small penetration (0.001-0.004) to maintain stable contacts
- Implement warmstarting for constraint solver convergence
- Position correction should use NGS instead of Baumgarte stabilization