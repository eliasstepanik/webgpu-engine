//! Test the physics_debug_test scene collision detection directly

use engine::core::entity::{update_hierarchy_system, World};
use engine::io::Scene;
use engine::physics::{
    avbd_solver::{AVBDConfig, AVBDSolver},
    systems::{detect_all_collisions, gather_colliders, gather_rigidbodies},
    PhysicsConfig,
};
use tracing::{debug, info};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("engine::physics=debug,test_physics_debug=info")
        .init();

    info!("=== Physics Debug Test ===");

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

    info!("Scene instantiated, running physics analysis...");

    // Create physics config and solver
    let physics_config = PhysicsConfig::default();
    let avbd_config = AVBDConfig::default();
    let mut solver = AVBDSolver::new(avbd_config);

    // Gather rigidbodies
    let (bodies, body_entity_map) = gather_rigidbodies(&world);
    info!("Found {} rigidbodies", bodies.len());

    for (i, body) in bodies.iter().enumerate() {
        debug!(
            "Body {}: entity={:?}, pos={:?}, mass={}, kinematic={}",
            i, body.entity, body.position, body.mass, body.is_kinematic
        );
    }

    // Gather colliders
    let colliders = gather_colliders(&world, &body_entity_map);
    info!("Found {} colliders", colliders.len());

    for (i, collider) in colliders.iter().enumerate() {
        debug!(
            "Collider {}: entity={:?}, pos={:?}, body_idx={:?}",
            i, collider.entity, collider.position, collider.body_index
        );
    }

    // Detect collisions
    let contacts = detect_all_collisions(&colliders, &bodies);
    info!("Detected {} contacts", contacts.len());

    for (i, contact) in contacts.iter().enumerate() {
        info!(
            "Contact {}: {:?} <-> {:?}",
            i, contact.entity_a, contact.entity_b
        );
        info!("  Position: {:?}", contact.position);
        info!("  Normal: {:?}", contact.normal);
        info!("  Penetration: {}", contact.penetration);
    }

    if contacts.is_empty() {
        info!("❌ No contacts detected - investigating...");

        // Check AABBs manually
        for (i, collider_a) in colliders.iter().enumerate() {
            for (j, collider_b) in colliders.iter().enumerate().skip(i + 1) {
                let aabb_a = collider_a
                    .collider
                    .shape
                    .world_aabb(collider_a.position, collider_a.rotation);
                let aabb_b = collider_b
                    .collider
                    .shape
                    .world_aabb(collider_b.position, collider_b.rotation);

                info!("AABB check: {} vs {}", i, j);
                info!("  A: {:?} to {:?}", aabb_a.min, aabb_a.max);
                info!("  B: {:?} to {:?}", aabb_b.min, aabb_b.max);
                info!("  Overlap: {}", aabb_a.overlaps(&aabb_b));
            }
        }
    } else {
        info!("✅ Contacts detected successfully!");
    }
}
