//! Material component for mesh rendering
//!
//! Provides material properties for rendering, currently supporting
//! basic color. Future versions will support textures and more
//! advanced material properties.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Material component defining surface properties
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Material {
    /// Base color of the material (RGBA)
    pub color: [f32; 4],
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0], // White by default
        }
    }
}

impl Material {
    /// Create a new material with the given color
    pub fn new(color: [f32; 4]) -> Self {
        Self { color }
    }

    /// Create a material from RGB values (alpha = 1.0)
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            color: [r, g, b, 1.0],
        }
    }

    /// Create a material from RGBA values
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            color: [r, g, b, a],
        }
    }

    /// Common preset: red material
    pub fn red() -> Self {
        Self::from_rgb(1.0, 0.0, 0.0)
    }

    /// Common preset: green material
    pub fn green() -> Self {
        Self::from_rgb(0.0, 1.0, 0.0)
    }

    /// Common preset: blue material
    pub fn blue() -> Self {
        Self::from_rgb(0.0, 0.0, 1.0)
    }

    /// Common preset: white material
    pub fn white() -> Self {
        Self::from_rgb(1.0, 1.0, 1.0)
    }

    /// Common preset: black material
    pub fn black() -> Self {
        Self::from_rgb(0.0, 0.0, 0.0)
    }

    /// Common preset: gray material
    pub fn gray(value: f32) -> Self {
        Self::from_rgb(value, value, value)
    }
}

/// Material data for GPU uniform buffer
///
/// This struct is aligned for GPU uniform buffer requirements
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MaterialUniform {
    /// Base color of the material (RGBA)
    pub color: [f32; 4],
}

impl From<Material> for MaterialUniform {
    fn from(material: Material) -> Self {
        Self {
            color: material.color,
        }
    }
}

impl From<&Material> for MaterialUniform {
    fn from(material: &Material) -> Self {
        Self {
            color: material.color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_default() {
        let mat = Material::default();
        assert_eq!(mat.color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_material_presets() {
        assert_eq!(Material::red().color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(Material::green().color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(Material::blue().color, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(Material::white().color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(Material::black().color, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(Material::gray(0.5).color, [0.5, 0.5, 0.5, 1.0]);
    }

    #[test]
    fn test_material_from_rgb() {
        let mat = Material::from_rgb(0.2, 0.3, 0.4);
        assert_eq!(mat.color, [0.2, 0.3, 0.4, 1.0]);
    }

    #[test]
    fn test_material_from_rgba() {
        let mat = Material::from_rgba(0.2, 0.3, 0.4, 0.5);
        assert_eq!(mat.color, [0.2, 0.3, 0.4, 0.5]);
    }

    #[test]
    fn test_material_uniform_conversion() {
        let mat = Material::from_rgba(0.1, 0.2, 0.3, 0.4);
        let uniform: MaterialUniform = mat.into();
        assert_eq!(uniform.color, mat.color);
    }

    #[test]
    fn test_material_uniform_size() {
        use std::mem;
        // Ensure MaterialUniform is the expected size for GPU
        assert_eq!(mem::size_of::<MaterialUniform>(), 16); // 4 floats * 4 bytes
    }
}
