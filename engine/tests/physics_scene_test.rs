//! Integration test for physics scene loading

use engine::core::entity::{update_hierarchy_system, World};
use engine::io::Scene;
use engine::physics::{components::Rigidbody, systems::update_physics_system};
use std::path::Path;

#[test]
fn test_physics_scene_loading() {
    // Load the physics demo scene
    let scene_path = Path::new("../game/assets/scenes/physics_demo.json");
    let scene = Scene::load_from_file(scene_path).expect("Failed to load scene");

    // Create world and instantiate scene
    let mut world = World::new();
    scene
        .instantiate(&mut world)
        .expect("Failed to instantiate scene");

    // Count rigidbodies before hierarchy update
    let bodies_before = world.query::<&Rigidbody>().iter().count();
    println!("Rigidbodies before hierarchy update: {bodies_before}");

    // Run hierarchy system to create GlobalTransform components
    engine::core::entity::hierarchy::advance_frame();
    update_hierarchy_system(&mut world);

    // Count rigidbodies after hierarchy update
    let bodies_after = world.query::<&Rigidbody>().iter().count();
    println!("Rigidbodies after hierarchy update: {bodies_after}");

    // Create physics solver
    let mut solver = engine::physics::systems::create_default_solver();

    // Run physics update - should not panic and should find rigidbodies
    update_physics_system(
        &mut world,
        &mut solver,
        &engine::physics::PhysicsConfig::default(),
        0.016,
    );

    // Verify we still have the same number of rigidbodies
    assert_eq!(bodies_before, bodies_after, "Rigidbody count changed!");
    assert!(bodies_after > 0, "No rigidbodies found in scene");
}
