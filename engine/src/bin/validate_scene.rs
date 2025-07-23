//! Quick scene validation utility

use engine::io::Scene;
use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    let scene_path = if args.len() > 1 {
        &args[1]
    } else {
        "game/assets/scenes/physics_debug_test.json"
    };

    let path = Path::new(scene_path);
    println!("Validating scene: {}", path.display());

    match Scene::load_from_file(path) {
        Ok(scene) => {
            println!("✓ Scene loaded successfully!");
            println!("  Entity count: {}", scene.entities.len());

            let mut world = engine::core::entity::World::new();
            match scene.instantiate(&mut world) {
                Ok(_) => {
                    println!("✓ Scene instantiated successfully!");

                    // Count components
                    let rigid_bodies = world
                        .query::<&engine::physics::components::Rigidbody>()
                        .iter()
                        .count();
                    let colliders = world
                        .query::<&engine::physics::components::Collider>()
                        .iter()
                        .count();
                    let cameras = world
                        .query::<&engine::core::camera::Camera>()
                        .iter()
                        .count();

                    println!("  Rigidbodies: {rigid_bodies}");
                    println!("  Colliders: {colliders}");
                    println!("  Cameras: {cameras}");
                }
                Err(e) => {
                    eprintln!("✗ Failed to instantiate scene: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load scene: {e}");
        }
    }
}
