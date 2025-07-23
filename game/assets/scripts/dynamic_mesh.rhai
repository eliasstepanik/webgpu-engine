// Dynamic mesh generation example
// This script creates a procedurally generated mesh that changes over time

//#property float wave_amplitude = 0.5 "Amplitude of the wave effect"
//#property float wave_frequency = 2.0 "Frequency of the wave"
//#property int grid_size = 10 "Size of the grid (grid_size x grid_size vertices)"

// Store the mesh builder for updates
let mesh_builder = ();
let time_elapsed = 0.0;

fn on_start() {
    print("Creating dynamic mesh grid");
    
    // Create initial flat grid mesh
    create_wave_mesh(0.0);
}

fn on_update() {
    time_elapsed += delta_time;
    
    // Update the mesh every frame for animation
    if time_elapsed > 0.05 { // Update 20 times per second
        create_wave_mesh(time_elapsed);
        time_elapsed = 0.0;
    }
    
    // Slowly rotate the mesh
    let transform = world::get_transform(entity);
    transform.rotate_y(0.5 * delta_time);
    world::set_transform(entity, transform);
}

fn create_wave_mesh(time) {
    let builder = MeshBuilder::create();
    let grid_size = properties.grid_size;
    let cell_size = 0.2;
    let half_size = (grid_size * cell_size) / 2.0;
    
    // Create vertices
    let vertices = [];
    for y in 0..=grid_size {
        let row = [];
        for x in 0..=grid_size {
            let px = x * cell_size - half_size;
            let pz = y * cell_size - half_size;
            
            // Calculate wave height
            let distance = sqrt(px * px + pz * pz);
            let py = properties.wave_amplitude * sin(distance * properties.wave_frequency - time * 2.0);
            
            // Calculate normal (approximate)
            let dx = properties.wave_amplitude * properties.wave_frequency * cos(distance * properties.wave_frequency - time * 2.0) * (px / distance);
            let dz = properties.wave_amplitude * properties.wave_frequency * cos(distance * properties.wave_frequency - time * 2.0) * (pz / distance);
            let normal_x = -dx;
            let normal_z = -dz;
            let normal_y = 1.0;
            let len = sqrt(normal_x * normal_x + normal_y * normal_y + normal_z * normal_z);
            
            // Add vertex
            let v = builder.add_vertex(Mesh::vertex(
                px, py, pz,
                normal_x / len, normal_y / len, normal_z / len,
                x / grid_size, y / grid_size
            ));
            row.push(v);
        }
        vertices.push(row);
    }
    
    // Create triangles
    for y in 0..grid_size {
        for x in 0..grid_size {
            let v0 = vertices[y][x];
            let v1 = vertices[y][x + 1];
            let v2 = vertices[y + 1][x + 1];
            let v3 = vertices[y + 1][x];
            
            builder.add_quad(v0, v1, v2, v3);
        }
    }
    
    // Build and upload the mesh
    let mesh = builder.build();
    let mesh_id = Mesh::upload("wave_mesh_" + entity.to_string(), mesh);
    world::set_mesh_id(entity, mesh_id);
    
    // Set a blue-ish material
    let mat = Material::from_rgba(0.3, 0.5, 1.0, 1.0);
    world::set_material(entity, mat);
}