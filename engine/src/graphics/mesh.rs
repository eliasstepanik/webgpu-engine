//! Mesh component and vertex data structures
//!
//! Provides mesh data structures and primitive generation functions
//! for common 3D shapes like cubes, spheres, and planes.

use bytemuck::{Pod, Zeroable};

/// Vertex data structure for GPU rendering
///
/// This struct is tightly packed for efficient GPU transfer using bytemuck.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    /// Position in 3D space
    pub position: [f32; 3],
    /// Surface normal vector (normalized)
    pub normal: [f32; 3],
    /// Texture coordinates (UV mapping)
    pub uv: [f32; 2],
}

impl Vertex {
    /// Create a new vertex with the given attributes
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }

    /// Get the vertex attribute layout for wgpu
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Mesh component containing vertex and index data
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex data for the mesh
    pub vertices: Vec<Vertex>,
    /// Index data for triangle assembly
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Create a new mesh from vertices and indices
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Create a cube mesh with the given size
    ///
    /// The cube is centered at the origin with each side having length `size`.
    pub fn cube(size: f32) -> Self {
        let half = size * 0.5;

        // Define the 8 corner positions
        let positions = [
            [-half, -half, -half], // 0: left bottom back
            [half, -half, -half],  // 1: right bottom back
            [half, half, -half],   // 2: right top back
            [-half, half, -half],  // 3: left top back
            [-half, -half, half],  // 4: left bottom front
            [half, -half, half],   // 5: right bottom front
            [half, half, half],    // 6: right top front
            [-half, half, half],   // 7: left top front
        ];

        // Each face needs 4 unique vertices (for proper normals and UVs)
        let vertices = vec![
            // Front face (positive Z)
            Vertex::new(positions[4], [0.0, 0.0, 1.0], [0.0, 1.0]),
            Vertex::new(positions[5], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex::new(positions[6], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex::new(positions[7], [0.0, 0.0, 1.0], [0.0, 0.0]),
            // Back face (negative Z)
            Vertex::new(positions[1], [0.0, 0.0, -1.0], [0.0, 1.0]),
            Vertex::new(positions[0], [0.0, 0.0, -1.0], [1.0, 1.0]),
            Vertex::new(positions[3], [0.0, 0.0, -1.0], [1.0, 0.0]),
            Vertex::new(positions[2], [0.0, 0.0, -1.0], [0.0, 0.0]),
            // Top face (positive Y)
            Vertex::new(positions[7], [0.0, 1.0, 0.0], [0.0, 1.0]),
            Vertex::new(positions[6], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new(positions[2], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new(positions[3], [0.0, 1.0, 0.0], [0.0, 0.0]),
            // Bottom face (negative Y)
            Vertex::new(positions[0], [0.0, -1.0, 0.0], [0.0, 1.0]),
            Vertex::new(positions[1], [0.0, -1.0, 0.0], [1.0, 1.0]),
            Vertex::new(positions[5], [0.0, -1.0, 0.0], [1.0, 0.0]),
            Vertex::new(positions[4], [0.0, -1.0, 0.0], [0.0, 0.0]),
            // Right face (positive X)
            Vertex::new(positions[5], [1.0, 0.0, 0.0], [0.0, 1.0]),
            Vertex::new(positions[1], [1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new(positions[2], [1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new(positions[6], [1.0, 0.0, 0.0], [0.0, 0.0]),
            // Left face (negative X)
            Vertex::new(positions[0], [-1.0, 0.0, 0.0], [0.0, 1.0]),
            Vertex::new(positions[4], [-1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new(positions[7], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new(positions[3], [-1.0, 0.0, 0.0], [0.0, 0.0]),
        ];

        // Create indices for triangles (2 triangles per face, 6 indices per face)
        let mut indices = Vec::with_capacity(36);
        for i in 0..6 {
            let base = i * 4;
            // First triangle
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            // Second triangle
            indices.push(base);
            indices.push(base + 2);
            indices.push(base + 3);
        }

        Self { vertices, indices }
    }

    /// Create a plane mesh on the XZ plane
    ///
    /// The plane is centered at the origin with the given width and depth.
    pub fn plane(width: f32, depth: f32) -> Self {
        let half_width = width * 0.5;
        let half_depth = depth * 0.5;

        let vertices = vec![
            Vertex::new([-half_width, 0.0, -half_depth], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex::new([half_width, 0.0, -half_depth], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new([half_width, 0.0, half_depth], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-half_width, 0.0, half_depth], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ];

        // Create double-sided plane by including both winding orders
        let indices = vec![
            // Top face (viewed from above)
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
            // Bottom face (viewed from below)
            0, 2, 1, // First triangle reversed
            0, 3, 2, // Second triangle reversed
        ];

        Self { vertices, indices }
    }

    /// Create a UV sphere mesh
    ///
    /// # Arguments
    /// * `radius` - Radius of the sphere
    /// * `sectors` - Number of longitude divisions (minimum 3)
    /// * `stacks` - Number of latitude divisions (minimum 2)
    pub fn sphere(radius: f32, sectors: u32, stacks: u32) -> Self {
        let sectors = sectors.max(3);
        let stacks = stacks.max(2);

        let mut vertices = Vec::new();

        let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
        let stack_step = std::f32::consts::PI / stacks as f32;

        // Generate vertices
        for i in 0..=stacks {
            let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
            let xy = radius * stack_angle.cos();
            let z = radius * stack_angle.sin();

            for j in 0..=sectors {
                let sector_angle = j as f32 * sector_step;

                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                let position = [x, z, y];
                let normal = [x / radius, z / radius, y / radius];
                let uv = [j as f32 / sectors as f32, i as f32 / stacks as f32];

                vertices.push(Vertex::new(position, normal, uv));
            }
        }

        // Generate indices
        let mut indices = Vec::new();
        for i in 0..stacks {
            for j in 0..sectors {
                let first = i * (sectors + 1) + j;
                let second = first + sectors + 1;

                // First triangle
                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                // Second triangle
                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        Self { vertices, indices }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_size() {
        use std::mem;
        // Ensure vertex is tightly packed for GPU
        assert_eq!(mem::size_of::<Vertex>(), 32); // 8 floats * 4 bytes
    }

    #[test]
    fn test_mesh_cube_vertices() {
        let cube = Mesh::cube(1.0);
        assert_eq!(cube.vertices.len(), 24); // 6 faces * 4 vertices
        assert_eq!(cube.indices.len(), 36); // 6 faces * 2 triangles * 3 indices
    }

    #[test]
    fn test_mesh_plane() {
        let plane = Mesh::plane(10.0, 10.0);
        assert_eq!(plane.vertices.len(), 4); // 4 corners
        assert_eq!(plane.indices.len(), 12); // 2 faces * 2 triangles * 3 indices

        // Check that all vertices have Y=0 (on XZ plane)
        for vertex in &plane.vertices {
            assert_eq!(vertex.position[1], 0.0);
        }
    }

    #[test]
    fn test_mesh_sphere() {
        let sphere = Mesh::sphere(1.0, 16, 8);
        // Verify we have the expected number of vertices
        assert_eq!(sphere.vertices.len(), (16 + 1) * (8 + 1));
        // Verify we have the expected number of indices
        assert_eq!(sphere.indices.len(), 16 * 8 * 6);
    }

    #[test]
    fn test_cube_normals() {
        let cube = Mesh::cube(2.0);

        // Check that first 4 vertices (front face) have positive Z normal
        for i in 0..4 {
            assert_eq!(cube.vertices[i].normal, [0.0, 0.0, 1.0]);
        }

        // Check that next 4 vertices (back face) have negative Z normal
        for i in 4..8 {
            assert_eq!(cube.vertices[i].normal, [0.0, 0.0, -1.0]);
        }
    }
}
