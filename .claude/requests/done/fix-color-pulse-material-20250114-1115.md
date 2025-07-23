## FEATURE:
Fix color pulse script by using Material constructor functions instead of attempting to modify immutable color arrays

## EXAMPLES:
game/assets/scripts/color_pulse.rhai – Script attempting to modify material colors but failing due to immutable arrays
game/assets/scripts/rotating_cube.rhai – Another script with the same material modification pattern that needs fixing
engine/src/scripting/modules/world.rs – Shows Material type registration with only getter, no setter for color array

## DOCUMENTATION:
https://rhai.rs/book/rust/custom-types.html#getters-setters-and-indexers
https://rhai.rs/book/rust/register-raw.html#fallible-getters-setters-and-indexers
https://docs.rs/rhai/latest/rhai/struct.Engine.html#method.register_type_with_name

## OTHER CONSIDERATIONS:
- Material color array is read-only in Rhai scripts (only getter registered, no setter)
- Must use Material::from_rgba() constructor to create new materials instead of modifying existing ones
- rotating_cube.rhai has the same issue when trying to apply tint_color
- Command queue system works correctly - the issue is purely in the script's approach
- No existing examples demonstrate the correct pattern for material color changes
- Consider adding a Material::with_color() helper or registering setters for better ergonomics