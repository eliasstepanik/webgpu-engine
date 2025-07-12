## FEATURE:

Fix scripting system limitations by implementing command pattern for safe world access and enabling scripts to modify ECS components

## EXAMPLES:

assets/scripts/rotating_cube.rhai – demonstrates non-functional world API calls that need fixing
assets/scripts/fly_camera.rhai – shows input handling but cannot actually modify camera transform
engine/src/scripting/modules/world.rs – contains placeholder implementations that need replacing

## DOCUMENTATION:

https://github.com/rhaiscript/rhai/issues/95 – Discussion about Rhai ECS integration
https://docs.rs/bevy/latest/bevy/ecs/system/struct.Commands.html – Bevy's command pattern implementation
https://docs.rs/bevy/latest/bevy/ecs/system/struct.Deferred.html – Deferred execution pattern in Bevy
https://rhai.rs/book/about/features.html – Rhai features including sync support

## OTHER CONSIDERATIONS:

- Rhai already has sync feature enabled in Cargo.toml but raw pointers in World prevent Send+Sync
- Current world module creates placeholder functions that return hardcoded values
- Scripts cannot query entities, create/destroy entities, or access components beyond Transform/Material/Name
- Need thread-safe wrapper or command queue to batch mutations after script execution
- Consider message passing between scripts rather than direct world access
- Bevy's Commands/Deferred pattern provides good reference implementation
- Performance impact of deferred mutations should be measured
- Example scripts contain misleading code that appears functional but does nothing