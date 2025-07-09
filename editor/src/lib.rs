//! ImGui-based scene editor for WebGPU engine
//!
//! This crate provides a comprehensive editor UI for scene creation, entity management,
//! and component editing. The editor is feature-gated and only included in development builds.

pub mod editor_state;
pub mod panels;

pub use editor_state::EditorState;
