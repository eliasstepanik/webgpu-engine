//! Rhai modules for exposing engine functionality to scripts

pub mod input;
pub mod math;
pub mod mesh;
pub mod physics;
pub mod profiling;
pub mod world;

use rhai::Engine;
use tracing::debug;

/// Register all modules with the Rhai engine
pub fn register_all_modules(engine: &mut Engine) {
    debug!("Registering scripting modules");

    // Register math types and functions
    math::register_math_types(engine);

    // Register world API
    world::register_world_api(engine);

    // Register input API
    input::register_input_api(engine);

    // Register mesh API
    mesh::register_mesh_api(engine);

    // Register physics API
    physics::register_physics_api(engine);

    // Register profiling API
    profiling::register_profiling_api(engine);

    debug!("All scripting modules registered");
}
