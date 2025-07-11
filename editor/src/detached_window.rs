//! Detached window management for editor panels
//!
//! This module handles creating and managing separate OS windows for detached panels.
//! Since imgui-rs 0.12 only supports one context at a time, detached windows share
//! the main imgui context and manage their own render targets.

use crate::panel_state::PanelId;
use std::sync::Arc;
use tracing::{debug, info};
use winit::window::Window;

/// Represents a detached editor window that shares the main imgui context
pub struct DetachedWindow {
    /// The panel ID this window is displaying
    pub panel_id: PanelId,
    /// Window reference
    pub window: Arc<Window>,
    /// Current window size for tracking changes
    pub size: (u32, u32),
}

impl DetachedWindow {
    /// Create a new detached window for a panel
    pub fn new(panel_id: PanelId, window: Arc<Window>) -> Self {
        info!("Creating detached window for panel: {:?}", panel_id);

        let window_size = window.inner_size();

        Self {
            panel_id,
            window,
            size: (window_size.width, window_size.height),
        }
    }

    /// Handle resize
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = (new_size.width, new_size.height);

        debug!(
            "Detached window resized: {}x{}",
            new_size.width, new_size.height
        );
    }

    /// Prepare for window shutdown
    pub fn prepare_shutdown(&mut self) {
        info!(
            "Preparing detached window for shutdown: {:?}",
            self.panel_id
        );

        // ImGui contexts will be automatically dropped when this struct is dropped
        // wgpu resources will be cleaned up automatically as well
        // No manual cleanup needed for current implementation
    }

    /// Get the window title for identification
    pub fn get_title(&self) -> String {
        format!("Detached Panel - {:?}", self.panel_id.0)
    }
}
