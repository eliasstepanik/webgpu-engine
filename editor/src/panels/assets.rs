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
        .size([800.0, 200.0], Condition::FirstUseEver)
        .position([100.0, 500.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
            // TODO: Implement asset browser
            ui.text("Asset browser coming soon...");

            // Panel position and size are now managed by ImGui's docking system
        });
}
