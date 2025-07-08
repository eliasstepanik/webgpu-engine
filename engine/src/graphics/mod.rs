//! Graphics module
//!
//! Provides rendering functionality including meshes, materials,
//! render pipelines, and the main renderer.

pub mod context;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod renderer;
pub mod uniform;

// Re-export commonly used types
pub use context::RenderContext;
pub use material::{Material, MaterialUniform};
pub use mesh::{Mesh, Vertex};
pub use pipeline::{DepthTexture, RenderPipeline};
pub use renderer::{MeshId, Renderer};
pub use uniform::{CameraUniform, ObjectUniform, UniformBuffer};
