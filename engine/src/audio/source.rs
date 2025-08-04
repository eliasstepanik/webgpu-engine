//! Audio source management and spatial calculations

use crate::audio::AudioHandle;
use glam::Vec3;
use tracing::debug;

/// Spatial audio parameters for a sound source
#[derive(Debug, Clone)]
pub struct SpatialParams {
    /// Source position in world space
    pub position: Vec3,
    /// Source velocity (for Doppler effect)
    pub velocity: Vec3,
    /// Maximum distance for attenuation
    pub max_distance: f32,
    /// Rolloff factor for distance attenuation
    pub rolloff_factor: f32,
}

/// Calculate distance attenuation
pub fn calculate_distance_attenuation(
    distance: f32,
    max_distance: f32,
    rolloff_factor: f32,
) -> f32 {
    if distance >= max_distance {
        return 0.0;
    }

    // Inverse distance model with rolloff
    let reference_distance = 1.0;
    let clamped_distance = distance.max(reference_distance);

    match rolloff_factor {
        // Linear rolloff
        f if f <= 0.0 => 1.0,
        // Standard inverse distance
        f if (f - 1.0).abs() < 0.001 => reference_distance / clamped_distance,
        // Custom rolloff
        _ => {
            let attenuation = reference_distance
                / (reference_distance + rolloff_factor * (clamped_distance - reference_distance));
            attenuation.clamp(0.0, 1.0)
        }
    }
}

/// Calculate stereo panning based on position relative to listener
pub fn calculate_panning(
    source_pos: Vec3,
    listener_pos: Vec3,
    _listener_forward: Vec3,
    listener_right: Vec3,
) -> f32 {
    // Handle case where source is at listener position
    let to_source_vec = source_pos - listener_pos;
    if to_source_vec.length_squared() < 0.001 {
        return 0.0; // Center pan
    }

    let to_source = to_source_vec.normalize();

    // Project onto listener's right vector
    let right_component = to_source.dot(listener_right);

    // Convert to panning value (-1.0 = full left, 1.0 = full right)
    right_component.clamp(-1.0, 1.0)
}

/// Calculate Doppler effect pitch shift
pub fn calculate_doppler_shift(
    source_pos: Vec3,
    source_velocity: Vec3,
    listener_pos: Vec3,
    listener_velocity: Vec3,
    speed_of_sound: f32,
) -> f32 {
    // Direction from listener to source
    let direction = (source_pos - listener_pos).normalize();

    // Relative velocities along the line of sight
    let source_speed = source_velocity.dot(direction);
    let listener_speed = listener_velocity.dot(direction);

    // Doppler formula
    let doppler = (speed_of_sound + listener_speed) / (speed_of_sound - source_speed);

    // Clamp to reasonable values to prevent extreme pitch shifts
    doppler.clamp(0.5, 2.0)
}

/// Apply spatial audio parameters to a sound handle
pub fn apply_spatial_params(
    handle: &AudioHandle,
    source_params: &SpatialParams,
    listener_pos: Vec3,
    listener_forward: Vec3,
    listener_right: Vec3,
    listener_velocity: Vec3,
    occlusion: f32,
) {
    let distance = (source_params.position - listener_pos).length();

    // Calculate volume from distance and occlusion
    let distance_attenuation = calculate_distance_attenuation(
        distance,
        source_params.max_distance,
        source_params.rolloff_factor,
    );

    // Apply occlusion
    let base_volume = distance_attenuation * (1.0 - occlusion * 0.8); // Leave some sound even when occluded

    // Calculate panning
    let pan = calculate_panning(
        source_params.position,
        listener_pos,
        listener_forward,
        listener_right,
    );

    // Calculate Doppler effect
    let doppler = calculate_doppler_shift(
        source_params.position,
        source_params.velocity,
        listener_pos,
        listener_velocity,
        343.0, // Speed of sound in m/s
    );

    // Apply spatial parameters
    debug!(
        distance = distance,
        distance_attenuation = distance_attenuation,
        occlusion = occlusion,
        base_volume = base_volume,
        pan = pan,
        doppler = doppler,
        "Applying spatial audio parameters"
    );
    handle.set_volume(base_volume, None);
    handle.set_playback_rate(doppler, None);
    handle.set_panning(pan); // Apply real stereo panning
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_attenuation() {
        // At reference distance
        let att1 = calculate_distance_attenuation(1.0, 50.0, 1.0);
        assert!((att1 - 1.0).abs() < 0.001);

        // Halfway to max distance
        let att2 = calculate_distance_attenuation(25.0, 50.0, 1.0);
        assert!(att2 > 0.0 && att2 < 1.0);

        // Beyond max distance
        let att3 = calculate_distance_attenuation(60.0, 50.0, 1.0);
        assert_eq!(att3, 0.0);
    }

    #[test]
    fn test_doppler_shift() {
        let speed_of_sound = 343.0;

        // Stationary source and listener
        let shift1 = calculate_doppler_shift(
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            speed_of_sound,
        );
        assert!((shift1 - 1.0).abs() < 0.001);

        // Source moving toward listener
        let shift2 = calculate_doppler_shift(
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(-10.0, 0.0, 0.0), // Moving left toward listener at origin
            Vec3::ZERO,
            Vec3::ZERO,
            speed_of_sound,
        );
        assert!(shift2 > 1.0); // Higher pitch

        // Source moving away from listener
        let shift3 = calculate_doppler_shift(
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0), // Moving right away from listener
            Vec3::ZERO,
            Vec3::ZERO,
            speed_of_sound,
        );
        assert!(shift3 < 1.0); // Lower pitch
    }
}
