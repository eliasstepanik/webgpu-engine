//! Scene Serialization Demo
//!
//! This example demonstrates the scene serialization system by:
//! 1. Creating a complex hierarchy of entities programmatically
//! 2. Saving the scene to a JSON file
//! 3. Loading the scene back into a new world
//! 4. Verifying the hierarchy is preserved

use engine::prelude::*;
use glam::Vec3;
use std::f32::consts::PI;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    engine::init_logging();

    // Create a world and build a demo scene
    let mut world = World::new();

    info!("Building demo scene...");

    // Camera entity (looking down at the scene)
    let _camera = world.spawn((
        Transform::from_position(Vec3::new(0.0, 8.0, 12.0)).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        Camera::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0),
    ));

    // Root object (center of scene)
    let root = world.spawn((
        Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    ));

    // Orbiting objects around the root
    let mut orbiters = Vec::new();
    for i in 0..4 {
        let angle = (i as f32) * PI * 0.5; // 90 degrees apart
        let radius = 3.0;

        let orbiter = world.spawn((
            Transform::from_position(Vec3::new(angle.cos() * radius, 1.0, angle.sin() * radius)),
            GlobalTransform::default(),
            Parent(root),
        ));

        orbiters.push(orbiter);

        // Each orbiter has a child satellite
        let satellite = world.spawn((
            Transform::from_position(Vec3::new(0.0, 1.5, 1.0)).with_scale(Vec3::splat(0.5)),
            GlobalTransform::default(),
            Parent(orbiter),
        ));

        // Some satellites have sub-satellites
        if i % 2 == 0 {
            let _sub_satellite = world.spawn((
                Transform::from_position(Vec3::new(0.5, 0.5, 0.0)).with_scale(Vec3::splat(0.3)),
                GlobalTransform::default(),
                Parent(satellite),
            ));
        }
    }

    // Ground plane
    let _ground = world.spawn((
        Transform::from_position(Vec3::new(0.0, -2.0, 0.0)).with_scale(Vec3::new(20.0, 0.1, 20.0)),
        GlobalTransform::default(),
    ));

    // Floating objects
    for i in 0..3 {
        let _floating = world.spawn((
            Transform::from_position(Vec3::new(
                (i as f32 - 1.0) * 4.0,
                3.0 + (i as f32) * 0.5,
                -6.0,
            )),
            GlobalTransform::default(),
        ));
    }

    info!("Scene statistics:");
    info!(entity_count = world.query::<()>().iter().count(), "Entities in scene");
    info!(transform_count = world.query::<&Transform>().iter().count(), "Entities with Transform");
    info!(parent_count = world.query::<&Parent>().iter().count(), "Entities with Parent");
    info!(camera_count = world.query::<&Camera>().iter().count(), "Entities with Camera");

    // Save the scene
    let scene_path = "assets/scenes/demo_scene_generated.json";
    info!(path = %scene_path, "Saving scene");
    world.save_scene(scene_path)?;

    // Load the scene into a new world
    info!("Loading scene into new world...");
    let mut new_world = World::new();
    let entity_mapper = new_world.load_scene_additive(scene_path)?;

    // Verify the scene loaded correctly
    info!("Scene loaded successfully");
    info!("New world statistics:");
    info!(entity_count = new_world.query::<()>().iter().count(), "Entities in new world");
    info!(transform_count = new_world.query::<&Transform>().iter().count(), "Entities with Transform in new world");
    info!(parent_count = new_world.query::<&Parent>().iter().count(), "Entities with Parent in new world");
    info!(camera_count = new_world.query::<&Camera>().iter().count(), "Entities with Camera in new world");

    // Show entity mapping
    info!("Entity ID mappings:");
    for (old_id, new_entity) in entity_mapper.iter().take(5) {
        info!(old_id = %old_id, new_entity = ?new_entity, "Entity ID mapping");
    }
    if entity_mapper.len() > 5 {
        info!(additional_mappings = entity_mapper.len() - 5, "Additional entity mappings");
    }

    // Verify parent-child relationships are preserved
    info!("Verifying parent-child relationships...");
    let parent_count = new_world.query::<&Parent>().iter().count();
    info!(parent_count, "Found parent-child relationships");

    // Check for any orphaned entities (parents that don't exist)
    let mut orphaned = 0;
    for (entity, parent) in new_world.query::<&Parent>().iter() {
        if !new_world.contains(parent.0) {
            orphaned += 1;
            info!(entity = ?entity, parent = ?parent.0, "Warning: Orphaned entity");
        }
    }

    if orphaned == 0 {
        info!("All parent references are valid");
    } else {
        info!(orphaned_count = orphaned, "Found orphaned entities");
    }

    // Show the JSON structure
    info!("Scene file structure:");
    let scene_content = std::fs::read_to_string(scene_path)?;
    let lines: Vec<&str> = scene_content.lines().take(10).collect();
    for line in lines {
        info!("  {line}");
    }
    if scene_content.lines().count() > 10 {
        info!(total_lines = scene_content.lines().count(), "Total lines in scene file");
    }

    info!("Demo completed successfully");
    info!(scene_path = %scene_path, "Try editing the scene file and loading it back");

    Ok(())
}
