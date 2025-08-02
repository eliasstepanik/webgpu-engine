//! Sound propagation algorithms for occlusion and room acoustics

use crate::audio::components::AudioMaterial;
use crate::audio::raycast::{audio_raycast, AudioRay, AudioRayHit};
use crate::audio::MAX_ROOM_SIZE;
use crate::core::entity::{Entity, World};
use glam::Vec3;
use std::f32::consts::TAU;
use tracing::trace;

/// Room acoustics properties
#[derive(Debug, Clone)]
pub struct RoomAcoustics {
    /// Average room size
    pub size: f32,
    /// Reverb time in seconds
    pub reverb_time: f32,
    /// Early reflections data
    pub early_reflections: Vec<Reflection>,
}

/// Single reflection data
#[derive(Debug, Clone)]
pub struct Reflection {
    /// Delay time in seconds
    pub delay: f32,
    /// Amplitude (0.0 to 1.0)
    pub amplitude: f32,
    /// Direction of reflection
    pub direction: Vec3,
}

/// Calculate sound occlusion between listener and source
pub fn calculate_occlusion(
    listener_pos: Vec3,
    source_pos: Vec3,
    world: &World,
    source_entity: Entity,
) -> f32 {
    let direction = (source_pos - listener_pos).normalize();
    let distance = (source_pos - listener_pos).length();

    // Skip if too close
    if distance < 0.1 {
        return 0.0;
    }

    let ray = AudioRay {
        origin: listener_pos,
        direction,
    };

    // Cast ray from listener to source
    let hit = audio_raycast(world, ray, distance, Some(source_entity));

    match hit {
        Some(hit) if hit.distance < distance - 0.1 => {
            // Ray hit something before reaching source
            // Get material properties of hit object
            let material = world
                .get::<&AudioMaterial>(hit.entity)
                .map(|m| **m)
                .unwrap_or_else(|_| AudioMaterial::default());

            trace!(
                "Occlusion hit at distance {:.2}, transmission: {:.2}",
                hit.distance,
                material.transmission
            );

            // Calculate occlusion based on material transmission
            1.0 - material.transmission
        }
        _ => 0.0, // No occlusion
    }
}

/// Calculate multi-bounce occlusion for more realistic sound propagation
pub fn calculate_multi_bounce_occlusion(
    listener_pos: Vec3,
    source_pos: Vec3,
    world: &World,
    source_entity: Entity,
    _max_bounces: usize,
) -> f32 {
    // Direct path occlusion
    let direct_occlusion = calculate_occlusion(listener_pos, source_pos, world, source_entity);

    // If direct path is not fully occluded, use it
    if direct_occlusion < 0.99 {
        return direct_occlusion;
    }

    // Try to find alternative paths with bounces
    let mut min_occlusion = direct_occlusion;

    // Sample directions around the source
    const NUM_SAMPLES: usize = 8;
    for i in 0..NUM_SAMPLES {
        let angle = (i as f32 / NUM_SAMPLES as f32) * TAU;
        let offset = Vec3::new(angle.cos(), 0.0, angle.sin()) * 2.0;
        let bounce_point = source_pos + offset;

        // Check if we can reach the bounce point from source
        let to_bounce = (bounce_point - source_pos).normalize();
        let ray1 = AudioRay {
            origin: source_pos,
            direction: to_bounce,
        };

        if let Some(hit1) = audio_raycast(world, ray1, 2.0, Some(source_entity)) {
            // Check if we can reach listener from bounce point
            let from_bounce = (listener_pos - hit1.point).normalize();
            let ray2 = AudioRay {
                origin: hit1.point + from_bounce * 0.1, // Offset to avoid self-intersection
                direction: from_bounce,
            };

            let distance_to_listener = (listener_pos - hit1.point).length();
            if audio_raycast(world, ray2, distance_to_listener, None).is_none() {
                // Found a valid bounce path
                let material = world
                    .get::<&AudioMaterial>(hit1.entity)
                    .map(|m| **m)
                    .unwrap_or_else(|_| AudioMaterial::default());

                // Calculate attenuation from reflection
                let reflection_loss = material.absorption;
                let path_occlusion = reflection_loss;

                min_occlusion = min_occlusion.min(path_occlusion);
            }
        }
    }

    min_occlusion
}

/// Estimate room acoustics at a given position
pub fn estimate_room_acoustics(position: Vec3, world: &World) -> RoomAcoustics {
    const NUM_RAYS: usize = 16;
    const NUM_VERTICAL: usize = 3;
    let mut hits = Vec::new();

    // Cast rays in multiple directions
    for v in 0..NUM_VERTICAL {
        let vertical_angle = (v as f32 / (NUM_VERTICAL - 1) as f32 - 0.5) * 0.5; // -0.25 to 0.25
        let y_component = vertical_angle.sin();
        let horizontal_scale = vertical_angle.cos();

        for i in 0..NUM_RAYS {
            let angle = (i as f32 / NUM_RAYS as f32) * TAU;
            let dir = Vec3::new(
                angle.cos() * horizontal_scale,
                y_component,
                angle.sin() * horizontal_scale,
            )
            .normalize();

            let ray = AudioRay {
                origin: position,
                direction: dir,
            };

            if let Some(hit) = audio_raycast(world, ray, MAX_ROOM_SIZE, None) {
                hits.push(hit);
            }
        }
    }

    // Analyze room dimensions and materials
    let avg_distance = if !hits.is_empty() {
        hits.iter().map(|h| h.distance).sum::<f32>() / hits.len() as f32
    } else {
        MAX_ROOM_SIZE
    };

    let avg_absorption = if !hits.is_empty() {
        hits.iter()
            .filter_map(|h| world.get::<&AudioMaterial>(h.entity).ok())
            .map(|m| m.absorption)
            .sum::<f32>()
            / hits.len().max(1) as f32
    } else {
        0.1 // Default absorption
    };

    RoomAcoustics {
        size: avg_distance,
        reverb_time: calculate_reverb_time(avg_distance, avg_absorption),
        early_reflections: generate_early_reflections(&hits),
    }
}

/// Calculate reverb time using Sabine's formula
fn calculate_reverb_time(room_size: f32, avg_absorption: f32) -> f32 {
    // Simplified Sabine's formula
    // RT60 = 0.161 * V / (S * α)
    // Where V is volume, S is surface area, α is absorption coefficient

    // Assume roughly cubic room for simplicity
    let volume = room_size.powi(3);
    let surface_area = 6.0 * room_size.powi(2);

    // Prevent division by zero
    let absorption = avg_absorption.max(0.01);

    let rt60 = 0.161 * volume / (surface_area * absorption);

    // Clamp to reasonable values
    rt60.clamp(0.1, 10.0)
}

/// Generate early reflections from raycast hits
fn generate_early_reflections(hits: &[AudioRayHit]) -> Vec<Reflection> {
    let mut reflections = Vec::new();
    const SPEED_OF_SOUND: f32 = 343.0; // meters per second

    // Take first few hits as early reflections
    for (_i, hit) in hits.iter().take(6).enumerate() {
        let delay = hit.distance / SPEED_OF_SOUND;

        // Simple amplitude calculation based on distance and material
        let distance_attenuation = 1.0 / (1.0 + hit.distance * 0.1);
        let amplitude = distance_attenuation * 0.5; // Scale down reflections

        reflections.push(Reflection {
            delay,
            amplitude,
            direction: -hit.normal, // Reflection comes from the hit surface
        });
    }

    reflections
}

/// Calculate diffraction around an edge for partial occlusion
pub fn calculate_diffraction(
    listener_pos: Vec3,
    source_pos: Vec3,
    obstacle_pos: Vec3,
    obstacle_size: f32,
) -> f32 {
    // Simplified diffraction model
    let to_source = (source_pos - obstacle_pos).normalize();
    let to_listener = (listener_pos - obstacle_pos).normalize();

    // Angle between source and listener around obstacle
    let angle = to_source.dot(to_listener).acos();

    // Distance from straight path
    let straight_path = (source_pos - listener_pos).normalize();
    let obstacle_offset = obstacle_pos - listener_pos;
    let distance_from_path = obstacle_offset
        .dot(straight_path.cross(Vec3::Y).normalize())
        .abs();

    // Simple diffraction model
    if distance_from_path < obstacle_size {
        // Partial occlusion based on angle
        let occlusion = (angle / std::f32::consts::PI).clamp(0.0, 1.0);
        occlusion * 0.5 // Maximum 50% occlusion from diffraction
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverb_time_calculation() {
        // Small room with high absorption
        let rt1 = calculate_reverb_time(5.0, 0.8);
        assert!(rt1 < 0.5);

        // Large room with low absorption
        let rt2 = calculate_reverb_time(20.0, 0.1);
        assert!(rt2 > 1.0);

        // Edge case: zero absorption
        let rt3 = calculate_reverb_time(10.0, 0.0);
        assert!(rt3 <= 10.0); // Should be clamped
    }
}
