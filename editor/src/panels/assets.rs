//! Asset browser panel
//!
//! Displays available assets like scenes, meshes, and materials.

use engine::core::entity::World;

#[allow(unused_variables)]
/// Render the assets panel
pub fn render_assets_panel(ui: &imgui::Ui, world: &mut World) {
    ui.window("Assets").resizable(true).build(|| {
        // TODO: Implement asset browser
        ui.text("Asset browser coming soon...");
    });
}
