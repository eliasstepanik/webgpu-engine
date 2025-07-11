//! Scene operation handlers
//!
//! This module provides the actual implementation of scene operations
//! that are triggered from the editor UI.

use engine::core::entity::World;
use engine::graphics::renderer::Renderer;
use engine::prelude::*;
use std::path::Path;
use tracing::info;

/// Create a default scene with camera and basic lighting
pub fn create_default_scene(world: &mut World, renderer: &mut Renderer) {
    info!("Creating default scene");

    // Clear the world first
    world.inner_mut().clear();

    // Create camera
    let _camera_entity = world.spawn((
        Name::new("Main Camera"),
        Camera::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0),
        Transform::from_position(Vec3::new(0.0, 5.0, 10.0)).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    // Create a cube
    let cube_mesh = Mesh::cube(1.0);
    let cube_mesh_id = renderer.upload_mesh(&cube_mesh, "cube");

    let _cube_entity = world.spawn((
        Name::new("Default Cube"),
        cube_mesh_id,
        Material::gray(0.8),
        Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    ));

    // Create a ground plane
    let plane_mesh = Mesh::plane(20.0, 20.0);
    let plane_mesh_id = renderer.upload_mesh(&plane_mesh, "plane");

    let _plane_entity = world.spawn((
        Name::new("Ground Plane"),
        plane_mesh_id,
        Material::gray(0.3),
        Transform::from_position(Vec3::new(0.0, -1.0, 0.0)),
        GlobalTransform::default(),
    ));

    info!(
        "Default scene created with {} entities",
        world.query::<()>().iter().count()
    );
}

/// Save the current world to a scene file
pub fn save_scene_to_file(world: &World, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Saving scene to: {:?}", path);
    world.save_scene(path)
}

/// Load a scene file into the world
pub fn load_scene_from_file(
    world: &mut World,
    _renderer: &mut Renderer,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading scene from: {:?}", path);
    world.load_scene(path)
}
