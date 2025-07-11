//! Game viewport panel
//!
//! Displays the rendered game view within the editor.

use crate::docking::check_dock_zones;
use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use imgui::*;
use tracing::debug;

/// Render the viewport panel with texture
pub fn render_viewport_panel(
    ui: &imgui::Ui,
    texture_id: imgui::TextureId,
    render_target: &engine::graphics::render_target::RenderTarget,
    _shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    window_size: (f32, f32),
) {
    let panel_id = PanelId("viewport".to_string());

    // Get panel info
    let (panel_title, panel_position, panel_size, is_visible) = {
        match panel_manager.get_panel(&panel_id) {
            Some(panel) => {
                let pos = panel.calculate_docked_position(window_size);
                (panel.title.clone(), pos, panel.size, panel.is_visible)
            }
            None => return,
        }
    };

    if !is_visible {
        return;
    }

    let window_name = format!("{}##{}", panel_title, panel_id.0);

    ui.window(&window_name)
        .position(
            [panel_position.0, panel_position.1],
            Condition::FirstUseEver,
        )
        .size([panel_size.0, panel_size.1], Condition::FirstUseEver)
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
            // Note: Actual resize is handled by the editor state on window resize
        }

        // Display the game render target with proper aspect ratio
        imgui::Image::new(texture_id, available_size).build(ui);

        // Update panel position and size if window was moved/resized
        if let Some(panel) = panel_manager.get_panel_mut(&panel_id) {
            let new_pos = ui.window_pos();
            let new_size = ui.window_size();
            
            // Check if window is being moved (position changed)
            let position_changed = (new_pos[0] - panel.position.0).abs() > 0.1 
                || (new_pos[1] - panel.position.1).abs() > 0.1;
            
            // Track drag state based on position changes and mouse state
            if position_changed && ui.is_mouse_down(MouseButton::Left) {
                if !panel.is_dragging {
                    panel.start_drag();
                    debug!(panel = "viewport", "Started dragging");
                }
                
                // Check for docking zones while dragging
                if let Some(docked_state) = check_dock_zones(
                    (new_pos[0], new_pos[1]),
                    (new_size[0], new_size[1]),
                    window_size,
                    None,
                ) {
                    // Visual feedback could be added here
                    debug!(panel = "viewport", edge = ?docked_state.edge, "Panel in dock zone");
                }
            } else if panel.is_dragging && !ui.is_mouse_down(MouseButton::Left) {
                // Mouse released - check if we should dock
                debug!(panel = "viewport", pos = ?(new_pos[0], new_pos[1]), "Checking for docking on mouse release");
                panel.stop_drag();
                
                if let Some(docked_state) = check_dock_zones(
                    (new_pos[0], new_pos[1]),
                    (new_size[0], new_size[1]),
                    window_size,
                    None,
                ) {
                    debug!(panel = "viewport", edge = ?docked_state.edge, "Docking panel");
                    panel.dock(docked_state);
                }
            }
            
            // Update position and size
            panel.position = (new_pos[0], new_pos[1]);
            panel.size = (new_size[0], new_size[1]);
            
            // Check if we should undock (panel dragged away from edge)
            if panel.is_dragging && panel.docked.is_some() {
                panel.check_undock((new_pos[0], new_pos[1]), window_size, 50.0);
            }
        }
    });
}
