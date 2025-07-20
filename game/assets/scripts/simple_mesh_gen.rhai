// Simple mesh generation example
// This script demonstrates basic mesh generation and entity creation

//#property float spawn_radius = 5.0 "Radius to spawn shapes around the entity"
//#property int shape_count = 6 "Number of shapes to spawn"

fn on_start() {
    print("Generating meshes...");
    
    // Generate different types of meshes
    let cube_mesh = Mesh::cube(1.0);
    let sphere_mesh = Mesh::sphere(0.5, 16, 8);
    let plane_mesh = Mesh::plane(2.0, 2.0);
    
    // Upload meshes to the renderer
    let cube_id = Mesh::upload("generated_cube", cube_mesh);
    let sphere_id = Mesh::upload("generated_sphere", sphere_mesh);
    let plane_id = Mesh::upload("generated_plane", plane_mesh);
    
    print("Uploaded meshes: " + cube_id + ", " + sphere_id + ", " + plane_id);
    
    // Create entities in a circle around this entity
    let center_pos = world::get_transform(entity).position;
    let angle_step = 2.0 * math::PI / properties.shape_count;
    
    for i in 0..properties.shape_count {
        let angle = i * angle_step;
        let x = center_pos.x + properties.spawn_radius * cos(angle);
        let z = center_pos.z + properties.spawn_radius * sin(angle);
        
        // Create transform for the new entity
        let transform = Transform::from_position(Vec3::create(x, center_pos.y, z));
        
        // Choose mesh, color, and name based on index
        let mesh_id = if i % 3 == 0 {
            cube_id
        } else if i % 3 == 1 {
            sphere_id
        } else {
            plane_id
        };
        
        let material = if i % 3 == 0 {
            Material::red()
        } else if i % 3 == 1 {
            Material::green()
        } else {
            Material::blue()
        };
        
        let entity_name = if i % 3 == 0 {
            "Generated Cube " + i
        } else if i % 3 == 1 {
            "Generated Sphere " + i
        } else {
            "Generated Plane " + i
        };
        
        // Create entity with mesh and name
        world::create_entity_with_mesh_and_name(transform, material, mesh_id, entity_name);
    }
    
    print("Created " + properties.shape_count + " entities with generated meshes");
}

fn on_update() {
    // Rotate this entity
    let transform = world::get_transform(entity);
    transform.rotate_y(0.5 * delta_time);
    world::set_transform(entity, transform);
}