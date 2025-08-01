// Script to generate a grid of cubes for frustum culling demonstration
// This script runs once on start to populate the scene with many objects

let grid_size = 20;    // 20x20 grid
let spacing = 5.0;     // 5 units between cubes
let cube_scale = 0.5;  // Half-size cubes

fn start() {
    debug("Generating frustum culling demo grid...");
    
    // Get our own entity ID
    let self_entity = entity();
    
    // Create a grid of cubes
    for x in 0..grid_size {
        for z in 0..grid_size {
            // Calculate position
            let pos_x = (x - grid_size / 2) * spacing;
            let pos_z = (z - grid_size / 2) * spacing;
            let pos_y = 1.0; // Slightly above ground
            
            // Create cube entity
            let cube = world::spawn_entity();
            
            // Set name
            let name = "GridCube_" + x + "_" + z;
            world::set_name(cube, name);
            
            // Set transform
            let transform = Transform();
            transform.position = vec3(pos_x, pos_y, pos_z);
            transform.scale = vec3(cube_scale, cube_scale, cube_scale);
            world::set_transform(cube, transform);
            
            // Set mesh
            world::set_mesh(cube, "cube");
            
            // Create a color based on position
            let r = (x as float) / grid_size;
            let g = 0.5;
            let b = (z as float) / grid_size;
            
            // Set material with position-based color
            let material = Material();
            material.base_color = vec4(r, g, b, 1.0);
            material.metallic = 0.2;
            material.roughness = 0.8;
            world::set_material(cube, material);
        }
    }
    
    let total_cubes = grid_size * grid_size;
    debug("Generated " + total_cubes + " cubes for frustum culling demo");
    
    // Destroy the generator entity after creating the grid
    world::queue_destroy(self_entity);
}

fn update(delta_time) {
    // Nothing to update - this script only runs start()
}