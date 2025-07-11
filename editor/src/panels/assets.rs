//! Asset browser panel
//!
//! Displays available assets like scenes, meshes, and materials.

use crate::panel_state::{PanelId, PanelManager};
use crate::panels::detachable::detachable_window;
use crate::shared_state::EditorSharedState;

#[allow(unused_variables)]
/// Render the assets panel
pub fn render_assets_panel(
    ui: &imgui::Ui,
    _shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
) {
    let panel_id = PanelId("assets".to_string());

    detachable_window(ui, &panel_id, panel_manager, || {
        // TODO: Implement asset browser
        ui.text("Asset browser coming soon...");
    });
}
