//! Game viewport panel
//!
//! Displays the rendered game view within the editor.

use crate::panel_state::{PanelId, PanelManager};
use crate::panels::detachable::detachable_window;
use crate::shared_state::EditorSharedState;

/// Render the viewport panel with texture
pub fn render_viewport_panel(
    ui: &imgui::Ui,
    texture_id: imgui::TextureId,
    render_target: &engine::graphics::render_target::RenderTarget,
    _shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
) {
    let panel_id = PanelId("viewport".to_string());

    detachable_window(ui, &panel_id, panel_manager, || {
        let available_size = ui.content_region_avail();
        tracing::debug!(
            "Rendering viewport panel: texture_id={:?}, available_size={:?}, render_target_size={:?}",
            texture_id, available_size, render_target.size
        );

        // Check if viewport needs resizing
        let new_size = (available_size[0] as u32, available_size[1] as u32);
        if new_size != render_target.size && new_size.0 > 0 && new_size.1 > 0 {
            tracing::debug!(
                "Viewport resize needed: {:?} -> {:?}",
                render_target.size,
                new_size
            );
            // Note: Actual resize is handled by the editor state on window resize
        }

        // Display the game render target with proper aspect ratio
        imgui::Image::new(texture_id, available_size).build(ui);
    });
}
