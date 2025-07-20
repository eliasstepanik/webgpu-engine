// Simple test script for mesh generation

fn on_start() {
    print("Test mesh script started!");
    
    // Create a simple cube mesh
    let cube_mesh = Mesh::cube(2.0);
    let mesh_id = Mesh::upload("test_cube", cube_mesh);
    
    print("Mesh uploaded with ID: " + mesh_id);
    
    // Set the mesh on this entity
    world::set_mesh_id(entity, mesh_id);
    
    // Set a red material
    let mat = Material::red();
    world::set_material(entity, mat);
}

fn on_update() {
    // Simple rotation
    let transform = world::get_transform(entity);
    transform.rotate_y(1.0 * delta_time);
    world::set_transform(entity, transform);
}