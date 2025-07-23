## FEATURE:

Fix physics collision detection failing due to scene configuration causing objects to never physically overlap despite correct collision pipeline

## EXAMPLES:

.claude/examples/console-output-before.txt – Shows debug output before physics fixes
.claude/examples/console-output-after.txt – Shows debug output after physics fixes

## DOCUMENTATION:

https://news.ycombinator.com/item?id=44334403 – AVBD solver penetration-free guarantees
https://gamedev.stackexchange.com/questions/22310/how-to-resolve-penetration-of-two-colliding-bodies – Position correction methods
https://pybullet.org/Bullet/phpBB3/viewtopic.php?t=10945 – CFM/ERP vs Baumgarte stabilization
https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection – 3D collision detection fundamentals

## OTHER CONSIDERATIONS:

- Floor in physics_debug_test.json has Y position -1.0 with scale 0.2, resulting in top surface at Y=-0.9
- Objects starting at Y=5.0 with half_extents 0.5 have bottom at Y=4.5, creating 5.4 unit gap
- Collision detection correctly identifies no AABB overlap between floor (-1.1 to -0.9) and falling objects
- AVBD solver requires GlobalTransform components which are now created by hierarchy system before physics
- NGS position correction was using hardcoded Vec3::Z instead of actual contact normal (fixed)
- Velocity clamping added to prevent tunneling through thin objects (max 100 units/s)
- Physics runs at 120Hz fixed timestep but objects may not fall fast enough within simulation time
- Consider adjusting scene geometry, gravity strength, or simulation duration rather than physics code