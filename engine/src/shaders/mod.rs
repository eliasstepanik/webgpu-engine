//! Shader management and compilation
//!
//! Provides access to compiled shaders for the rendering pipeline.

/// Basic vertex and fragment shader for 3D rendering
pub const BASIC_SHADER: &str = include_str!("basic.wgsl");

/// Outline shader for selection highlighting
pub const OUTLINE_SHADER: &str = include_str!("outline.wgsl");

/// Debug lines shader for visualizing colliders and debug geometry
pub const DEBUG_LINES_SHADER: &str = include_str!("debug_lines.wgsl");
