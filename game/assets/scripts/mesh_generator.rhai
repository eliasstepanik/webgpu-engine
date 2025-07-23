// Mesh generation example script
// This script demonstrates how to create custom meshes at runtime

// Script properties
//#property float size = 2.0 "Size of the generated mesh"
//#property color mesh_color = [1.0, 0.5, 0.0, 1.0] "Color of the mesh"
//#property int segments = 16 "Number of segments for circular shapes"

fn on_start() {
    // Create a custom pyramid mesh
    let builder = MeshBuilder::create();
    
    // Define vertices for a pyramid
    let half_size = properties.size / 2.0;
    
    // Base vertices (square)
    let v0 = builder.add_vertex(Mesh::vertex(-half_size, 0.0, -half_size, 0.0, -1.0, 0.0, 0.0, 0.0));
    let v1 = builder.add_vertex(Mesh::vertex(half_size, 0.0, -half_size, 0.0, -1.0, 0.0, 1.0, 0.0));
    let v2 = builder.add_vertex(Mesh::vertex(half_size, 0.0, half_size, 0.0, -1.0, 0.0, 1.0, 1.0));
    let v3 = builder.add_vertex(Mesh::vertex(-half_size, 0.0, half_size, 0.0, -1.0, 0.0, 0.0, 1.0));
    
    // Apex vertex
    let v4 = builder.add_vertex(Mesh::vertex(0.0, properties.size, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5));
    
    // Base quad (two triangles)
    builder.add_quad(v0, v3, v2, v1);
    
    // Side triangles
    builder.add_triangle(v0, v1, v4); // Front
    builder.add_triangle(v1, v2, v4); // Right
    builder.add_triangle(v2, v3, v4); // Back
    builder.add_triangle(v3, v0, v4); // Left
    
    // Build the mesh
    let pyramid_mesh = builder.build();
    
    // Upload the mesh and get its ID
    let mesh_name = "pyramid_" + entity.to_string();
    let mesh_id = Mesh::upload(mesh_name, pyramid_mesh);
    
    // Set the mesh on this entity
    world::set_mesh_id(entity, mesh_id);
    
    // Set the material color
    let mat = Material::from_rgba(
        properties.mesh_color[0],
        properties.mesh_color[1],
        properties.mesh_color[2],
        properties.mesh_color[3]
    );
    world::set_material(entity, mat);
    
    print("Created pyramid mesh with " + builder.vertex_count + " vertices and " + builder.triangle_count + " triangles");
}

fn on_update() {
    // Rotate the mesh
    let transform = world::get_transform(entity);
    transform.rotate_y(1.0 * delta_time);
    world::set_transform(entity, transform);
}

// Function to create a star mesh
fn create_star_mesh(points, inner_radius, outer_radius) {
    let builder = MeshBuilder::create();
    
    // Center vertex
    let center = builder.add_vertex(Mesh::vertex(0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5));
    
    // Create star vertices
    let angle_step = 2.0 * PI / points;
    let vertices = [];
    
    for i in 0..points {
        let angle = i * angle_step;
        
        // Outer vertex
        let x = outer_radius * cos(angle);
        let z = outer_radius * sin(angle);
        let outer_v = builder.add_vertex(Mesh::vertex(x, 0.0, z, 0.0, 1.0, 0.0, 0.5 + cos(angle) * 0.5, 0.5 + sin(angle) * 0.5));
        vertices.push(outer_v);
        
        // Inner vertex (between points)
        let inner_angle = angle + angle_step / 2.0;
        let ix = inner_radius * cos(inner_angle);
        let iz = inner_radius * sin(inner_angle);
        let inner_v = builder.add_vertex(Mesh::vertex(ix, 0.0, iz, 0.0, 1.0, 0.0, 0.5 + cos(inner_angle) * 0.3, 0.5 + sin(inner_angle) * 0.3));
        vertices.push(inner_v);
    }
    
    // Create triangles
    for i in 0..vertices.len() {
        let next = (i + 1) % vertices.len();
        builder.add_triangle(center, vertices[i], vertices[next]);
    }
    
    return builder.build();
}

// Function to create a cylinder mesh
fn create_cylinder_mesh(segments, radius, height) {
    let builder = MeshBuilder::create();
    
    let angle_step = 2.0 * PI / segments;
    let half_height = height / 2.0;
    
    // Create vertices for top and bottom circles
    let top_vertices = [];
    let bottom_vertices = [];
    
    for i in 0..segments {
        let angle = i * angle_step;
        let x = radius * cos(angle);
        let z = radius * sin(angle);
        let u = i / segments;
        
        // Top vertex
        let tv = builder.add_vertex(Mesh::vertex(x, half_height, z, x/radius, 0.0, z/radius, u, 0.0));
        top_vertices.push(tv);
        
        // Bottom vertex
        let bv = builder.add_vertex(Mesh::vertex(x, -half_height, z, x/radius, 0.0, z/radius, u, 1.0));
        bottom_vertices.push(bv);
    }
    
    // Create side quads
    for i in 0..segments {
        let next = (i + 1) % segments;
        builder.add_quad(
            bottom_vertices[i],
            bottom_vertices[next],
            top_vertices[next],
            top_vertices[i]
        );
    }
    
    // Create top cap
    let top_center = builder.add_vertex(Mesh::vertex(0.0, half_height, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5));
    for i in 0..segments {
        let next = (i + 1) % segments;
        builder.add_triangle(top_center, top_vertices[i], top_vertices[next]);
    }
    
    // Create bottom cap
    let bottom_center = builder.add_vertex(Mesh::vertex(0.0, -half_height, 0.0, 0.0, -1.0, 0.0, 0.5, 0.5));
    for i in 0..segments {
        let next = (i + 1) % segments;
        builder.add_triangle(bottom_center, bottom_vertices[next], bottom_vertices[i]);
    }
    
    return builder.build();
}