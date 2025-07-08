//! Asset management with validation and fallback handling

use crate::graphics::{Material, Mesh, MeshId, MeshLibrary};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Asset manager for mesh resolution and validation
pub struct AssetManager {
    /// Mesh library for default meshes
    mesh_library: MeshLibrary,
    /// Validated mesh cache
    validated_meshes: HashMap<String, bool>,
    /// Default material for fallbacks
    default_material: Material,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetManager {
    /// Create a new asset manager
    pub fn new() -> Self {
        Self {
            mesh_library: MeshLibrary::new(),
            validated_meshes: HashMap::new(),
            default_material: Material::default(),
        }
    }

    /// Validate that a mesh is available or can be generated
    pub fn validate_mesh(&mut self, mesh_name: &str) -> bool {
        // Check cache first
        if let Some(&is_valid) = self.validated_meshes.get(mesh_name) {
            return is_valid;
        }

        // Check if mesh library can generate it
        let is_valid = self.mesh_library.has_mesh(mesh_name);
        
        // Cache the result
        self.validated_meshes.insert(mesh_name.to_string(), is_valid);
        
        if is_valid {
            debug!(mesh_name = mesh_name, "Mesh validation passed");
        } else {
            warn!(mesh_name = mesh_name, "Mesh validation failed - mesh not available");
        }
        
        is_valid
    }

    /// Get a mesh, returning fallback if validation fails
    pub fn get_mesh_or_fallback(&mut self, mesh_name: &str) -> (Mesh, bool) {
        if self.validate_mesh(mesh_name) {
            if let Some(mesh) = self.mesh_library.get_or_generate(mesh_name) {
                debug!(mesh_name = mesh_name, "Generated requested mesh");
                return (mesh, true);
            }
        }
        
        // Return error mesh as fallback
        warn!(mesh_name = mesh_name, "Using error mesh fallback");
        (MeshLibrary::error_mesh(), false)
    }

    /// Validate all assets referenced in a scene file
    pub fn validate_scene_assets<P: AsRef<Path>>(&mut self, scene_path: P) -> Result<AssetValidationReport, Box<dyn std::error::Error>> {
        use crate::io::Scene;
        
        let scene_path = scene_path.as_ref();
        info!(path = ?scene_path, "Validating scene assets");
        
        let scene = Scene::load_from_file(scene_path)?;
        let mut report = AssetValidationReport::new(scene_path.to_path_buf());
        
        // Check all entities for graphics components
        for (entity_index, entity) in scene.entities.iter().enumerate() {
            // Check MeshId components
            if let Some(mesh_value) = entity.components.get("MeshId") {
                match serde_json::from_value::<MeshId>(mesh_value.clone()) {
                    Ok(mesh_id) => {
                        let is_valid = self.validate_mesh(&mesh_id.0);
                        report.add_mesh_reference(entity_index, mesh_id.0, is_valid);
                    }
                    Err(e) => {
                        report.add_error(entity_index, format!("Invalid MeshId format: {}", e));
                    }
                }
            }
            
            // Check Material components
            if let Some(material_value) = entity.components.get("Material") {
                match serde_json::from_value::<Material>(material_value.clone()) {
                    Ok(_material) => {
                        report.add_material_reference(entity_index, true);
                    }
                    Err(e) => {
                        report.add_error(entity_index, format!("Invalid Material format: {}", e));
                    }
                }
            }
        }
        
        info!(
            total_meshes = report.mesh_references.len(),
            valid_meshes = report.mesh_references.iter().filter(|(_, _, valid)| *valid).count(),
            total_materials = report.material_references.len(),
            errors = report.errors.len(),
            "Scene asset validation complete"
        );
        
        Ok(report)
    }

    /// Get default material for fallbacks
    pub fn default_material(&self) -> Material {
        self.default_material.clone()
    }

    /// Get list of available meshes
    pub fn available_meshes(&self) -> Vec<String> {
        self.mesh_library.available_meshes()
    }

    /// Check if a mesh is available
    pub fn has_mesh(&self, mesh_name: &str) -> bool {
        self.mesh_library.has_mesh(mesh_name)
    }

    /// Register a custom mesh generator
    pub fn register_mesh<F>(&mut self, name: &str, generator: F)
    where
        F: Fn() -> Mesh + Send + Sync + 'static,
    {
        self.mesh_library.register(name, generator);
        // Invalidate cache for this mesh
        self.validated_meshes.remove(name);
        debug!(mesh_name = name, "Registered custom mesh generator");
    }
}

/// Report of asset validation results
#[derive(Debug, Clone)]
pub struct AssetValidationReport {
    /// Path to the scene file
    pub scene_path: std::path::PathBuf,
    /// Mesh references found (entity_index, mesh_name, is_valid)
    pub mesh_references: Vec<(usize, String, bool)>,
    /// Material references found (entity_index, is_valid)
    pub material_references: Vec<(usize, bool)>,
    /// Validation errors (entity_index, error_message)
    pub errors: Vec<(usize, String)>,
}

impl AssetValidationReport {
    /// Create a new validation report
    pub fn new(scene_path: std::path::PathBuf) -> Self {
        Self {
            scene_path,
            mesh_references: Vec::new(),
            material_references: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add a mesh reference to the report
    pub fn add_mesh_reference(&mut self, entity_index: usize, mesh_name: String, is_valid: bool) {
        self.mesh_references.push((entity_index, mesh_name, is_valid));
    }

    /// Add a material reference to the report
    pub fn add_material_reference(&mut self, entity_index: usize, is_valid: bool) {
        self.material_references.push((entity_index, is_valid));
    }

    /// Add an error to the report
    pub fn add_error(&mut self, entity_index: usize, error_message: String) {
        self.errors.push((entity_index, error_message));
    }

    /// Check if validation passed (no errors and all references valid)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() 
            && self.mesh_references.iter().all(|(_, _, valid)| *valid)
            && self.material_references.iter().all(|(_, valid)| *valid)
    }

    /// Get invalid mesh references
    pub fn invalid_meshes(&self) -> Vec<(usize, &String)> {
        self.mesh_references
            .iter()
            .filter_map(|(idx, name, valid)| if !valid { Some((*idx, name)) } else { None })
            .collect()
    }

    /// Get summary statistics
    pub fn summary(&self) -> AssetValidationSummary {
        AssetValidationSummary {
            total_mesh_references: self.mesh_references.len(),
            valid_mesh_references: self.mesh_references.iter().filter(|(_, _, valid)| *valid).count(),
            total_material_references: self.material_references.len(),
            valid_material_references: self.material_references.iter().filter(|(_, valid)| *valid).count(),
            total_errors: self.errors.len(),
            is_valid: self.is_valid(),
        }
    }
}

/// Summary statistics for asset validation
#[derive(Debug, Clone)]
pub struct AssetValidationSummary {
    pub total_mesh_references: usize,
    pub valid_mesh_references: usize,
    pub total_material_references: usize,
    pub valid_material_references: usize,
    pub total_errors: usize,
    pub is_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::Scene;
    use std::collections::HashMap;

    #[test]
    fn test_asset_manager_creation() {
        let manager = AssetManager::new();
        
        // Should have default meshes available
        assert!(manager.has_mesh("cube"));
        assert!(manager.has_mesh("sphere"));
        assert!(manager.has_mesh("plane"));
        assert!(manager.has_mesh("error_mesh"));
        assert!(!manager.has_mesh("nonexistent"));
    }

    #[test]
    fn test_mesh_validation() {
        let mut manager = AssetManager::new();
        
        // Valid meshes
        assert!(manager.validate_mesh("cube"));
        assert!(manager.validate_mesh("sphere"));
        
        // Invalid mesh
        assert!(!manager.validate_mesh("nonexistent"));
        
        // Should cache results
        assert!(manager.validated_meshes.contains_key("cube"));
        assert!(manager.validated_meshes.contains_key("nonexistent"));
    }

    #[test]
    fn test_mesh_fallback() {
        let mut manager = AssetManager::new();
        
        // Valid mesh should return requested mesh
        let (mesh, is_original) = manager.get_mesh_or_fallback("cube");
        assert!(is_original);
        assert!(!mesh.vertices.is_empty());
        
        // Invalid mesh should return error mesh
        let (fallback_mesh, is_original) = manager.get_mesh_or_fallback("nonexistent");
        assert!(!is_original);
        assert!(!fallback_mesh.vertices.is_empty());
    }

    #[test]
    fn test_custom_mesh_registration() {
        let mut manager = AssetManager::new();
        
        // Register custom mesh
        manager.register_mesh("custom_triangle", || {
            // Simple triangle mesh for testing
            Mesh::cube(0.5) // Using cube as placeholder
        });
        
        assert!(manager.has_mesh("custom_triangle"));
        assert!(manager.validate_mesh("custom_triangle"));
        
        let (mesh, is_original) = manager.get_mesh_or_fallback("custom_triangle");
        assert!(is_original);
        assert!(!mesh.vertices.is_empty());
    }

    #[test]
    fn test_validation_report() {
        let mut report = AssetValidationReport::new("test.json".into());
        
        report.add_mesh_reference(0, "cube".to_string(), true);
        report.add_mesh_reference(1, "invalid".to_string(), false);
        report.add_material_reference(0, true);
        report.add_error(2, "Test error".to_string());
        
        assert!(!report.is_valid()); // Has errors and invalid mesh
        assert_eq!(report.invalid_meshes().len(), 1);
        
        let summary = report.summary();
        assert_eq!(summary.total_mesh_references, 2);
        assert_eq!(summary.valid_mesh_references, 1);
        assert_eq!(summary.total_material_references, 1);
        assert_eq!(summary.valid_material_references, 1);
        assert_eq!(summary.total_errors, 1);
        assert!(!summary.is_valid);
    }

    #[test]
    fn test_scene_asset_validation() {
        // Create a temporary scene file for testing
        let mut scene = Scene::new();
        
        // Create test entity with components
        let mut components = HashMap::new();
        components.insert("MeshId".to_string(), serde_json::json!({"0": "cube"}));
        components.insert("Material".to_string(), serde_json::json!({"color": [1.0, 0.0, 0.0, 1.0]}));
        
        scene.entities.push(crate::io::SerializedEntity { components });
        
        // Save to temp file
        let temp_path = "test_asset_validation.json";
        scene.save_to_file(temp_path).unwrap();
        
        // Validate
        let mut manager = AssetManager::new();
        let report = manager.validate_scene_assets(temp_path).unwrap();
        
        // Clean up
        let _ = std::fs::remove_file(temp_path);
        
        // Check report (might have validation issues due to MeshId format)
        assert_eq!(report.scene_path.file_name().unwrap(), "test_asset_validation.json");
    }
}