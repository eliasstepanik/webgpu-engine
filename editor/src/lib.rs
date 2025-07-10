//! ImGui-based scene editor for WebGPU engine
//!
//! This crate provides a comprehensive editor UI for scene creation, entity management,
//! and component editing. The editor is feature-gated and only included in development builds.

pub mod detached_window;
pub mod detached_window_manager;
pub mod editor_state;
pub mod panel_state;
pub mod panels;
pub mod performance_monitor;
pub mod scene_operations;
pub mod shared_state;

#[cfg(feature = "viewport")]
pub mod viewport_backend;
#[cfg(feature = "viewport")]
pub mod viewport_renderer;
#[cfg(feature = "viewport")]
pub mod enhanced_viewport_renderer;
#[cfg(feature = "viewport")]
pub mod viewport_renderer_backend;
#[cfg(feature = "viewport")]
pub mod test_viewport_fork;
#[cfg(feature = "viewport")]
pub mod check_viewport_issue;

pub use editor_state::{EditorState, SceneOperation};
