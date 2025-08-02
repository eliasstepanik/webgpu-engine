//! Ambient soundscape system

use glam::Vec3;

/// Ambient sound parameters
#[derive(Debug, Clone)]
pub struct AmbientSoundscape {
    /// Base ambient sounds (wind, room tone, etc.)
    pub layers: Vec<AmbientLayer>,
    /// Global volume multiplier
    pub master_volume: f32,
}

/// Individual ambient sound layer
#[derive(Debug, Clone)]
pub struct AmbientLayer {
    /// Sound file path
    pub sound_path: String,
    /// Base volume
    pub volume: f32,
    /// Volume variation over time
    pub volume_variation: f32,
    /// Variation period in seconds
    pub variation_period: f32,
    /// Current phase
    pub phase: f32,
}

impl AmbientLayer {
    /// Update volume based on time
    pub fn get_current_volume(&self, delta_time: f32) -> f32 {
        let phase = self.phase + delta_time / self.variation_period;
        let variation = (phase * std::f32::consts::TAU).sin() * self.volume_variation;
        (self.volume + variation).clamp(0.0, 1.0)
    }
}

/// Calculate ambient volume based on environment
pub fn calculate_ambient_volume(
    listener_pos: Vec3,
    zone_center: Vec3,
    zone_radius: f32,
    base_volume: f32,
) -> f32 {
    let distance = (listener_pos - zone_center).length();

    if distance > zone_radius {
        0.0
    } else {
        // Smooth falloff at zone edges
        let normalized = distance / zone_radius;
        let falloff = 1.0 - normalized.powi(2);
        base_volume * falloff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambient_volume_calculation() {
        let listener = Vec3::ZERO;
        let zone_center = Vec3::new(10.0, 0.0, 0.0);
        let zone_radius = 20.0;

        // At zone center
        let vol1 = calculate_ambient_volume(zone_center, zone_center, zone_radius, 1.0);
        assert_eq!(vol1, 1.0);

        // Outside zone
        let vol2 =
            calculate_ambient_volume(Vec3::new(50.0, 0.0, 0.0), zone_center, zone_radius, 1.0);
        assert_eq!(vol2, 0.0);

        // At zone edge
        let vol3 =
            calculate_ambient_volume(Vec3::new(30.0, 0.0, 0.0), zone_center, zone_radius, 1.0);
        assert_eq!(vol3, 0.0);
    }

    #[test]
    fn test_ambient_layer_variation() {
        let layer = AmbientLayer {
            sound_path: "wind.ogg".to_string(),
            volume: 0.5,
            volume_variation: 0.2,
            variation_period: 10.0,
            phase: 0.0,
        };

        let vol = layer.get_current_volume(0.0);
        assert!((vol - 0.5).abs() < 0.21);
    }
}
