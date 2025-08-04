//! Enhanced physical audio occlusion system
//!
//! This module provides physically-based audio occlusion using multiple rays
//! and frequency-dependent attenuation for realistic sound propagation.

use crate::audio::components::AudioMaterial;
use crate::audio::raycast::{audio_raycast, AudioRay};
use crate::core::entity::{Entity, World};
use glam::Vec3;
use std::f32::consts::PI;
use tracing::{debug, trace};

/// Configuration for physical occlusion calculation
#[derive(Debug, Clone)]
pub struct OcclusionConfig {
    /// Number of rays to cast for occlusion testing
    pub num_rays: usize,
    /// Maximum diffraction angle in radians
    pub max_diffraction_angle: f32,
    /// Enable frequency-dependent attenuation
    pub frequency_dependent: bool,
    /// Frequency bands for analysis (Hz)
    pub frequency_bands: Vec<f32>,
}

impl Default for OcclusionConfig {
    fn default() -> Self {
        Self {
            num_rays: 5,
            max_diffraction_angle: PI / 4.0,
            frequency_dependent: true,
            frequency_bands: vec![125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0],
        }
    }
}

/// Result of physical occlusion calculation
#[derive(Debug, Clone)]
pub struct OcclusionResult {
    /// Overall occlusion factor (0.0 = no occlusion, 1.0 = fully occluded)
    pub occlusion: f32,
    /// Frequency-dependent attenuation factors
    pub frequency_attenuation: Vec<f32>,
    /// Whether any direct path exists
    pub has_direct_path: bool,
    /// Estimated diffraction paths
    pub diffraction_paths: Vec<DiffractionPath>,
}

/// Represents a sound diffraction path
#[derive(Debug, Clone)]
pub struct DiffractionPath {
    /// Points along the diffraction path
    pub points: Vec<Vec3>,
    /// Total path length
    pub length: f32,
    /// Attenuation factor for this path
    pub attenuation: f32,
}

/// Calculate physically-based audio occlusion
pub fn calculate_physical_occlusion(
    listener_pos: Vec3,
    source_pos: Vec3,
    world: &World,
    source_entity: Entity,
    config: &OcclusionConfig,
) -> OcclusionResult {
    let direct_distance = (source_pos - listener_pos).length();

    // Skip if too close
    if direct_distance < 0.1 {
        return OcclusionResult {
            occlusion: 0.0,
            frequency_attenuation: vec![1.0; config.frequency_bands.len()],
            has_direct_path: true,
            diffraction_paths: vec![],
        };
    }

    // Cast multiple rays for robustness
    let (occlusion, materials) = cast_occlusion_rays(
        listener_pos,
        source_pos,
        world,
        source_entity,
        config.num_rays,
    );

    // Check for direct path
    let has_direct_path = occlusion < 0.99;

    // Calculate frequency-dependent attenuation
    let frequency_attenuation = if config.frequency_dependent {
        calculate_frequency_response(&materials, &config.frequency_bands, occlusion)
    } else {
        vec![1.0 - occlusion; config.frequency_bands.len()]
    };

    // Find diffraction paths if fully occluded
    let diffraction_paths = if !has_direct_path {
        find_diffraction_paths(
            listener_pos,
            source_pos,
            world,
            source_entity,
            config.max_diffraction_angle,
        )
    } else {
        vec![]
    };

    debug!(
        "Physical occlusion: {:.2}, direct_path: {}, diffraction_paths: {}",
        occlusion,
        has_direct_path,
        diffraction_paths.len()
    );

    OcclusionResult {
        occlusion,
        frequency_attenuation,
        has_direct_path,
        diffraction_paths,
    }
}

/// Cast multiple rays to determine occlusion
fn cast_occlusion_rays(
    listener_pos: Vec3,
    source_pos: Vec3,
    world: &World,
    source_entity: Entity,
    num_rays: usize,
) -> (f32, Vec<AudioMaterial>) {
    let mut total_occlusion = 0.0;
    let mut hit_materials = Vec::new();
    let base_direction = (source_pos - listener_pos).normalize();
    let distance = (source_pos - listener_pos).length();

    // Cast central ray
    let central_ray = AudioRay {
        origin: listener_pos,
        direction: base_direction,
    };

    if let Some(hit) = audio_raycast(world, central_ray, distance, Some(source_entity)) {
        if hit.distance < distance - 0.1 {
            let material = world
                .get::<&AudioMaterial>(hit.entity)
                .map(|m| **m)
                .unwrap_or_else(|_| AudioMaterial::default());

            total_occlusion += 1.0 - material.transmission;
            hit_materials.push(material);
        }
    }

    // Cast additional rays in a cone pattern
    if num_rays > 1 {
        let cone_angle = 0.1; // radians
        let rays_per_ring = num_rays - 1;

        for i in 0..rays_per_ring {
            let angle = (i as f32 / rays_per_ring as f32) * 2.0 * PI;

            // Create offset direction
            let right = base_direction.cross(Vec3::Y).normalize();
            let up = base_direction.cross(right);

            let offset_dir = (base_direction
                + right * (angle.cos() * cone_angle)
                + up * (angle.sin() * cone_angle))
                .normalize();

            let ray = AudioRay {
                origin: listener_pos,
                direction: offset_dir,
            };

            if let Some(hit) = audio_raycast(world, ray, distance * 1.1, Some(source_entity)) {
                if hit.distance < distance {
                    let material = world
                        .get::<&AudioMaterial>(hit.entity)
                        .map(|m| **m)
                        .unwrap_or_else(|_| AudioMaterial::default());

                    total_occlusion += 1.0 - material.transmission;
                    hit_materials.push(material);
                }
            }
        }

        total_occlusion /= num_rays as f32;
    }

    (total_occlusion.clamp(0.0, 1.0), hit_materials)
}

/// Calculate frequency-dependent response based on materials
fn calculate_frequency_response(
    materials: &[AudioMaterial],
    frequency_bands: &[f32],
    base_occlusion: f32,
) -> Vec<f32> {
    if materials.is_empty() {
        return vec![1.0; frequency_bands.len()];
    }

    // Average material properties
    let avg_absorption =
        materials.iter().map(|m| m.absorption).sum::<f32>() / materials.len() as f32;

    // Use absorption as a proxy for density effect
    // Materials with high absorption tend to block high frequencies more
    frequency_bands
        .iter()
        .map(|&freq| {
            // Higher frequencies are attenuated more by absorptive materials
            // Using simplified model: attenuation increases with frequency and absorption
            let freq_factor = (freq / 1000.0).ln().max(0.0) / 3.0; // Logarithmic scaling
            let absorption_factor = avg_absorption * 2.0; // Scale absorption effect

            let additional_attenuation = freq_factor * absorption_factor * 0.3;
            let total_attenuation = base_occlusion + additional_attenuation * base_occlusion;

            (1.0 - total_attenuation).max(0.0)
        })
        .collect()
}

/// Find alternative paths through diffraction
fn find_diffraction_paths(
    listener_pos: Vec3,
    source_pos: Vec3,
    world: &World,
    source_entity: Entity,
    max_angle: f32,
) -> Vec<DiffractionPath> {
    let mut paths = Vec::new();
    let direct_dir = (source_pos - listener_pos).normalize();
    let direct_distance = (source_pos - listener_pos).length();

    // Try diffraction around edges by sampling points around obstacles
    const NUM_SAMPLES: usize = 8;
    const EDGE_SEARCH_RADIUS: f32 = 5.0;

    for i in 0..NUM_SAMPLES {
        let angle = (i as f32 / NUM_SAMPLES as f32) * 2.0 * PI;

        // Create a perpendicular offset
        let right = direct_dir.cross(Vec3::Y).normalize();
        let offset = right * angle.cos() + Vec3::Y * angle.sin();

        // Try to find an edge point
        let edge_point =
            listener_pos + direct_dir * (direct_distance * 0.5) + offset * EDGE_SEARCH_RADIUS;

        // Check if we can reach the edge point from listener
        let to_edge = (edge_point - listener_pos).normalize();
        let ray_to_edge = AudioRay {
            origin: listener_pos,
            direction: to_edge,
        };

        let edge_distance = (edge_point - listener_pos).length();
        let hit_to_edge = audio_raycast(world, ray_to_edge, edge_distance, None);

        // Check if we can reach source from edge point
        let from_edge = (source_pos - edge_point).normalize();
        let ray_from_edge = AudioRay {
            origin: edge_point,
            direction: from_edge,
        };

        let source_distance = (source_pos - edge_point).length();
        let hit_from_edge =
            audio_raycast(world, ray_from_edge, source_distance, Some(source_entity));

        // If both paths are clear, we have a diffraction path
        if hit_to_edge.is_none() && hit_from_edge.is_none() {
            let path_length = edge_distance + source_distance;
            let diffraction_angle = to_edge.dot(-from_edge).acos();

            if diffraction_angle <= max_angle {
                // Calculate attenuation based on diffraction angle and path length
                let angle_attenuation = 1.0 - (diffraction_angle / max_angle).powi(2);
                let distance_attenuation = direct_distance / path_length;
                let total_attenuation = angle_attenuation * distance_attenuation * 0.5;

                paths.push(DiffractionPath {
                    points: vec![listener_pos, edge_point, source_pos],
                    length: path_length,
                    attenuation: total_attenuation,
                });

                trace!(
                    "Found diffraction path: angle={:.2}°, length={:.2}m, attenuation={:.2}",
                    diffraction_angle.to_degrees(),
                    path_length,
                    total_attenuation
                );
            }
        }
    }

    // Sort by attenuation (best paths first)
    paths.sort_by(|a, b| b.attenuation.partial_cmp(&a.attenuation).unwrap());
    paths.truncate(3); // Keep only best paths

    paths
}

/// Apply occlusion result to audio output
pub fn apply_occlusion_to_audio(
    handle: &crate::audio::AudioHandle,
    result: &OcclusionResult,
    volume: f32,
) {
    // For now, apply simple volume reduction based on occlusion
    // In a more advanced system, this would apply frequency filters
    let occluded_volume = volume * (1.0 - result.occlusion);

    // If we have diffraction paths, add their contribution
    let diffraction_contribution = result
        .diffraction_paths
        .iter()
        .map(|p| p.attenuation)
        .fold(0.0_f32, |max, val| max.max(val));

    let final_volume =
        (occluded_volume + volume * diffraction_contribution * result.occlusion).min(volume);

    handle.set_volume(final_volume, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_response() {
        let materials = vec![AudioMaterial {
            absorption: 0.5,
            transmission: 0.3,
            scattering: 0.2,
        }];

        let frequencies = vec![125.0, 500.0, 2000.0, 8000.0];
        let response = calculate_frequency_response(&materials, &frequencies, 0.5);

        // Higher frequencies should be more attenuated
        for i in 1..response.len() {
            assert!(response[i] <= response[i - 1]);
        }
    }

    #[test]
    fn test_occlusion_config_default() {
        let config = OcclusionConfig::default();
        assert_eq!(config.num_rays, 5);
        assert!(config.frequency_dependent);
        assert_eq!(config.frequency_bands.len(), 7);
    }
}
