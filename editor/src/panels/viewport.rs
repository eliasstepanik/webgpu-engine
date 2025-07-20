//! Game viewport panel
//!
//! Displays the rendered game view within the editor.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use imgui::*;

/// Render the viewport panel with texture
/// Returns the desired viewport size if it has changed
pub fn render_viewport_panel(
    ui: &imgui::Ui,
    texture_id: imgui::TextureId,
    render_target: &engine::graphics::render_target::RenderTarget,
    _shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
) -> Option<(u32, u32)> {
    let panel_id = PanelId("viewport".to_string());

    // Get panel info
    let (panel_title, is_visible) = {
        match panel_manager.get_panel(&panel_id) {
            Some(panel) => (panel.title.clone(), panel.is_visible),
            None => return None,
        }
    };

    if !is_visible {
        return None;
    }

    let window_name = format!("{}##{}", panel_title, panel_id.0);
    let mut resize_needed = None;

    ui.window(&window_name)
        .size([800.0, 600.0], Condition::FirstUseEver)
        .position([100.0, 100.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
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
            resize_needed = Some(new_size);
        }

        // Display the game render target with proper aspect ratio
        imgui::Image::new(texture_id, available_size).build(ui);

        // Panel position and size are now managed by ImGui's docking system
    });
    
    resize_needed
}
