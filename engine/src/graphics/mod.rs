//! Graphics module
//!
//! Provides rendering functionality including meshes, materials,
//! render pipelines, and the main renderer.

pub mod asset_manager;
pub mod context;
pub mod material;
pub mod mesh;
pub mod mesh_library;
pub mod pipeline;
pub mod render_target;
pub mod renderer;
pub mod uniform;

// Re-export commonly used types
pub use asset_manager::{AssetManager, AssetValidationReport, AssetValidationSummary};
pub use context::RenderContext;
pub use material::{Material, MaterialUniform};
pub use mesh::{Mesh, Vertex};
pub use mesh_library::MeshLibrary;
pub use pipeline::{DepthTexture, RenderPipeline};
pub use render_target::RenderTarget;
pub use renderer::{MeshId, Renderer};
pub use uniform::{CameraUniform, ObjectUniform, UniformBuffer};
