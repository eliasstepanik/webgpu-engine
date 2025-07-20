//! Mesh generation API for scripts
//!
//! This module provides functions for creating and manipulating meshes from scripts.

use crate::graphics::mesh::{Mesh, Vertex};
use crate::scripting::mesh_registry::ScriptMeshRegistry;
use rhai::{Dynamic, Engine, EvalAltResult, Module};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::debug;

/// Global counter for generating unique callback IDs
static CALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Register mesh generation API with Rhai engine
pub fn register_mesh_api(engine: &mut Engine) {
    debug!("Registering mesh API");

    // Register Mesh type
    engine
        .register_type_with_name::<Mesh>("Mesh")
        .register_fn(
            "from_vertices",
            |vertices: Vec<Vertex>, indices: Vec<i64>| {
                let indices: Vec<u32> = indices.iter().map(|&i| i as u32).collect();
                Mesh::new(vertices, indices)
            },
        )
        .register_get("vertices", |mesh: &mut Mesh| mesh.vertices.clone())
        .register_get("indices", |mesh: &mut Mesh| {
            mesh.indices.iter().map(|&i| i as i64).collect::<Vec<i64>>()
        })
        .register_get("vertex_count", |mesh: &mut Mesh| mesh.vertices.len() as i64)
        .register_get("triangle_count", |mesh: &mut Mesh| {
            (mesh.indices.len() / 3) as i64
        });

    // Register Vertex type
    engine
        .register_type_with_name::<Vertex>("Vertex")
        .register_fn("create", |pos: Vec<f64>, normal: Vec<f64>, uv: Vec<f64>| {
            if pos.len() < 3 || normal.len() < 3 || uv.len() < 2 {
                Vertex::new([0.0; 3], [0.0, 1.0, 0.0], [0.0; 2])
            } else {
                Vertex::new(
                    [pos[0] as f32, pos[1] as f32, pos[2] as f32],
                    [normal[0] as f32, normal[1] as f32, normal[2] as f32],
                    [uv[0] as f32, uv[1] as f32],
                )
            }
        })
        .register_get("position", |v: &mut Vertex| {
            vec![
                Dynamic::from(v.position[0] as f64),
                Dynamic::from(v.position[1] as f64),
                Dynamic::from(v.position[2] as f64),
            ]
        })
        .register_get("normal", |v: &mut Vertex| {
            vec![
                Dynamic::from(v.normal[0] as f64),
                Dynamic::from(v.normal[1] as f64),
                Dynamic::from(v.normal[2] as f64),
            ]
        })
        .register_get("uv", |v: &mut Vertex| {
            vec![Dynamic::from(v.uv[0] as f64), Dynamic::from(v.uv[1] as f64)]
        });

    // Register mesh builder type
    engine
        .register_type_with_name::<MeshBuilder>("MeshBuilder")
        .register_fn("create", MeshBuilder::new)
        .register_fn("add_vertex", |builder: &mut MeshBuilder, vertex: Vertex| {
            builder.add_vertex(vertex);
        })
        .register_fn(
            "add_triangle",
            |builder: &mut MeshBuilder, a: i64, b: i64, c: i64| {
                builder.add_triangle(a as u32, b as u32, c as u32);
            },
        )
        .register_fn(
            "add_quad",
            |builder: &mut MeshBuilder, a: i64, b: i64, c: i64, d: i64| {
                builder.add_quad(a as u32, b as u32, c as u32, d as u32);
            },
        )
        .register_fn("build", |builder: &mut MeshBuilder| builder.build())
        .register_get("vertex_count", |builder: &mut MeshBuilder| {
            builder.vertices.len() as i64
        })
        .register_get("triangle_count", |builder: &mut MeshBuilder| {
            (builder.indices.len() / 3) as i64
        });

    debug!("Mesh API registered");
}

/// Create a mesh module with mesh generation functions
pub fn create_mesh_module(
    mesh_registry: ScriptMeshRegistry,
    command_queue: crate::scripting::commands::CommandQueue,
) -> Module {
    let mut module = Module::new();

    // Upload a mesh and get its ID
    let registry = mesh_registry.clone();
    let queue = command_queue.clone();
    module.set_native_fn(
        "upload",
        move |name: &str, mesh: Mesh| -> Result<String, Box<EvalAltResult>> {
            let callback_id = CALLBACK_COUNTER.fetch_add(1, Ordering::SeqCst);

            // Queue the mesh upload command
            queue
                .write()
                .unwrap()
                .push(crate::scripting::commands::ScriptCommand::UploadMesh {
                    name: name.to_string(),
                    mesh: mesh.clone(),
                    callback_id,
                });

            // Add to registry for tracking
            registry.add_pending_mesh(name.to_string(), mesh, callback_id);

            // For now, return a temporary mesh ID that will be replaced after upload
            Ok(format!("pending_mesh_{callback_id}"))
        },
    );

    // Get uploaded mesh ID by name
    let registry = mesh_registry.clone();
    module.set_native_fn(
        "get_mesh_id",
        move |callback_id: i64| -> Result<String, Box<EvalAltResult>> {
            if let Some(mesh_id) = registry.get_uploaded(callback_id as u64) {
                Ok(mesh_id.0)
            } else {
                Err(format!("Mesh with callback ID {callback_id} not yet uploaded").into())
            }
        },
    );

    // Cube generation
    module.set_native_fn("cube", |size: f64| -> Result<Mesh, Box<EvalAltResult>> {
        Ok(Mesh::cube(size as f32))
    });

    // Plane generation
    module.set_native_fn(
        "plane",
        |width: f64, depth: f64| -> Result<Mesh, Box<EvalAltResult>> {
            Ok(Mesh::plane(width as f32, depth as f32))
        },
    );

    // Sphere generation
    module.set_native_fn(
        "sphere",
        |radius: f64, sectors: i64, stacks: i64| -> Result<Mesh, Box<EvalAltResult>> {
            Ok(Mesh::sphere(radius as f32, sectors as u32, stacks as u32))
        },
    );

    module
}

/// Builder for creating custom meshes
#[derive(Clone, Debug)]
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    /// Create a new mesh builder
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Add a vertex and return its index
    pub fn add_vertex(&mut self, vertex: Vertex) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(vertex);
        index
    }

    /// Add a triangle using vertex indices
    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }

    /// Add a quad as two triangles
    pub fn add_quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        // First triangle
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
        // Second triangle
        self.indices.push(a);
        self.indices.push(c);
        self.indices.push(d);
    }

    /// Build the final mesh
    pub fn build(&self) -> Result<Mesh, String> {
        if self.vertices.is_empty() {
            return Err("No vertices in mesh".to_string());
        }
        if self.indices.is_empty() {
            return Err("No indices in mesh".to_string());
        }
        if self.indices.len() % 3 != 0 {
            return Err("Index count must be a multiple of 3".to_string());
        }

        // Validate indices
        let max_index = self.vertices.len() as u32 - 1;
        for &index in &self.indices {
            if index > max_index {
                return Err(format!("Index {index} out of bounds (max: {max_index})"));
            }
        }

        Ok(Mesh::new(self.vertices.clone(), self.indices.clone()))
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_builder() {
        let mut builder = MeshBuilder::new();

        // Add vertices for a triangle
        builder.add_vertex(Vertex::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]));
        builder.add_vertex(Vertex::new([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0]));
        builder.add_vertex(Vertex::new([0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0, 1.0]));

        // Add triangle
        builder.add_triangle(0, 1, 2);

        // Build mesh
        let mesh = builder.build().unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn test_mesh_builder_quad() {
        let mut builder = MeshBuilder::new();

        // Add vertices for a quad
        builder.add_vertex(Vertex::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]));
        builder.add_vertex(Vertex::new([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0]));
        builder.add_vertex(Vertex::new([1.0, 0.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0]));
        builder.add_vertex(Vertex::new([0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0, 1.0]));

        // Add quad
        builder.add_quad(0, 1, 2, 3);

        // Build mesh
        let mesh = builder.build().unwrap();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.indices.len(), 6); // 2 triangles
    }

    #[test]
    fn test_mesh_builder_validation() {
        let mut builder = MeshBuilder::new();

        // Empty builder should fail
        assert!(builder.build().is_err());

        // Add vertices but no indices
        builder.add_vertex(Vertex::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0]));
        assert!(builder.build().is_err());

        // Add invalid index
        builder.add_triangle(0, 1, 2); // Only have 1 vertex (index 0)
        assert!(builder.build().is_err());
    }
}
