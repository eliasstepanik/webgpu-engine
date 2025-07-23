//! CLI tool for validating physics scene configurations
//!
//! Usage: cargo run --bin validate_physics_scene <scene.json>

use engine::io::Scene;
use engine::physics::scene_validator::{validate_physics_scene, ErrorType};
use std::env;
use std::fs;
use std::process;

fn main() {
    // Initialize logging for debug output
    tracing_subscriber::fmt()
        .with_env_filter("engine::physics::scene_validator=debug")
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <scene.json>", args[0]);
        eprintln!("\nValidates physics scene configuration for collision detection issues.");
        eprintln!("\nExamples:");
        eprintln!("  {} game/assets/scenes/physics_debug_test.json", args[0]);
        eprintln!("  {} game/assets/scenes/physics_*.json", args[0]);
        process::exit(1);
    }

    let mut all_valid = true;
    
    // Process each scene file argument
    for scene_path in &args[1..] {
        println!("=== Physics Scene Validation Report ===");
        println!("Scene: {}", scene_path);
        
        // Load scene
        let scene = match load_scene(scene_path) {
            Ok(scene) => scene,
            Err(e) => {
                eprintln!("ERROR: Failed to load scene '{}': {}", scene_path, e);
                all_valid = false;
                continue;
            }
        };
        
        // Validate
        let result = validate_physics_scene(&scene);
        
        // Report results
        println!("Valid: {}", result.is_valid);
        
        if !result.errors.is_empty() {
            println!("\nERRORS:");
            for error in &result.errors {
                println!("  - {}: {}", error.entity_name, error.details);
                
                // Provide specific fix suggestions
                match &error.error_type {
                    ErrorType::NoOverlap { gap_distance } => {
                        println!("    Fix: Reduce gap by moving floor up or object down");
                        println!("    Current gap: {:.1}m", gap_distance);
                    }
                    ErrorType::FloatingObject { height } => {
                        println!("    Fix: Add a floor below object at Y < {:.1}", height);
                    }
                    ErrorType::InitialPenetration => {
                        println!("    Fix: Separate overlapping objects in scene");
                    }
                    ErrorType::InvalidScale { scale } => {
                        println!("    Fix: Use reasonable scale values, got {:?}", scale);
                    }
                    ErrorType::MissingCollider => {
                        println!("    Fix: Add Collider component to physics object");
                    }
                }
            }
            all_valid = false;
        }
        
        if !result.warnings.is_empty() {
            println!("\nWARNINGS:");
            for warning in &result.warnings {
                println!("  - {}: {}", warning.entity, warning.warning);
            }
        }
        
        if !result.suggestions.is_empty() {
            println!("\nSUGGESTIONS:");
            for suggestion in &result.suggestions {
                println!("  - {}", suggestion);
            }
        }
        
        // Add scene-specific guidance
        if !result.is_valid {
            println!("\nTROUBLESHOoting:");
            println!("  1. Run the scene with: SCENE={} cargo run", 
                     scene_path.strip_prefix("game/assets/scenes/").unwrap_or(scene_path).strip_suffix(".json").unwrap_or(scene_path));
            println!("  2. Check if objects actually collide");
            println!("  3. Use debug visualization to see collision shapes");
            println!("  4. Consider using physics_working_test.json as template");
        }
        
        println!(); // Empty line between scenes
    }
    
    // Exit with error code if any scene was invalid
    if !all_valid {
        println!("❌ Some scenes have validation errors");
        process::exit(1);
    } else {
        println!("✅ All scenes passed validation");
    }
}

/// Load a scene from file
fn load_scene(path: &str) -> Result<Scene, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let scene: Scene = serde_json::from_str(&contents)?;
    Ok(scene)
}