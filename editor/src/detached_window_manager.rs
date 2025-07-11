//! Manager for all detached editor windows
//!
//! Coordinates the creation, rendering, and destruction of detached panel windows.

use crate::detached_window::DetachedWindow;
use crate::panel_state::{PanelId, PanelManager};
use engine::graphics::context::RenderContext;
use engine::windowing::WindowManager;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};
use wgpu::TextureFormat;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

/// Manages all detached editor windows
pub struct DetachedWindowManager {
    /// Map of window ID to detached window
    windows: HashMap<WindowId, DetachedWindow>,
    /// Render context reference
    #[allow(dead_code)]
    render_context: Arc<RenderContext>,
    /// Surface format for new windows
    #[allow(dead_code)]
    surface_format: TextureFormat,
}

impl DetachedWindowManager {
    /// Create a new detached window manager
    pub fn new(render_context: Arc<RenderContext>, surface_format: TextureFormat) -> Self {
        Self {
            windows: HashMap::new(),
            render_context,
            surface_format,
        }
    }

    /// Process pending detach requests from the panel manager
    /// Note: Currently disabled due to imgui-rs 0.12 single-context limitation
    pub fn process_detach_requests(
        &mut self,
        panel_manager: &mut PanelManager,
        _window_manager: &mut WindowManager,
        _event_loop: &ActiveEventLoop,
    ) {
        let pending = panel_manager.take_pending_detach();

        // Clear pending requests but don't process them
        if !pending.is_empty() {
            warn!(
                "Ignoring {} detach requests due to imgui-rs 0.12 limitations",
                pending.len()
            );
        }

        // Auto-reattach any panels that might be marked as detached
        for panel_id in pending {
            if let Some(panel) = panel_manager.get_panel_mut(&panel_id) {
                if panel.is_detached {
                    panel.attach();
                    info!("Auto-reattaching panel due to limitations: {:?}", panel_id);
                }
            }
        }
    }

    /// Process pending attach requests
    pub fn process_attach_requests(
        &mut self,
        panel_manager: &mut PanelManager,
        window_manager: &mut WindowManager,
    ) {
        let pending = panel_manager.take_pending_attach();

        for panel_id in pending {
            if let Some(panel) = panel_manager.get_panel_mut(&panel_id) {
                if let Some(window_id) = panel.window_id {
                    // Remove the detached window
                    if self.windows.remove(&window_id).is_some() {
                        // Destroy the OS window
                        if let Err(e) = window_manager.destroy_window(window_id) {
                            warn!("Failed to destroy window: {}", e);
                        }

                        // Update panel state
                        panel.attach();

                        info!("Reattached panel: {:?}", panel_id);
                    }
                }
            }
        }
    }

    /// Get window information for rendering
    pub fn get_detached_window(&self, window_id: WindowId) -> Option<&DetachedWindow> {
        self.windows.get(&window_id)
    }

    /// Handle window resize
    pub fn resize_window(&mut self, window_id: WindowId, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.resize(new_size);
        }
    }

    /// Handle window close
    pub fn handle_window_close(
        &mut self,
        window_id: WindowId,
        panel_manager: &mut PanelManager,
        window_manager: &mut WindowManager,
    ) {
        // Find which panel was in this window
        if let Some(mut detached) = self.windows.remove(&window_id) {
            info!(
                "Cleaning up detached window for panel: {:?}",
                detached.panel_id
            );

            // Prepare the window for shutdown
            detached.prepare_shutdown();

            // Reattach the panel
            if let Some(panel) = panel_manager.get_panel_mut(&detached.panel_id) {
                panel.attach();
                info!(
                    "Reattached panel {:?} due to window close",
                    detached.panel_id
                );
            }

            // Clean up the window in the window manager
            if let Err(e) = window_manager.destroy_window(window_id) {
                warn!("Failed to destroy window during cleanup: {}", e);
            }
        }

        // Also handle any other panels that might have been in this window
        panel_manager.handle_window_close(window_id);
    }

    /// Get all window IDs managed by this manager
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }

    /// Check if a window is managed by this manager
    pub fn has_window(&self, window_id: WindowId) -> bool {
        self.windows.contains_key(&window_id)
    }

    /// Get the panel ID for a window
    pub fn get_panel_id(&self, window_id: WindowId) -> Option<&PanelId> {
        self.windows.get(&window_id).map(|w| &w.panel_id)
    }

    /// Clean up all detached windows during shutdown
    pub fn shutdown_all_windows(
        &mut self,
        panel_manager: &mut PanelManager,
        window_manager: &mut WindowManager,
    ) {
        info!("Shutting down all detached windows");

        let window_ids: Vec<WindowId> = self.windows.keys().copied().collect();

        for window_id in window_ids {
            if let Some(mut detached) = self.windows.remove(&window_id) {
                info!(
                    "Shutting down detached window for panel: {:?}",
                    detached.panel_id
                );

                // Prepare the window for shutdown
                detached.prepare_shutdown();

                // Reattach the panel
                if let Some(panel) = panel_manager.get_panel_mut(&detached.panel_id) {
                    panel.attach();
                }

                // Clean up the window in the window manager
                if let Err(e) = window_manager.destroy_window(window_id) {
                    warn!("Failed to destroy window during shutdown: {}", e);
                }
            }
        }

        // Clear the windows map
        self.windows.clear();
        info!("All detached windows have been shut down");
    }

    /// Get the number of active detached windows
    pub fn active_window_count(&self) -> usize {
        self.windows.len()
    }

    /// Check if any windows are currently detached
    pub fn has_active_windows(&self) -> bool {
        !self.windows.is_empty()
    }
}
