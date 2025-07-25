//! Test reduced gravity physics

use engine::core::entity::{update_hierarchy_system, World};
use engine::io::Scene;
use engine::physics::{
    avbd_solver::{AVBDConfig, AVBDSolver},
    systems::{
        detect_all_collisions, gather_colliders, gather_rigidbodies, update_physics_system_avbd,
    },
    PhysicsConfig,
};
use glam::Vec3;
use tracing::info;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("engine::physics=debug,test_reduced_gravity=info")
        .init();

    info!("=== Reduced Gravity Physics Test ===");

    // Load the scene
    let scene = Scene::load_from_file("game/assets/scenes/physics_debug_test.json")
        .expect("Failed to load physics_debug_test.json");

    info!("Loaded scene with {} entities", scene.entities.len());

    // Create world and instantiate scene
    let mut world = World::new();
    scene
        .instantiate(&mut world)
        .expect("Failed to instantiate scene");

    // Update hierarchy system first to create GlobalTransform
    engine::core::entity::hierarchy::advance_frame();
    update_hierarchy_system(&mut world);

    info!("Scene instantiated, testing with reduced gravity...");

    // Create custom physics config with reduced gravity
    let mut physics_config = PhysicsConfig::default();
    physics_config.gravity = Vec3::new(0.0, -2.0, 0.0); // Much gentler than -9.81

    info!("Using custom gravity: {:?}", physics_config.gravity);

    let avbd_config = AVBDConfig {
        iterations: physics_config.velocity_iterations,
        beta: 10.0,
        alpha: 0.98,
        gamma: 0.99,
        k_start: 5000.0,
        gravity: physics_config.gravity,
    };
    let mut solver = AVBDSolver::with_physics_config(avbd_config, &physics_config);

    // Run several physics steps to see slower motion
    let dt = 1.0 / 120.0; // 120 Hz
    for step in 0..5 {
        info!("=== Physics Step {} ===", step);

        update_physics_system_avbd(&mut world, &mut solver, dt);

        // Check positions after this step
        let (bodies, body_map) = gather_rigidbodies(&world);
        for body in &bodies {
            info!(
                "Body {:?}: pos={:?}, vel={:?}",
                body.entity, body.position, body.linear_velocity
            );
        }

        // Check contacts
        let colliders = gather_colliders(&world, &body_map);
        let contacts = detect_all_collisions(&colliders, &bodies);
        info!("Contacts: {}", contacts.len());
    }

    info!("Reduced gravity physics test completed - objects should fall more slowly");
}
