//! Mesh file loading utilities
//!
//! Supports loading meshes from various file formats like OBJ and FBX.

use crate::graphics::mesh::{Mesh, Vertex};
use glam::Vec3;
use std::path::Path;
use tracing::{debug, error, info};

/// Errors that can occur during mesh loading
#[derive(Debug, thiserror::Error)]
pub enum MeshLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OBJ loading error: {0}")]
    ObjLoad(#[from] tobj::LoadError),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("No mesh data found in file")]
    NoMeshData,
}

/// Load a mesh from a file
pub fn load_mesh_from_file(path: &Path) -> Result<Mesh, MeshLoadError> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "obj" => load_obj(path),
        ext => Err(MeshLoadError::UnsupportedFormat(ext.to_string())),
    }
}

/// Load a mesh from an OBJ file
fn load_obj(path: &Path) -> Result<Mesh, MeshLoadError> {
    info!("Loading OBJ file: {:?}", path);

    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )?;

    if models.is_empty() {
        return Err(MeshLoadError::NoMeshData);
    }

    // For now, just load the first model
    let model = &models[0];
    let mesh = &model.mesh;

    debug!(
        "Loaded OBJ model '{}' with {} vertices and {} indices",
        model.name,
        mesh.positions.len() / 3,
        mesh.indices.len()
    );

    // Convert OBJ data to our vertex format
    let mut vertices = Vec::new();
    let num_vertices = mesh.positions.len() / 3;

    for i in 0..num_vertices {
        let pos_offset = i * 3;
        let position = [
            mesh.positions[pos_offset],
            mesh.positions[pos_offset + 1],
            mesh.positions[pos_offset + 2],
        ];

        // Check if we have texture coordinates
        let tex_coords = if !mesh.texcoords.is_empty() && i * 2 + 1 < mesh.texcoords.len() {
            let tex_offset = i * 2;
            [mesh.texcoords[tex_offset], mesh.texcoords[tex_offset + 1]]
        } else {
            [0.0, 0.0]
        };

        // Check if we have normals
        let normal = if !mesh.normals.is_empty() && pos_offset + 2 < mesh.normals.len() {
            [
                mesh.normals[pos_offset],
                mesh.normals[pos_offset + 1],
                mesh.normals[pos_offset + 2],
            ]
        } else {
            // Calculate a default normal if none provided
            [0.0, 1.0, 0.0]
        };

        vertices.push(Vertex {
            position,
            normal,
            uv: tex_coords,
        });
    }

    // If no normals were provided, calculate them
    if mesh.normals.is_empty() {
        calculate_normals(&mut vertices, &mesh.indices);
    }

    Ok(Mesh {
        vertices,
        indices: mesh.indices.clone(),
    })
}

/// Calculate normals for vertices based on face geometry
fn calculate_normals(vertices: &mut [Vertex], indices: &[u32]) {
    // First, zero out all normals
    for vertex in vertices.iter_mut() {
        vertex.normal = [0.0, 0.0, 0.0];
    }

    // Calculate face normals and add to vertex normals
    for chunk in indices.chunks(3) {
        if chunk.len() != 3 {
            continue;
        }

        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
            continue;
        }

        let v0 = Vec3::from(vertices[i0].position);
        let v1 = Vec3::from(vertices[i1].position);
        let v2 = Vec3::from(vertices[i2].position);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2).normalize();

        // Add face normal to each vertex
        for &i in &[i0, i1, i2] {
            let current = Vec3::from(vertices[i].normal);
            let new_normal = current + face_normal;
            vertices[i].normal = new_normal.to_array();
        }
    }

    // Normalize all vertex normals
    for vertex in vertices.iter_mut() {
        let normal = Vec3::from(vertex.normal).normalize();
        vertex.normal = normal.to_array();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_format() {
        let result = load_mesh_from_file(Path::new("test.fbx"));
        assert!(matches!(result, Err(MeshLoadError::UnsupportedFormat(_))));
    }
}
