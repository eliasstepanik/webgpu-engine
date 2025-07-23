//! Editor UI panels
//!
//! This module contains all the individual UI panels that make up the editor,
//! including the hierarchy, inspector, viewport, and asset browser.

pub mod assets;
pub mod hierarchy;
pub mod inspector;
pub mod viewport;

pub use assets::render_assets_panel;
pub use hierarchy::render_hierarchy_panel;
pub use inspector::render_inspector_panel;
pub use viewport::{render_viewport_panel, PerformanceMetrics};
