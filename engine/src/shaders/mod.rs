//! Shader management and compilation
//!
//! Provides access to compiled shaders for the rendering pipeline.

/// Basic vertex and fragment shader for 3D rendering
pub const BASIC_SHADER: &str = include_str!("basic.wgsl");

/// Outline shader for selection highlighting
pub const OUTLINE_SHADER: &str = include_str!("outline.wgsl");
