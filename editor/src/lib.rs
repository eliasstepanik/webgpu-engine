//! ImGui-based scene editor for WebGPU engine
//!
//! This crate provides a comprehensive editor UI for scene creation, entity management,
//! and component editing. The editor is feature-gated and only included in development builds.

pub mod dpi_utils;
pub mod editor_state;
pub mod panel_state;
pub mod panels;
pub mod safe_imgui_renderer;
pub mod scene_operations;
pub mod shared_state;

pub use editor_state::{EditorState, SceneOperation};
