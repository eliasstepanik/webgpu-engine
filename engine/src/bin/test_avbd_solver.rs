//! Test AVBD solver with overlapping objects

use engine::core::entity::{update_hierarchy_system, World};
use engine::io::Scene;
use engine::physics::{
    avbd_solver::{AVBDConfig, AVBDSolver},
    systems::{
        detect_all_collisions, gather_colliders, gather_rigidbodies, update_physics_system_avbd,
    },
};
use tracing::info;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("engine::physics=debug,test_avbd_solver=info")
        .init();

    info!("=== AVBD Solver Test ===");

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

    info!("Scene instantiated, running AVBD solver...");

    // Create physics config and solver
    let avbd_config = AVBDConfig::default();
    let mut solver = AVBDSolver::new(avbd_config);

    // Run a few physics steps to see if objects are resolved
    let dt = 1.0 / 120.0; // 120 Hz
    for step in 0..10 {
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

        // Check if we still have penetration
        let colliders = gather_colliders(&world, &body_map);
        let contacts = detect_all_collisions(&colliders, &bodies);
        info!("Contacts after step {}: {}", step, contacts.len());

        for contact in &contacts {
            info!(
                "  Contact: pos={:?}, normal={:?}, penetration={}",
                contact.position, contact.normal, contact.penetration
            );
        }
    }

    info!("AVBD solver test completed");
}
