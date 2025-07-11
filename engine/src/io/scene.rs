//! Scene serialization and loading

use crate::core::{
    camera::Camera,
    entity::{
        components::{GlobalTransform, Name, Parent, ParentData, Transform},
        World,
    },
};
use crate::graphics::{AssetManager, AssetValidationReport, Material, MeshId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use tracing::{debug, error, info, warn};

use super::entity_mapper::EntityMapper;

/// Scene data structure containing serialized entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// List of serialized entities with their components
    pub entities: Vec<SerializedEntity>,
}

/// A single serialized entity with its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    /// Map of component type names to their serialized JSON values
    pub components: HashMap<String, serde_json::Value>,
}

/// Errors that can occur during scene operations
#[derive(Debug)]
pub enum SceneError {
    /// IO error when reading/writing files
    Io(io::Error),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// Component deserialization error
    ComponentError(String),
    /// Entity not found during remapping
    EntityNotFound(u64),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneError::Io(e) => write!(f, "IO error: {e}"),
            SceneError::Json(e) => write!(f, "JSON error: {e}"),
            SceneError::ComponentError(msg) => write!(f, "Component error: {msg}"),
            SceneError::EntityNotFound(id) => write!(f, "Entity with ID {id} not found"),
        }
    }
}

impl std::error::Error for SceneError {}

impl From<io::Error> for SceneError {
    fn from(error: io::Error) -> Self {
        SceneError::Io(error)
    }
}

impl From<serde_json::Error> for SceneError {
    fn from(error: serde_json::Error) -> Self {
        SceneError::Json(error)
    }
}

impl Scene {
    /// Create a new empty scene
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Create a scene from a world, capturing all entities and their components
    pub fn from_world(world: &World) -> Self {
        let mut entities = Vec::new();
        let mut entity_to_id = HashMap::new();

        // First pass: assign IDs to all entities
        for (id, (entity, _)) in world.query::<()>().iter().enumerate() {
            entity_to_id.insert(entity, id as u64);
        }

        debug!(
            entity_count = entity_to_id.len(),
            "Assigned IDs to entities"
        );

        // Second pass: serialize components
        for (entity, ()) in world.query::<()>().iter() {
            let mut components = HashMap::new();

            // Serialize Transform component
            if let Ok(transform) = world.get::<Transform>(entity) {
                match serde_json::to_value(*transform) {
                    Ok(value) => {
                        components.insert("Transform".to_string(), value);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to serialize Transform");
                    }
                }
            }

            // Serialize GlobalTransform component
            if let Ok(global_transform) = world.get::<GlobalTransform>(entity) {
                match serde_json::to_value(*global_transform) {
                    Ok(value) => {
                        components.insert("GlobalTransform".to_string(), value);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to serialize GlobalTransform");
                    }
                }
            }

            // Serialize Camera component
            if let Ok(camera) = world.get::<Camera>(entity) {
                match serde_json::to_value(*camera) {
                    Ok(value) => {
                        components.insert("Camera".to_string(), value);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to serialize Camera");
                    }
                }
            }

            // Special handling for Parent component
            if let Ok(parent) = world.get::<Parent>(entity) {
                // Convert Parent to ParentData with remapped ID
                if let Some(&parent_id) = entity_to_id.get(&parent.0) {
                    let parent_data = ParentData::from_parent_with_id(*parent, parent_id);
                    match serde_json::to_value(parent_data) {
                        Ok(value) => {
                            components.insert("Parent".to_string(), value);
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to serialize Parent");
                        }
                    }
                } else {
                    warn!(
                        entity = ?entity,
                        parent = ?parent.0,
                        "Parent entity not found in scene"
                    );
                }
            }

            // Serialize MeshId component
            if let Ok(mesh_id) = world.get::<MeshId>(entity) {
                match serde_json::to_value((*mesh_id).clone()) {
                    Ok(value) => {
                        components.insert("MeshId".to_string(), value);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to serialize MeshId");
                    }
                }
            }

            // Serialize Material component
            if let Ok(material) = world.get::<Material>(entity) {
                match serde_json::to_value(*material) {
                    Ok(value) => {
                        components.insert("Material".to_string(), value);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to serialize Material");
                    }
                }
            }

            // Serialize Name component
            if let Ok(name) = world.get::<Name>(entity) {
                match serde_json::to_value(&*name) {
                    Ok(value) => {
                        components.insert("Name".to_string(), value);
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to serialize Name");
                    }
                }
            }

            entities.push(SerializedEntity { components });
        }

        info!(entity_count = entities.len(), "Created scene from world");

        Scene { entities }
    }

    /// Instantiate this scene into a world, returning an entity mapper for ID lookups
    pub fn instantiate(&self, world: &mut World) -> Result<EntityMapper, SceneError> {
        let mut mapper = EntityMapper::new();
        let mut entities_to_build = Vec::new();

        info!(entity_count = self.entities.len(), "Instantiating scene");

        // First pass: spawn all entities and build ID mapping
        for (id, serialized_entity) in self.entities.iter().enumerate() {
            let entity = world.spawn(());
            mapper.register(id as u64, entity);
            entities_to_build.push((entity, serialized_entity));
            debug!(id = id, entity = ?entity, "Spawned entity");
        }

        // Second pass: add components with remapping
        for (entity, serialized_entity) in entities_to_build {
            for (component_type, value) in &serialized_entity.components {
                match component_type.as_str() {
                    "Transform" => match serde_json::from_value::<Transform>(value.clone()) {
                        Ok(transform) => {
                            if let Err(e) = world.insert_one(entity, transform) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Transform");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Transform");
                        }
                    },
                    "GlobalTransform" => {
                        match serde_json::from_value::<GlobalTransform>(value.clone()) {
                            Ok(global_transform) => {
                                if let Err(e) = world.insert_one(entity, global_transform) {
                                    error!(
                                        error = ?e,
                                        entity = ?entity,
                                        "Failed to insert GlobalTransform"
                                    );
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to deserialize GlobalTransform");
                            }
                        }
                    }
                    "Camera" => match serde_json::from_value::<Camera>(value.clone()) {
                        Ok(camera) => {
                            if let Err(e) = world.insert_one(entity, camera) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Camera");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Camera");
                        }
                    },
                    "Parent" => match serde_json::from_value::<ParentData>(value.clone()) {
                        Ok(parent_data) => {
                            if let Some(parent_component) =
                                parent_data.try_to_parent(|id| mapper.remap(id))
                            {
                                if let Err(e) = world.insert_one(entity, parent_component) {
                                    error!(
                                        error = ?e,
                                        entity = ?entity,
                                        "Failed to insert Parent"
                                    );
                                }
                            } else {
                                warn!(
                                    parent_id = parent_data.entity_id,
                                    "Parent entity not found in scene during instantiation"
                                );
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Parent");
                        }
                    },
                    "MeshId" => match serde_json::from_value::<MeshId>(value.clone()) {
                        Ok(mesh_id) => {
                            if let Err(e) = world.insert_one(entity, mesh_id) {
                                error!(error = ?e, entity = ?entity, "Failed to insert MeshId");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize MeshId");
                        }
                    },
                    "Material" => match serde_json::from_value::<Material>(value.clone()) {
                        Ok(material) => {
                            if let Err(e) = world.insert_one(entity, material) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Material");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Material");
                        }
                    },
                    "Name" => match serde_json::from_value::<Name>(value.clone()) {
                        Ok(name) => {
                            if let Err(e) = world.insert_one(entity, name) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Name");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Name");
                        }
                    },
                    unknown => {
                        warn!(
                            component_type = unknown,
                            "Unknown component type in scene, skipping"
                        );
                    }
                }
            }
        }

        info!("Scene instantiation complete");
        Ok(mapper)
    }

    /// Save this scene to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SceneError> {
        let path = path.as_ref();
        info!(path = ?path, "Saving scene to file");

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;

        info!(path = ?path, "Scene saved successfully");
        Ok(())
    }

    /// Load a scene from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, SceneError> {
        let path = path.as_ref();
        info!(path = ?path, "Loading scene from file");

        let json = fs::read_to_string(path)?;
        let scene = serde_json::from_str(&json)?;

        info!(path = ?path, "Scene loaded successfully");
        Ok(scene)
    }

    /// Load a scene from a JSON file with asset validation
    ///
    /// This method loads the scene and validates all referenced assets,
    /// providing detailed error reporting for missing or invalid assets.
    pub fn load_from_file_with_validation<P: AsRef<std::path::Path>>(
        path: P,
        asset_manager: &mut AssetManager,
    ) -> Result<(Self, AssetValidationReport), SceneError> {
        let path = path.as_ref();
        info!(path = ?path, "Loading scene from file with validation");

        // Load the scene first
        let scene = Self::load_from_file(path)?;

        // Validate assets
        let validation_report = asset_manager
            .validate_scene_assets(path)
            .map_err(|e| SceneError::ComponentError(format!("Asset validation failed: {e}")))?;

        // Log validation results
        let summary = validation_report.summary();
        if summary.is_valid {
            info!(
                total_meshes = summary.total_mesh_references,
                total_materials = summary.total_material_references,
                "Scene asset validation passed"
            );
        } else {
            warn!(
                valid_meshes = summary.valid_mesh_references,
                total_meshes = summary.total_mesh_references,
                valid_materials = summary.valid_material_references,
                total_materials = summary.total_material_references,
                errors = summary.total_errors,
                "Scene asset validation found issues"
            );

            // Log specific invalid meshes
            for (entity_idx, mesh_name) in validation_report.invalid_meshes() {
                warn!(
                    entity_index = entity_idx,
                    mesh_name = mesh_name,
                    "Invalid mesh reference will use fallback"
                );
            }

            // Log errors
            for (entity_idx, error) in &validation_report.errors {
                error!(
                    entity_index = entity_idx,
                    error = error,
                    "Asset validation error"
                );
            }
        }

        Ok((scene, validation_report))
    }

    /// Instantiate this scene into a world with asset validation and fallbacks
    ///
    /// This method provides better error handling by using the asset manager
    /// to validate assets and provide fallbacks for missing resources.
    pub fn instantiate_with_validation(
        &self,
        world: &mut World,
        asset_manager: &mut AssetManager,
    ) -> Result<EntityMapper, SceneError> {
        let mut mapper = EntityMapper::new();
        let mut entities_to_build = Vec::new();

        info!(
            entity_count = self.entities.len(),
            "Instantiating scene with validation"
        );

        // First pass: spawn all entities and build ID mapping
        for (id, serialized_entity) in self.entities.iter().enumerate() {
            let entity = world.spawn(());
            mapper.register(id as u64, entity);
            entities_to_build.push((entity, serialized_entity));
            debug!(id = id, entity = ?entity, "Spawned entity");
        }

        // Second pass: add components with validation and fallbacks
        for (entity, serialized_entity) in entities_to_build {
            for (component_type, value) in &serialized_entity.components {
                match component_type.as_str() {
                    "Transform" => match serde_json::from_value::<Transform>(value.clone()) {
                        Ok(transform) => {
                            if let Err(e) = world.insert_one(entity, transform) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Transform");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Transform");
                        }
                    },
                    "GlobalTransform" => {
                        match serde_json::from_value::<GlobalTransform>(value.clone()) {
                            Ok(global_transform) => {
                                if let Err(e) = world.insert_one(entity, global_transform) {
                                    error!(
                                        error = ?e,
                                        entity = ?entity,
                                        "Failed to insert GlobalTransform"
                                    );
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to deserialize GlobalTransform");
                            }
                        }
                    }
                    "Camera" => match serde_json::from_value::<Camera>(value.clone()) {
                        Ok(camera) => {
                            if let Err(e) = world.insert_one(entity, camera) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Camera");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Camera");
                        }
                    },
                    "Parent" => match serde_json::from_value::<ParentData>(value.clone()) {
                        Ok(parent_data) => {
                            if let Some(parent_component) =
                                parent_data.try_to_parent(|id| mapper.remap(id))
                            {
                                if let Err(e) = world.insert_one(entity, parent_component) {
                                    error!(
                                        error = ?e,
                                        entity = ?entity,
                                        "Failed to insert Parent"
                                    );
                                }
                            } else {
                                warn!(
                                    parent_id = parent_data.entity_id,
                                    "Parent entity not found in scene during instantiation"
                                );
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Parent");
                        }
                    },
                    "MeshId" => match serde_json::from_value::<MeshId>(value.clone()) {
                        Ok(mesh_id) => {
                            // Validate mesh and use fallback if needed
                            let final_mesh_id = if asset_manager.validate_mesh(&mesh_id.0) {
                                debug!(entity = ?entity, mesh_id = %mesh_id.0, "Using validated mesh");
                                mesh_id
                            } else {
                                warn!(
                                    entity = ?entity,
                                    requested_mesh = %mesh_id.0,
                                    "Mesh not found, using error_mesh fallback"
                                );
                                MeshId("error_mesh".to_string())
                            };

                            if let Err(e) = world.insert_one(entity, final_mesh_id) {
                                error!(error = ?e, entity = ?entity, "Failed to insert MeshId");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize MeshId, using default");
                            // Insert fallback mesh on deserialization error
                            let fallback_mesh = MeshId("error_mesh".to_string());
                            if let Err(e) = world.insert_one(entity, fallback_mesh) {
                                error!(error = ?e, entity = ?entity, "Failed to insert fallback MeshId");
                            }
                        }
                    },
                    "Material" => match serde_json::from_value::<Material>(value.clone()) {
                        Ok(material) => {
                            if let Err(e) = world.insert_one(entity, material) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Material");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Material, using default");
                            // Insert fallback material on deserialization error
                            let fallback_material = asset_manager.default_material();
                            if let Err(e) = world.insert_one(entity, fallback_material) {
                                error!(error = ?e, entity = ?entity, "Failed to insert fallback Material");
                            }
                        }
                    },
                    "Name" => match serde_json::from_value::<Name>(value.clone()) {
                        Ok(name) => {
                            if let Err(e) = world.insert_one(entity, name) {
                                error!(error = ?e, entity = ?entity, "Failed to insert Name");
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to deserialize Name");
                        }
                    },
                    unknown => {
                        warn!(
                            component_type = unknown,
                            "Unknown component type in scene, skipping"
                        );
                    }
                }
            }
        }

        info!("Scene instantiation with validation complete");
        Ok(mapper)
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
