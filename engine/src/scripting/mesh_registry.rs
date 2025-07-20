//! Registry for managing meshes created by scripts
//!
//! This module provides a thread-safe registry for storing meshes generated
//! by scripts and managing their upload to the renderer.

use crate::graphics::mesh::Mesh;
use crate::graphics::renderer::MeshId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

/// Thread-safe registry for script-generated meshes
#[derive(Clone, Default)]
pub struct ScriptMeshRegistry {
    /// Meshes waiting to be uploaded to the renderer
    pending_meshes: Arc<RwLock<Vec<PendingMesh>>>,
    /// Map from callback ID to uploaded mesh ID
    uploaded_meshes: Arc<RwLock<HashMap<u64, MeshId>>>,
}

#[derive(Clone, Debug)]
pub struct PendingMesh {
    pub name: String,
    pub mesh: Mesh,
    pub callback_id: u64,
}

impl ScriptMeshRegistry {
    /// Create a new mesh registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mesh to be uploaded
    pub fn add_pending_mesh(&self, name: String, mesh: Mesh, callback_id: u64) {
        let pending = PendingMesh {
            name,
            mesh,
            callback_id,
        };
        self.pending_meshes.write().unwrap().push(pending);
        debug!(callback_id, "Added pending mesh");
    }

    /// Get all pending meshes and clear the list
    pub fn take_pending_meshes(&self) -> Vec<PendingMesh> {
        let mut pending = self.pending_meshes.write().unwrap();
        std::mem::take(&mut *pending)
    }

    /// Register an uploaded mesh
    pub fn register_uploaded(&self, callback_id: u64, mesh_id: MeshId) {
        debug!(callback_id, mesh_id = ?mesh_id, "Registered uploaded mesh");
        self.uploaded_meshes
            .write()
            .unwrap()
            .insert(callback_id, mesh_id);
    }

    /// Get an uploaded mesh ID by callback ID
    pub fn get_uploaded(&self, callback_id: u64) -> Option<MeshId> {
        self.uploaded_meshes
            .read()
            .unwrap()
            .get(&callback_id)
            .cloned()
    }

    /// Clear all data
    pub fn clear(&self) {
        self.pending_meshes.write().unwrap().clear();
        self.uploaded_meshes.write().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_registry() {
        let registry = ScriptMeshRegistry::new();

        // Add a pending mesh
        let mesh = Mesh::cube(1.0);
        registry.add_pending_mesh("test_cube".to_string(), mesh.clone(), 42);

        // Take pending meshes
        let pending = registry.take_pending_meshes();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].callback_id, 42);
        assert_eq!(pending[0].name, "test_cube");

        // Verify list is now empty
        let pending = registry.take_pending_meshes();
        assert!(pending.is_empty());

        // Register uploaded mesh
        let mesh_id = MeshId("test_cube".to_string());
        registry.register_uploaded(42, mesh_id.clone());

        // Get uploaded mesh
        assert_eq!(registry.get_uploaded(42), Some(mesh_id));
        assert_eq!(registry.get_uploaded(99), None);
    }
}
