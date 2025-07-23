//! Physics scene validation utilities
//!
//! This module provides tools to validate physics scene configurations,
//! detecting common issues like floating objects, collision gaps, and invalid scales.

use crate::core::entity::Transform;
use crate::io::Scene;
use crate::physics::collision::AABB;
use crate::physics::components::{Collider, CollisionShape, Rigidbody};
use glam::Vec3;
use tracing::debug;

/// Result of validating a physics scene
#[derive(Debug, Default)]
pub struct SceneValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

/// Validation error information
#[derive(Debug)]
pub struct ValidationError {
    pub entity_name: String,
    pub error_type: ErrorType,
    pub details: String,
}

/// Types of validation errors
#[derive(Debug)]
pub enum ErrorType {
    NoOverlap { gap_distance: f32 },
    MissingCollider,
    InvalidScale { scale: Vec3 },
    FloatingObject { height: f32 },
    InitialPenetration,
}

/// Validation warning information
#[derive(Debug)]
pub struct ValidationWarning {
    pub entity: String,
    pub warning: String,
}

/// Information about a collider in the scene
#[derive(Debug)]
struct ColliderInfo {
    name: String,
    position: Vec3,
    scale: Vec3,
    half_extents: Vec3,
    is_static: bool,
}

/// Information about a rigidbody in the scene
#[derive(Debug)]
struct RigidbodyInfo {
    name: String,
    position: Vec3,
    scale: Vec3,
    scaled_half_extents: Vec3,
    has_gravity: bool,
}

/// Validate a physics scene for common configuration issues
pub fn validate_physics_scene(scene: &Scene) -> SceneValidationResult {
    let mut result = SceneValidationResult::default();
    
    // Collect all colliders and rigidbodies
    let (static_colliders, dynamic_bodies) = analyze_scene(scene);
    
    debug!(
        "Found {} static colliders and {} dynamic bodies",
        static_colliders.len(),
        dynamic_bodies.len()
    );
    
    // Check for floating objects
    check_floating_objects(&dynamic_bodies, &static_colliders, &mut result);
    
    // Check floor configuration
    check_floor_configuration(&static_colliders, &mut result);
    
    // Check for initial overlaps
    check_initial_overlaps(&dynamic_bodies, &static_colliders, &mut result);
    
    // Add suggestions based on findings
    add_suggestions(&mut result);
    
    // Determine overall validity
    result.is_valid = result.errors.is_empty();
    
    result
}

/// Analyze the scene and extract collider and rigidbody information
fn analyze_scene(scene: &Scene) -> (Vec<ColliderInfo>, Vec<RigidbodyInfo>) {
    let mut static_colliders = Vec::new();
    let mut dynamic_bodies = Vec::new();
    
    for entity_data in &scene.entities {
        let name = entity_data
            .components
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed")
            .to_string();
        
        // Get transform data
        let transform = if let Some(transform_value) = entity_data.components.get("Transform") {
            if let Ok(transform) = serde_json::from_value::<Transform>(transform_value.clone()) {
                transform
            } else {
                continue;
            }
        } else {
            continue;
        };
        
        // Check for collider
        if let Some(collider_value) = entity_data.components.get("Collider") {
            if let Ok(collider) = serde_json::from_value::<Collider>(collider_value.clone()) {
                // Extract half extents based on shape
                let half_extents = match &collider.shape {
                    CollisionShape::Box { half_extents } => *half_extents,
                    CollisionShape::Sphere { radius } => Vec3::splat(*radius),
                    CollisionShape::Capsule {
                        radius,
                        half_height,
                    } => Vec3::new(*radius, half_height + radius, *radius),
                };
                
                // Check if this is a dynamic body
                if let Some(rigidbody_value) = entity_data.components.get("Rigidbody") {
                    if let Ok(rigidbody) = serde_json::from_value::<Rigidbody>(rigidbody_value.clone()) {
                        dynamic_bodies.push(RigidbodyInfo {
                            name: name.clone(),
                            position: transform.position,
                            scale: transform.scale,
                            scaled_half_extents: half_extents * transform.scale,
                            has_gravity: rigidbody.use_gravity,
                        });
                    }
                } else {
                    // Static collider
                    static_colliders.push(ColliderInfo {
                        name,
                        position: transform.position,
                        scale: transform.scale,
                        half_extents,
                        is_static: true,
                    });
                }
            }
        }
    }
    
    (static_colliders, dynamic_bodies)
}

/// Check for floating objects that won't reach any floor
fn check_floating_objects(
    dynamic_bodies: &[RigidbodyInfo],
    static_colliders: &[ColliderInfo],
    result: &mut SceneValidationResult,
) {
    for body in dynamic_bodies {
        if !body.has_gravity {
            continue; // Skip non-gravity objects
        }
        
        let body_bottom = body.position.y - body.scaled_half_extents.y;
        let (nearest_floor, distance) = find_nearest_floor_below(body, static_colliders);
        
        if let Some(floor) = nearest_floor {
            if distance > 2.0 {
                result.warnings.push(ValidationWarning {
                    entity: body.name.clone(),
                    warning: format!(
                        "Object starts {:.1}m above nearest floor '{}'. Consider reducing gap.",
                        distance, floor.name
                    ),
                });
            }
            
            // Check if gap is too large (more than 10 units)
            if distance > 10.0 {
                result.errors.push(ValidationError {
                    entity_name: body.name.clone(),
                    error_type: ErrorType::NoOverlap { gap_distance: distance },
                    details: format!(
                        "Object is {:.1}m above floor '{}' - collision unlikely",
                        distance, floor.name
                    ),
                });
            }
        } else if body_bottom > 0.0 {
            result.errors.push(ValidationError {
                entity_name: body.name.clone(),
                error_type: ErrorType::FloatingObject { height: body_bottom },
                details: format!("No floor found below object at height {:.1}", body_bottom),
            });
        }
    }
}

/// Find the nearest floor below a dynamic body
fn find_nearest_floor_below<'a>(
    body: &RigidbodyInfo,
    static_colliders: &'a [ColliderInfo],
) -> (Option<&'a ColliderInfo>, f32) {
    let mut nearest_floor = None;
    let mut min_distance = f32::MAX;
    
    for collider in static_colliders {
        // Check if collider is below the body (X-Z overlap)
        let x_overlap = (body.position.x - collider.position.x).abs()
            <= body.scaled_half_extents.x + collider.half_extents.x * collider.scale.x;
        let z_overlap = (body.position.z - collider.position.z).abs()
            <= body.scaled_half_extents.z + collider.half_extents.z * collider.scale.z;
        
        if x_overlap && z_overlap {
            let floor_top = collider.position.y + collider.half_extents.y * collider.scale.y;
            let body_bottom = body.position.y - body.scaled_half_extents.y;
            
            if floor_top < body_bottom {
                let distance = body_bottom - floor_top;
                if distance < min_distance {
                    min_distance = distance;
                    nearest_floor = Some(collider);
                }
            }
        }
    }
    
    (nearest_floor, min_distance)
}

/// Check floor configuration for potential issues
fn check_floor_configuration(
    static_colliders: &[ColliderInfo],
    result: &mut SceneValidationResult,
) {
    for collider in static_colliders {
        // Check for very thin floors
        let actual_height = collider.half_extents.y * collider.scale.y * 2.0;
        if actual_height < 0.1 {
            result.warnings.push(ValidationWarning {
                entity: collider.name.clone(),
                warning: format!(
                    "Floor is only {:.3}m thick - may cause tunneling at high velocities",
                    actual_height
                ),
            });
        }
        
        // Check for extreme scales
        if collider.scale.x > 100.0 || collider.scale.y > 100.0 || collider.scale.z > 100.0 {
            result.warnings.push(ValidationWarning {
                entity: collider.name.clone(),
                warning: format!(
                    "Very large scale {:?} - consider using larger base geometry",
                    collider.scale
                ),
            });
        }
        
        // Warn about non-uniform scaling for certain shapes
        if (collider.scale.x - collider.scale.y).abs() > 0.01
            || (collider.scale.x - collider.scale.z).abs() > 0.01
        {
            result.warnings.push(ValidationWarning {
                entity: collider.name.clone(),
                warning: "Non-uniform scaling may cause unexpected collision behavior".to_string(),
            });
        }
    }
}

/// Check for initial overlaps between bodies
fn check_initial_overlaps(
    dynamic_bodies: &[RigidbodyInfo],
    static_colliders: &[ColliderInfo],
    result: &mut SceneValidationResult,
) {
    // Check dynamic vs static overlaps
    for body in dynamic_bodies {
        let body_aabb = compute_aabb(
            body.position,
            body.scaled_half_extents,
        );
        
        for collider in static_colliders {
            let collider_aabb = compute_aabb(
                collider.position,
                collider.half_extents * collider.scale,
            );
            
            if body_aabb.overlaps(&collider_aabb) {
                result.errors.push(ValidationError {
                    entity_name: body.name.clone(),
                    error_type: ErrorType::InitialPenetration,
                    details: format!(
                        "Overlaps with static collider '{}' at start - will cause ejection",
                        collider.name
                    ),
                });
            }
        }
    }
    
    // Check dynamic vs dynamic overlaps
    for (i, body1) in dynamic_bodies.iter().enumerate() {
        let body1_aabb = compute_aabb(
            body1.position,
            body1.scaled_half_extents,
        );
        
        for body2 in dynamic_bodies.iter().skip(i + 1) {
            let body2_aabb = compute_aabb(
                body2.position,
                body2.scaled_half_extents,
            );
            
            if body1_aabb.overlaps(&body2_aabb) {
                result.warnings.push(ValidationWarning {
                    entity: body1.name.clone(),
                    warning: format!(
                        "Initially overlaps with '{}' - will separate on simulation start",
                        body2.name
                    ),
                });
            }
        }
    }
}

/// Compute AABB from position and half extents
fn compute_aabb(position: Vec3, half_extents: Vec3) -> AABB {
    AABB::from_center_half_extents(position, half_extents)
}

/// Add helpful suggestions based on validation results
fn add_suggestions(result: &mut SceneValidationResult) {
    // Suggest floor positioning
    if result.errors.iter().any(|e| matches!(e.error_type, ErrorType::NoOverlap { .. })) {
        result.suggestions.push(
            "Consider positioning floors at Y=0 or Y=-0.5 for easier object placement".to_string()
        );
        result.suggestions.push(
            "Reduce vertical gaps between floors and falling objects to < 2 units".to_string()
        );
    }
    
    // Suggest scale improvements
    if result.warnings.iter().any(|w| w.warning.contains("thick")) {
        result.suggestions.push(
            "Use scale Y=1.0 for floors and adjust half_extents instead".to_string()
        );
    }
    
    // Suggest testing approach
    if !result.errors.is_empty() {
        result.suggestions.push(
            "Test with SCENE=your_scene cargo run to verify physics behavior".to_string()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::SerializedEntity;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_scene_with_gap() -> Scene {
        Scene {
            entities: vec![
                SerializedEntity {
                    components: {
                        let mut map = HashMap::new();
                        map.insert("Name".to_string(), json!("Floor"));
                        map.insert("Transform".to_string(), json!({
                            "position": [0.0, -1.0, 0.0],
                            "rotation": [0.0, 0.0, 0.0, 1.0],
                            "scale": [20.0, 0.2, 20.0]
                        }));
                        map.insert("Collider".to_string(), json!({
                            "shape": {
                                "Box": {
                                    "half_extents": [0.5, 0.5, 0.5]
                                }
                            },
                            "is_trigger": false
                        }));
                        map
                    },
                },
                SerializedEntity {
                    components: {
                        let mut map = HashMap::new();
                        map.insert("Name".to_string(), json!("Falling Box"));
                        map.insert("Transform".to_string(), json!({
                            "position": [0.0, 5.0, 0.0],
                            "rotation": [0.0, 0.0, 0.0, 1.0],
                            "scale": [1.0, 1.0, 1.0]
                        }));
                        map.insert("Rigidbody".to_string(), json!({
                            "mass": 1.0,
                            "linear_damping": 0.01,
                            "angular_damping": 0.01,
                            "linear_velocity": [0.0, 0.0, 0.0],
                            "angular_velocity": [0.0, 0.0, 0.0],
                            "inertia_tensor": [[0.16666667, 0.0, 0.0], [0.0, 0.16666667, 0.0], [0.0, 0.0, 0.16666667]],
                            "use_gravity": true,
                            "is_kinematic": false
                        }));
                        map.insert("Collider".to_string(), json!({
                            "shape": {
                                "Box": {
                                    "half_extents": [0.5, 0.5, 0.5]
                                }
                            },
                            "is_trigger": false
                        }));
                        map
                    },
                },
            ],
        }
    }

    fn create_valid_test_scene() -> Scene {
        Scene {
            entities: vec![
                SerializedEntity {
                    components: {
                        let mut map = HashMap::new();
                        map.insert("Name".to_string(), json!("Floor"));
                        map.insert("Transform".to_string(), json!({
                            "position": [0.0, -0.5, 0.0],
                            "rotation": [0.0, 0.0, 0.0, 1.0],
                            "scale": [20.0, 1.0, 20.0]
                        }));
                        map.insert("Collider".to_string(), json!({
                            "shape": {
                                "Box": {
                                    "half_extents": [0.5, 0.5, 0.5]
                                }
                            },
                            "is_trigger": false
                        }));
                        map
                    },
                },
                SerializedEntity {
                    components: {
                        let mut map = HashMap::new();
                        map.insert("Name".to_string(), json!("Box"));
                        map.insert("Transform".to_string(), json!({
                            "position": [0.0, 1.0, 0.0],
                            "rotation": [0.0, 0.0, 0.0, 1.0],
                            "scale": [1.0, 1.0, 1.0]
                        }));
                        map.insert("Rigidbody".to_string(), json!({
                            "mass": 1.0,
                            "linear_damping": 0.01,
                            "angular_damping": 0.01,
                            "linear_velocity": [0.0, 0.0, 0.0],
                            "angular_velocity": [0.0, 0.0, 0.0],
                            "inertia_tensor": [[0.16666667, 0.0, 0.0], [0.0, 0.16666667, 0.0], [0.0, 0.0, 0.16666667]],
                            "use_gravity": true,
                            "is_kinematic": false
                        }));
                        map.insert("Collider".to_string(), json!({
                            "shape": {
                                "Box": {
                                    "half_extents": [0.5, 0.5, 0.5]
                                }
                            },
                            "is_trigger": false
                        }));
                        map
                    },
                },
            ],
        }
    }

    #[test]
    fn test_detect_floating_objects() {
        let scene = create_test_scene_with_gap();
        let result = validate_physics_scene(&scene);
        
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        
        // Should detect the large gap
        let has_gap_error = result.errors.iter().any(|e| {
            matches!(e.error_type, ErrorType::NoOverlap { .. })
        });
        assert!(has_gap_error);
    }

    #[test]
    fn test_valid_scene() {
        let scene = create_valid_test_scene();
        let result = validate_physics_scene(&scene);
        
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_thin_floor_warning() {
        let mut scene = create_test_scene_with_gap();
        // Make floor even thinner
        if let Some(entity) = scene.entities.get_mut(0) {
            entity.components.insert("Transform".to_string(), json!({
                "position": [0.0, -1.0, 0.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [20.0, 0.05, 20.0]  // Very thin floor
            }));
        }
        
        let result = validate_physics_scene(&scene);
        
        // Should have warning about thin floor
        let has_thin_warning = result.warnings.iter().any(|w| {
            w.warning.contains("thick")
        });
        assert!(has_thin_warning);
    }
}