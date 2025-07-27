//! Quick scene validation utility

use engine::io::Scene;
use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();
    let scene_path = if args.len() > 1 {
        &args[1]
    } else {
        "game/assets/scenes/test_scene.json"
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
                    let transforms = world
                        .query::<&engine::core::entity::Transform>()
                        .iter()
                        .count();
                    let cameras = world
                        .query::<&engine::core::camera::Camera>()
                        .iter()
                        .count();
                    let meshes = world.query::<&engine::graphics::MeshId>().iter().count();

                    println!("  Transforms: {transforms}");
                    println!("  Cameras: {cameras}");
                    println!("  Meshes: {meshes}");
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
