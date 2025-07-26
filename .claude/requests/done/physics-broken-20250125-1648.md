## FEATURE:
Fix persistent physics drift and instability issues in AVBD solver causing objects to move randomly without forces

## EXAMPLES:
.claude/examples/console-output-before.txt – Shows physics error logs before attempted fixes
.claude/examples/console-output-after.txt – Shows persistent issues after previous fix attempts

## DOCUMENTATION:
https://github.com/Raikiri/LegitParticles – AVBD constraint-based dynamics solver reference implementation
https://www.gamedev.net/tutorials/programming/math-and-physics/understanding-constraint-resolution-in-physics-engine-r4839/ – Constraint solver drift issues
https://kevinyu.net/2018/01/17/understanding-constraint-solver-in-physics-engine/ – Position drift in physics engines
https://research.ncl.ac.uk/game/mastersdegree/gametechnologies/physicstutorials/8constraintsandsolvers/Physics%20-%20Constraints%20and%20Solvers.pdf – Constraints and solvers fundamentals
https://mmacklin.com/EG2015PBD.pdf – Position-based dynamics methods

## OTHER CONSIDERATIONS:
- AVBD solver has sensitive parameters (beta=10.0, k_start=5000.0) that are scene-dependent not dimensionless
- Current implementation comment states "AVBD physics update system (currently broken, needs fixes)"
- Physics accumulator uses unsafe static mutable state which may cause timing issues
- Transform interpolation affects all entities despite recent fix to only process Rigidbody entities
- Multiple failing hierarchy tests indicate transform propagation issues affecting physics
- store_previous_transforms and interpolate_transforms are currently stub implementations
- No warmstarting implementation despite AVBD requiring it for stability (gamma=0.99 warmstart decay unused)