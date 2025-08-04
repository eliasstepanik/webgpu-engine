//! Mesh-based audio occlusion for physically accurate sound propagation
//!
//! This module provides ray-triangle intersection tests for precise
//! audio occlusion calculations using actual mesh geometry.

use crate::core::entity::{Entity, Transform, World};
use crate::graphics::mesh::Mesh;
use crate::graphics::renderer::MeshId;
use crate::graphics::culling::AABB;
use glam::Vec3;
use tracing::debug;

/// Represents a triangle in world space
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub normal: Vec3,
}

impl Triangle {
    /// Create a triangle from three vertices and calculate its normal
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();
        
        Self { v0, v1, v2, normal }
    }
}

/// Ray-triangle intersection using Möller–Trumbore algorithm
///
/// Returns the distance to the intersection point if the ray hits the triangle
pub fn ray_triangle_intersection(
    origin: Vec3,
    direction: Vec3,
    triangle: &Triangle,
    max_distance: f32,
) -> Option<f32> {
    const EPSILON: f32 = 1e-6;
    
    let edge1 = triangle.v1 - triangle.v0;
    let edge2 = triangle.v2 - triangle.v0;
    
    let h = direction.cross(edge2);
    let a = edge1.dot(h);
    
    // Ray is parallel to triangle
    if a.abs() < EPSILON {
        return None;
    }
    
    let f = 1.0 / a;
    let s = origin - triangle.v0;
    let u = f * s.dot(h);
    
    // Outside triangle bounds
    if u < 0.0 || u > 1.0 {
        return None;
    }
    
    let q = s.cross(edge1);
    let v = f * direction.dot(q);
    
    // Outside triangle bounds
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    
    // Calculate intersection distance
    let t = f * edge2.dot(q);
    
    // Check if intersection is within valid range
    if t > EPSILON && t < max_distance {
        Some(t)
    } else {
        None
    }
}

/// Enhanced hit information including mesh details
#[derive(Debug, Clone)]
pub struct MeshRayHit {
    /// Entity that was hit
    pub entity: Entity,
    /// Distance to the hit point
    pub distance: f32,
    /// Hit point in world space
    pub point: Vec3,
    /// Surface normal at hit point
    pub normal: Vec3,
    /// Triangle index that was hit
    pub triangle_index: usize,
    /// Whether the hit was on a mesh or just AABB
    pub is_mesh_hit: bool,
}

/// Perform mesh-based audio raycast
///
/// This function first uses AABB tests for broad phase, then performs
/// precise ray-triangle intersection tests for entities with meshes.
pub fn mesh_audio_raycast(
    world: &World,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    exclude: Option<Entity>,
    mesh_library: &crate::graphics::mesh_library::MeshLibrary,
) -> Option<MeshRayHit> {
    let mut closest_hit: Option<MeshRayHit> = None;
    let normalized_dir = direction.normalize();
    
    // First pass: AABB broad phase
    let mut potential_hits = Vec::new();
    
    for (entity, (aabb, transform)) in world.query::<(&AABB, &Transform)>().iter() {
        if Some(entity) == exclude {
            continue;
        }
        
        // Transform AABB to world space
        let world_aabb = transform_aabb(aabb, transform);
        
        // Test ray-AABB intersection
        if let Some(aabb_distance) = ray_aabb_intersection(
            origin,
            normalized_dir,
            &world_aabb,
            max_distance,
        ) {
            potential_hits.push((entity, transform.clone(), aabb_distance));
        }
    }
    
    // Sort by distance for early exit optimization
    potential_hits.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    
    // Second pass: Precise mesh intersection tests
    for (entity, transform, aabb_distance) in potential_hits {
        // Skip if we already found a closer hit
        if let Some(ref hit) = closest_hit {
            if hit.distance < aabb_distance {
                continue;
            }
        }
        
        // Check if entity has a mesh
        if let Ok(mesh_id) = world.get::<&MeshId>(entity) {
            if let Some(mesh_data) = mesh_library.get(&mesh_id.0) {
                // Test against mesh triangles
                if let Some(hit) = test_mesh_intersection(
                    origin,
                    normalized_dir,
                    max_distance,
                    mesh_data,
                    &transform,
                    entity,
                ) {
                    if closest_hit.as_ref().map_or(true, |ch| hit.distance < ch.distance) {
                        closest_hit = Some(hit);
                    }
                }
            } else {
                // No mesh data, use AABB hit
                let hit_point = origin + normalized_dir * aabb_distance;
                let hit = MeshRayHit {
                    entity,
                    distance: aabb_distance,
                    point: hit_point,
                    normal: calculate_aabb_normal(hit_point, &world.get::<&AABB>(entity).ok().map(|a| **a).unwrap_or_default()),
                    triangle_index: 0,
                    is_mesh_hit: false,
                };
                
                if closest_hit.as_ref().map_or(true, |ch| hit.distance < ch.distance) {
                    closest_hit = Some(hit);
                }
            }
        } else {
            // No mesh component, use AABB hit
            let hit_point = origin + normalized_dir * aabb_distance;
            let hit = MeshRayHit {
                entity,
                distance: aabb_distance,
                point: hit_point,
                normal: calculate_aabb_normal(hit_point, &world.get::<&AABB>(entity).ok().map(|a| **a).unwrap_or_default()),
                triangle_index: 0,
                is_mesh_hit: false,
            };
            
            if closest_hit.as_ref().map_or(true, |ch| hit.distance < ch.distance) {
                closest_hit = Some(hit);
            }
        }
    }
    
    if let Some(ref hit) = closest_hit {
        debug!(
            "Mesh raycast hit: entity={:?}, distance={:.2}, is_mesh={}", 
            hit.entity, hit.distance, hit.is_mesh_hit
        );
    }
    
    closest_hit
}

/// Test ray intersection against a mesh
fn test_mesh_intersection(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    mesh: &Mesh,
    transform: &Transform,
    entity: Entity,
) -> Option<MeshRayHit> {
    let mut closest_hit: Option<(f32, usize, Vec3)> = None;
    
    // Transform ray to local space
    let local_origin = transform.rotation.inverse() * ((origin - transform.position) / transform.scale);
    let local_direction = transform.rotation.inverse() * direction;
    
    // Test each triangle
    for i in (0..mesh.indices.len()).step_by(3) {
        let i0 = mesh.indices[i] as usize;
        let i1 = mesh.indices[i + 1] as usize;
        let i2 = mesh.indices[i + 2] as usize;
        
        let v0 = Vec3::from(mesh.vertices[i0].position);
        let v1 = Vec3::from(mesh.vertices[i1].position);
        let v2 = Vec3::from(mesh.vertices[i2].position);
        
        let triangle = Triangle::new(v0, v1, v2);
        
        if let Some(distance) = ray_triangle_intersection(
            local_origin,
            local_direction,
            &triangle,
            max_distance,
        ) {
            if closest_hit.as_ref().map_or(true, |(d, _, _)| distance < *d) {
                closest_hit = Some((distance, i / 3, triangle.normal));
            }
        }
    }
    
    closest_hit.map(|(distance, triangle_index, local_normal)| {
        // Transform hit back to world space
        let local_hit = local_origin + local_direction * distance;
        let world_hit = transform.position + transform.rotation * (local_hit * transform.scale);
        let world_normal = (transform.rotation * local_normal).normalize();
        
        MeshRayHit {
            entity,
            distance: (world_hit - origin).length(),
            point: world_hit,
            normal: world_normal,
            triangle_index,
            is_mesh_hit: true,
        }
    })
}

/// Transform AABB from local to world space
fn transform_aabb(aabb: &AABB, transform: &Transform) -> AABB {
    let corners = [
        Vec3::new(aabb.min.x, aabb.min.y, aabb.min.z),
        Vec3::new(aabb.max.x, aabb.min.y, aabb.min.z),
        Vec3::new(aabb.min.x, aabb.max.y, aabb.min.z),
        Vec3::new(aabb.max.x, aabb.max.y, aabb.min.z),
        Vec3::new(aabb.min.x, aabb.min.y, aabb.max.z),
        Vec3::new(aabb.max.x, aabb.min.y, aabb.max.z),
        Vec3::new(aabb.min.x, aabb.max.y, aabb.max.z),
        Vec3::new(aabb.max.x, aabb.max.y, aabb.max.z),
    ];
    
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    
    for corner in &corners {
        let world_corner = transform.position + transform.rotation * (*corner * transform.scale);
        min = min.min(world_corner);
        max = max.max(world_corner);
    }
    
    AABB { min, max }
}

/// Simplified ray-AABB intersection
fn ray_aabb_intersection(
    origin: Vec3,
    direction: Vec3,
    aabb: &AABB,
    max_distance: f32,
) -> Option<f32> {
    let inv_dir = Vec3::new(
        if direction.x.abs() < f32::EPSILON { f32::INFINITY } else { 1.0 / direction.x },
        if direction.y.abs() < f32::EPSILON { f32::INFINITY } else { 1.0 / direction.y },
        if direction.z.abs() < f32::EPSILON { f32::INFINITY } else { 1.0 / direction.z },
    );
    
    let t1 = (aabb.min - origin) * inv_dir;
    let t2 = (aabb.max - origin) * inv_dir;
    
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);
    
    let tmin = tmin.x.max(tmin.y).max(tmin.z).max(0.0);
    let tmax = tmax.x.min(tmax.y).min(tmax.z).min(max_distance);
    
    if tmin <= tmax {
        Some(tmin)
    } else {
        None
    }
}

/// Calculate normal for AABB hit
fn calculate_aabb_normal(hit_point: Vec3, aabb: &AABB) -> Vec3 {
    let center = (aabb.min + aabb.max) * 0.5;
    let local_point = hit_point - center;
    let half_extents = (aabb.max - aabb.min) * 0.5;
    
    let relative = local_point / half_extents;
    let abs_relative = relative.abs();
    
    if abs_relative.x > abs_relative.y && abs_relative.x > abs_relative.z {
        Vec3::new(relative.x.signum(), 0.0, 0.0)
    } else if abs_relative.y > abs_relative.z {
        Vec3::new(0.0, relative.y.signum(), 0.0)
    } else {
        Vec3::new(0.0, 0.0, relative.z.signum())
    }
}

/// Calculate frequency-dependent attenuation for occluded sounds
///
/// Higher frequencies are attenuated more by obstacles
pub fn calculate_frequency_attenuation(
    occlusion: f32,
    frequency_bands: &[f32],
) -> Vec<f32> {
    frequency_bands.iter().map(|&freq| {
        // Higher frequencies are blocked more
        let freq_factor = (freq / 1000.0).ln().max(0.0) / 3.0;
        let attenuation = occlusion * (1.0 + freq_factor * 0.5);
        1.0 - attenuation.min(1.0)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ray_triangle_intersection() {
        let triangle = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        
        // Ray hitting the triangle
        let hit = ray_triangle_intersection(
            Vec3::new(0.25, 0.25, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            &triangle,
            10.0,
        );
        assert!(hit.is_some());
        assert!((hit.unwrap() - 1.0).abs() < 0.001);
        
        // Ray missing the triangle
        let miss = ray_triangle_intersection(
            Vec3::new(2.0, 2.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            &triangle,
            10.0,
        );
        assert!(miss.is_none());
    }
    
    #[test]
    fn test_frequency_attenuation() {
        let frequencies = vec![100.0, 500.0, 1000.0, 5000.0];
        let attenuation = calculate_frequency_attenuation(0.5, &frequencies);
        
        // Higher frequencies should be attenuated more
        for i in 1..attenuation.len() {
            assert!(attenuation[i] <= attenuation[i-1]);
        }
    }
}