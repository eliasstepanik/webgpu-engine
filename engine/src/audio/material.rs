//! Audio material properties for sound absorption and transmission

use crate::audio::components::AudioMaterial;

impl Default for AudioMaterial {
    fn default() -> Self {
        Self {
            absorption: 0.1,   // Slightly absorptive (like concrete)
            scattering: 0.5,   // Half diffuse, half specular
            transmission: 0.0, // Fully occluding
        }
    }
}

impl AudioMaterial {
    /// Concrete material - hard, reflective
    pub fn concrete() -> Self {
        Self {
            absorption: 0.02,
            scattering: 0.1,
            transmission: 0.0,
        }
    }

    /// Wood material - moderate absorption
    pub fn wood() -> Self {
        Self {
            absorption: 0.15,
            scattering: 0.5,
            transmission: 0.0,
        }
    }

    /// Glass material - reflective but transmissive
    pub fn glass() -> Self {
        Self {
            absorption: 0.05,
            scattering: 0.1,
            transmission: 0.8,
        }
    }

    /// Fabric material - highly absorptive
    pub fn fabric() -> Self {
        Self {
            absorption: 0.6,
            scattering: 0.9,
            transmission: 0.3,
        }
    }

    /// Metal material - highly reflective
    pub fn metal() -> Self {
        Self {
            absorption: 0.01,
            scattering: 0.05,
            transmission: 0.0,
        }
    }
}

/// Calculate the occlusion factor based on material properties
pub fn calculate_occlusion(materials: &[AudioMaterial]) -> f32 {
    if materials.is_empty() {
        return 0.0;
    }

    // Combine transmission values (multiplicative for multiple layers)
    let total_transmission = materials
        .iter()
        .map(|m| m.transmission)
        .fold(1.0, |acc, t| acc * t);

    // Occlusion is inverse of transmission
    1.0 - total_transmission
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_presets() {
        let concrete = AudioMaterial::concrete();
        assert!(concrete.absorption < 0.1);
        assert_eq!(concrete.transmission, 0.0);

        let glass = AudioMaterial::glass();
        assert!(glass.transmission > 0.5);
    }

    #[test]
    fn test_occlusion_calculation() {
        let materials = vec![AudioMaterial::glass(), AudioMaterial::wood()];
        let occlusion = calculate_occlusion(&materials);
        assert!(occlusion > 0.0 && occlusion < 1.0);
    }
}
