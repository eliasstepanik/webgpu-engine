//! Asset browser panel
//!
//! Displays available assets like scenes, meshes, and materials.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use imgui::*;

#[allow(unused_variables)]
/// Render the assets panel
pub fn render_assets_panel(
    ui: &imgui::Ui,
    _shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
) {
    let panel_id = PanelId("assets".to_string());

    // Get panel info
    let (panel_title, is_visible) = {
        match panel_manager.get_panel(&panel_id) {
            Some(panel) => (panel.title.clone(), panel.is_visible),
            None => return,
        }
    };

    if !is_visible {
        return;
    }

    let window_name = format!("{}##{}", panel_title, panel_id.0);

    ui.window(&window_name)
        .size([800.0, 328.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
            // TODO: Implement asset browser
            ui.text("Asset browser coming soon...");

            // Update panel position and size if window was moved/resized
            if let Some(panel) = panel_manager.get_panel_mut(&panel_id) {
                let new_pos = ui.window_pos();
                let new_size = ui.window_size();

                // Update position and size for layout saving
                panel.position = (new_pos[0], new_pos[1]);
                panel.size = (new_size[0], new_size[1]);
            }
        });
}
