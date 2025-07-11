//! Component inspector panel
//!
//! Displays and allows editing of components for the selected entity.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use engine::prelude::{Camera, Material, MeshId, Parent, Transform};
use imgui::*;
use tracing::debug;

/// Render the component inspector panel
pub fn render_inspector_panel(
    ui: &imgui::Ui,
    shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
) {
    let panel_id = PanelId("inspector".to_string());

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
        .size([280.0, 300.0], Condition::FirstUseEver)
        .position([1000.0, 50.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
            if let Some(entity) = shared_state.selected_entity() {
                ui.text(format!("Entity: {entity:?}"));
                ui.separator();

                // Access world data through shared state
                shared_state.with_world_read(|world| {
                    // Transform component
                    if let Ok(transform) = world.get::<&Transform>(entity) {
                        if ui.collapsing_header("Transform", TreeNodeFlags::DEFAULT_OPEN) {
                            render_transform_inspector(ui, &transform);
                        }
                    }

                    // Parent component
                    if let Ok(parent) = world.get::<&Parent>(entity) {
                        ui.text("Parent:");
                        ui.same_line();
                        ui.text(format!("{:?}", parent.0));
                    }

                    // Camera component
                    if let Ok(camera) = world.get::<&Camera>(entity) {
                        if ui.collapsing_header("Camera", TreeNodeFlags::DEFAULT_OPEN) {
                            render_camera_inspector(ui, &camera);
                        }
                    }

                    // Material component
                    if let Ok(material) = world.get::<&Material>(entity) {
                        if ui.collapsing_header("Material", TreeNodeFlags::DEFAULT_OPEN) {
                            render_material_inspector(ui, &material);
                        }
                    }

                    // MeshId component
                    if let Ok(mesh_id) = world.get::<&MeshId>(entity) {
                        if ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
                            render_mesh_inspector(ui, &mesh_id);
                        }
                    }
                });

                // Add component button
                ui.separator();
                if ui.button("Add Component") {
                    debug!("Add component requested for entity {:?}", entity);
                    // TODO: Implement component addition
                }
            } else {
                ui.text("No entity selected");
                ui.text("Select an entity from the hierarchy to inspect its components.");
            }

            // Panel position and size are now managed by ImGui's docking system
        });
}

/// Render transform component viewer (read-only for now)
fn render_transform_inspector(ui: &imgui::Ui, transform: &Transform) {
    ui.text("Position:");
    ui.same_line();
    ui.text(format!(
        "X: {:.2}  Y: {:.2}  Z: {:.2}",
        transform.position.x, transform.position.y, transform.position.z
    ));

    // Convert quaternion to euler angles for display
    let (x, y, z) = transform.rotation.to_euler(glam::EulerRot::XYZ);
    ui.text("Rotation:");
    ui.same_line();
    ui.text(format!(
        "X: {:.1}°  Y: {:.1}°  Z: {:.1}°",
        x.to_degrees(),
        y.to_degrees(),
        z.to_degrees()
    ));

    ui.text("Scale:");
    ui.same_line();
    ui.text(format!(
        "X: {:.2}  Y: {:.2}  Z: {:.2}",
        transform.scale.x, transform.scale.y, transform.scale.z
    ));

    // TODO: Implement actual editing once we have proper mutable access
    ui.text_disabled("(Read-only view)");
}

/// Render camera component viewer
fn render_camera_inspector(ui: &imgui::Ui, camera: &Camera) {
    match camera.projection_mode {
        engine::prelude::ProjectionMode::Perspective => {
            ui.text("Type: Perspective");
            ui.text(format!("FOV: {:.1}°", camera.fov_y_radians.to_degrees()));
        }
        engine::prelude::ProjectionMode::Orthographic { height } => {
            ui.text("Type: Orthographic");
            ui.text(format!("Height: {height:.1}"));
        }
    }
    ui.text(format!("Aspect: {:.2}", camera.aspect_ratio));
    ui.text(format!("Near: {:.3}", camera.z_near));
    ui.text(format!("Far: {:.1}", camera.z_far));
}

/// Render material component viewer
fn render_material_inspector(ui: &imgui::Ui, material: &Material) {
    ui.text(format!(
        "Color: [{:.2}, {:.2}, {:.2}, {:.2}]",
        material.color[0], material.color[1], material.color[2], material.color[3]
    ));
}

/// Render mesh component viewer
fn render_mesh_inspector(ui: &imgui::Ui, mesh_id: &MeshId) {
    ui.text(format!("Mesh ID: {}", mesh_id.0));
}
