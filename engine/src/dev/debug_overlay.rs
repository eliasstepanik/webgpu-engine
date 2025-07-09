//! Debug overlay for scene statistics and error visualization

use crate::core::entity::World;
use crate::graphics::{AssetValidationReport, Material, MeshId};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Debug overlay for displaying scene information
pub struct DebugOverlay {
    /// Last time statistics were updated
    last_update: Instant,
    /// Update interval for statistics
    update_interval: Duration,
    /// Cached debug information
    cached_info: Option<SceneDebugInfo>,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugOverlay {
    /// Create a new debug overlay
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            update_interval: Duration::from_millis(500), // Update every 500ms
            cached_info: None,
        }
    }

    /// Update the debug overlay with current scene information
    pub fn update(&mut self, world: &World) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.cached_info = Some(SceneDebugInfo::collect(world));
            self.last_update = now;
            debug!("Updated debug overlay statistics");
        }
    }

    /// Get the current debug information
    pub fn debug_info(&self) -> Option<&SceneDebugInfo> {
        self.cached_info.as_ref()
    }

    /// Set the update interval for statistics collection
    pub fn set_update_interval(&mut self, interval: Duration) {
        self.update_interval = interval;
    }

    /// Force an immediate update of statistics
    pub fn force_update(&mut self, world: &World) {
        self.cached_info = Some(SceneDebugInfo::collect(world));
        self.last_update = Instant::now();
    }

    /// Print debug information to console
    pub fn print_debug_info(&self) {
        if let Some(info) = &self.cached_info {
            info!(
                total_entities = info.total_entities,
                renderable_entities = info.renderable_entities,
                cameras = info.cameras,
                transforms = info.transforms,
                global_transforms = info.global_transforms,
                parent_relationships = info.parent_relationships,
                unique_meshes = info.mesh_usage.len(),
                materials = info.materials,
                "Scene Debug Statistics"
            );

            // Print mesh usage
            for (mesh_name, count) in &info.mesh_usage {
                debug!(mesh = mesh_name, count = count, "Mesh usage");
            }

            // Print performance metrics
            if let Some(perf) = &info.performance_metrics {
                debug!(
                    entity_query_time_us = perf.entity_query_time.as_micros(),
                    component_query_time_us = perf.component_query_time.as_micros(),
                    "Performance Metrics"
                );
            }
        }
    }

    /// Get a formatted string of debug information
    pub fn format_debug_info(&self) -> String {
        if let Some(info) = &self.cached_info {
            format!(
                "Scene Debug Info:\n\
                 - Total Entities: {}\n\
                 - Renderable: {}\n\
                 - Cameras: {}\n\
                 - Transforms: {}\n\
                 - Unique Meshes: {}\n\
                 - Materials: {}\n\
                 - Parent Relationships: {}",
                info.total_entities,
                info.renderable_entities,
                info.cameras,
                info.transforms,
                info.mesh_usage.len(),
                info.materials,
                info.parent_relationships
            )
        } else {
            "Debug info not available".to_string()
        }
    }

    /// Check for potential issues and return warnings
    pub fn check_scene_health(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if let Some(info) = &self.cached_info {
            // Check for entities without transforms
            let entities_without_transforms = info.total_entities - info.transforms;
            if entities_without_transforms > 0 {
                warnings.push(format!(
                    "{entities_without_transforms} entities without Transform components"
                ));
            }

            // Check for transforms without global transforms
            let missing_global_transforms = info.transforms - info.global_transforms;
            if missing_global_transforms > 0 {
                warnings.push(format!(
                    "{missing_global_transforms} entities with Transform but no GlobalTransform"
                ));
            }

            // Check for no cameras
            if info.cameras == 0 {
                warnings.push("No camera entities found in scene".to_string());
            }

            // Check for multiple cameras (might be intentional but worth noting)
            if info.cameras > 1 {
                warnings.push(format!(
                    "{} camera entities found (multiple cameras)",
                    info.cameras
                ));
            }

            // Check for entities with meshes but no materials
            let mesh_entities = info.mesh_usage.values().sum::<usize>();
            if mesh_entities > info.materials {
                warnings.push(format!(
                    "{} entities with meshes but possibly missing materials",
                    mesh_entities - info.materials.min(mesh_entities)
                ));
            }

            // Check for performance issues
            if let Some(perf) = &info.performance_metrics {
                if perf.entity_query_time > Duration::from_millis(1) {
                    warnings.push(format!(
                        "Slow entity queries: {}ms",
                        perf.entity_query_time.as_millis()
                    ));
                }
            }
        }

        warnings
    }
}

/// Comprehensive debug information about a scene
#[derive(Debug, Clone)]
pub struct SceneDebugInfo {
    /// Total number of entities
    pub total_entities: usize,
    /// Number of renderable entities (with MeshId, Material, and Transform)
    pub renderable_entities: usize,
    /// Number of camera entities
    pub cameras: usize,
    /// Number of entities with Transform
    pub transforms: usize,
    /// Number of entities with GlobalTransform
    pub global_transforms: usize,
    /// Number of parent-child relationships
    pub parent_relationships: usize,
    /// Usage count for each mesh type
    pub mesh_usage: HashMap<String, usize>,
    /// Number of entities with materials
    pub materials: usize,
    /// Performance metrics for queries
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Performance metrics for debug overlay
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Time taken for entity queries
    pub entity_query_time: Duration,
    /// Time taken for component queries
    pub component_query_time: Duration,
}

impl SceneDebugInfo {
    /// Collect debug information from a world
    pub fn collect(world: &World) -> Self {
        let start_time = Instant::now();

        // Count total entities
        let total_entities = world.query::<()>().iter().count();
        let entity_query_time = start_time.elapsed();

        let component_start = Instant::now();

        // Count renderable entities
        let renderable_entities = world
            .query::<(
                &MeshId,
                &Material,
                &crate::core::entity::components::GlobalTransform,
            )>()
            .iter()
            .count();

        // Count cameras
        let cameras = world.query::<&crate::core::camera::Camera>().iter().count();

        // Count transforms
        let transforms = world
            .query::<&crate::core::entity::components::Transform>()
            .iter()
            .count();

        // Count global transforms
        let global_transforms = world
            .query::<&crate::core::entity::components::GlobalTransform>()
            .iter()
            .count();

        // Count parent relationships
        let parent_relationships = world
            .query::<&crate::core::entity::components::Parent>()
            .iter()
            .count();

        // Count materials
        let materials = world.query::<&Material>().iter().count();

        // Collect mesh usage statistics
        let mut mesh_usage = HashMap::new();
        for (_, mesh_id) in world.query::<&MeshId>().iter() {
            *mesh_usage.entry(mesh_id.0.clone()).or_insert(0) += 1;
        }

        let component_query_time = component_start.elapsed();

        let performance_metrics = Some(PerformanceMetrics {
            entity_query_time,
            component_query_time,
        });

        Self {
            total_entities,
            renderable_entities,
            cameras,
            transforms,
            global_transforms,
            parent_relationships,
            mesh_usage,
            materials,
            performance_metrics,
        }
    }

    /// Get a summary of the scene statistics
    pub fn summary(&self) -> String {
        format!(
            "Entities: {}, Renderable: {}, Cameras: {}, Meshes: {}",
            self.total_entities,
            self.renderable_entities,
            self.cameras,
            self.mesh_usage.len()
        )
    }

    /// Get detailed mesh usage information
    pub fn mesh_usage_summary(&self) -> String {
        if self.mesh_usage.is_empty() {
            "No meshes in use".to_string()
        } else {
            let mut parts = Vec::new();
            for (mesh_name, count) in &self.mesh_usage {
                parts.push(format!("{mesh_name}: {count}"));
            }
            parts.join(", ")
        }
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        if let Some(perf) = &self.performance_metrics {
            format!(
                "Query time: {}µs entities, {}µs components",
                perf.entity_query_time.as_micros(),
                perf.component_query_time.as_micros()
            )
        } else {
            "No performance data".to_string()
        }
    }
}

/// Asset validation debugging utilities
pub struct AssetValidationDebug;

impl AssetValidationDebug {
    /// Print detailed asset validation report
    pub fn print_validation_report(report: &AssetValidationReport) {
        let summary = report.summary();

        info!(
            scene_path = ?report.scene_path,
            total_meshes = summary.total_mesh_references,
            valid_meshes = summary.valid_mesh_references,
            total_materials = summary.total_material_references,
            valid_materials = summary.valid_material_references,
            errors = summary.total_errors,
            is_valid = summary.is_valid,
            "Asset Validation Report"
        );

        // Print invalid meshes
        for (entity_idx, mesh_name) in report.invalid_meshes() {
            tracing::warn!(
                entity_index = entity_idx,
                mesh_name = mesh_name,
                "Invalid mesh reference"
            );
        }

        // Print errors
        for (entity_idx, error) in &report.errors {
            tracing::error!(entity_index = entity_idx, error = error, "Validation error");
        }
    }

    /// Format validation report as string
    pub fn format_validation_report(report: &AssetValidationReport) -> String {
        let summary = report.summary();
        let mut output = format!(
            "Asset Validation Report for {:?}\n\
             Valid: {}\n\
             Meshes: {}/{}\n\
             Materials: {}/{}\n\
             Errors: {}\n",
            report.scene_path.file_name().unwrap_or_default(),
            summary.is_valid,
            summary.valid_mesh_references,
            summary.total_mesh_references,
            summary.valid_material_references,
            summary.total_material_references,
            summary.total_errors
        );

        if !report.invalid_meshes().is_empty() {
            output.push_str("\nInvalid Meshes:\n");
            for (entity_idx, mesh_name) in report.invalid_meshes() {
                output.push_str(&format!("  Entity {entity_idx}: {mesh_name}\n"));
            }
        }

        if !report.errors.is_empty() {
            output.push_str("\nErrors:\n");
            for (entity_idx, error) in &report.errors {
                output.push_str(&format!("  Entity {entity_idx}: {error}\n"));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::{components::Transform, World};

    #[test]
    fn test_debug_overlay_creation() {
        let overlay = DebugOverlay::new();
        assert!(overlay.debug_info().is_none());
    }

    #[test]
    fn test_scene_debug_info_collection() {
        let mut world = World::new();

        // Add some test entities
        world.spawn((Transform::default(),));
        world.spawn((Transform::default(),));

        let debug_info = SceneDebugInfo::collect(&world);
        assert_eq!(debug_info.total_entities, 2);
        assert!(debug_info.performance_metrics.is_some());
    }

    #[test]
    fn test_debug_overlay_update() {
        let mut overlay = DebugOverlay::new();
        let world = World::new();

        overlay.force_update(&world);
        assert!(overlay.debug_info().is_some());

        let info = overlay.debug_info().unwrap();
        assert_eq!(info.total_entities, 0);
    }

    #[test]
    fn test_scene_health_check() {
        let mut overlay = DebugOverlay::new();
        let world = World::new();

        overlay.force_update(&world);
        let warnings = overlay.check_scene_health();

        // Should warn about no cameras
        assert!(warnings.iter().any(|w| w.contains("No camera")));
    }

    #[test]
    fn test_debug_info_formatting() {
        let debug_info = SceneDebugInfo {
            total_entities: 5,
            renderable_entities: 3,
            cameras: 1,
            transforms: 4,
            global_transforms: 4,
            parent_relationships: 2,
            mesh_usage: [("cube".to_string(), 2), ("sphere".to_string(), 1)]
                .iter()
                .cloned()
                .collect(),
            materials: 3,
            performance_metrics: None,
        };

        let summary = debug_info.summary();
        assert!(summary.contains("Entities: 5"));
        assert!(summary.contains("Renderable: 3"));
        assert!(summary.contains("Cameras: 1"));

        let mesh_summary = debug_info.mesh_usage_summary();
        assert!(mesh_summary.contains("cube"));
        assert!(mesh_summary.contains("sphere"));
    }
}
