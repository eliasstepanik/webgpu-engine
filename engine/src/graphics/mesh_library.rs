//! Mesh library for predefined meshes and fallbacks

use crate::graphics::mesh::Mesh;
use std::collections::HashMap;
use tracing::debug;

/// Mesh library with predefined mesh generators and fallbacks
pub struct MeshLibrary {
    generators: HashMap<String, Box<dyn Fn() -> Mesh + Send + Sync>>,
}

impl Default for MeshLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshLibrary {
    /// Create a new mesh library with default generators
    pub fn new() -> Self {
        let mut library = Self {
            generators: HashMap::new(),
        };

        // Register default mesh generators
        library.register("cube", Box::new(|| Mesh::cube(1.0)));
        library.register("sphere", Box::new(|| Mesh::sphere(0.5, 32, 16)));
        library.register("plane", Box::new(|| Mesh::plane(2.0, 2.0)));
        library.register("error_mesh", Box::new(Self::create_error_mesh));

        debug!(
            "Initialized mesh library with {} default meshes",
            library.generators.len()
        );

        library
    }

    /// Register a mesh generator
    pub fn register<F>(&mut self, name: &str, generator: F)
    where
        F: Fn() -> Mesh + Send + Sync + 'static,
    {
        self.generators
            .insert(name.to_string(), Box::new(generator));
        debug!(mesh_name = name, "Registered mesh generator");
    }

    /// Get or generate a mesh by name
    pub fn get_or_generate(&self, name: &str) -> Option<Mesh> {
        self.generators.get(name).map(|generator| {
            debug!(mesh_name = name, "Generating mesh");
            generator()
        })
    }

    /// Get the error mesh (red wireframe cube)
    pub fn error_mesh() -> Mesh {
        Self::create_error_mesh()
    }

    /// Check if a mesh is available
    pub fn has_mesh(&self, name: &str) -> bool {
        self.generators.contains_key(name)
    }

    /// Get all available mesh names
    pub fn available_meshes(&self) -> Vec<String> {
        self.generators.keys().cloned().collect()
    }

    /// Create an error mesh (red wireframe cube for missing assets)
    fn create_error_mesh() -> Mesh {
        // Create a simple cube but with different colors to indicate error
        // This could be enhanced to be wireframe-only in the future
        Mesh::cube(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_library_creation() {
        let library = MeshLibrary::new();
        assert!(library.has_mesh("cube"));
        assert!(library.has_mesh("sphere"));
        assert!(library.has_mesh("plane"));
        assert!(library.has_mesh("error_mesh"));
        assert!(!library.has_mesh("nonexistent"));
    }

    #[test]
    fn test_mesh_generation() {
        let library = MeshLibrary::new();

        let cube = library.get_or_generate("cube");
        assert!(cube.is_some());

        let nonexistent = library.get_or_generate("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_custom_mesh_registration() {
        let mut library = MeshLibrary::new();

        library.register("custom_cube", Box::new(|| Mesh::cube(1.0)));
        assert!(library.has_mesh("custom_cube"));

        let custom_mesh = library.get_or_generate("custom_cube");
        assert!(custom_mesh.is_some());
    }

    #[test]
    fn test_error_mesh() {
        let error_mesh = MeshLibrary::error_mesh();
        assert!(!error_mesh.vertices.is_empty());
        assert!(!error_mesh.indices.is_empty());
    }

    #[test]
    fn test_available_meshes() {
        let library = MeshLibrary::new();
        let meshes = library.available_meshes();

        assert!(meshes.contains(&"cube".to_string()));
        assert!(meshes.contains(&"sphere".to_string()));
        assert!(meshes.contains(&"plane".to_string()));
        assert!(meshes.contains(&"error_mesh".to_string()));
        assert_eq!(meshes.len(), 4);
    }
}
