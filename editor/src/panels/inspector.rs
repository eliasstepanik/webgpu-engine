//! Component inspector panel
//!
//! Displays and allows editing of components for the selected entity.

use crate::docking::check_dock_zones;
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
    window_size: (f32, f32),
) {
    let panel_id = PanelId("inspector".to_string());

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

            // Update panel position and size if window was moved/resized
            if let Some(panel) = panel_manager.get_panel_mut(&panel_id) {
                let new_pos = ui.window_pos();
                let new_size = ui.window_size();
                
                // Track drag state
                if ui.is_window_hovered() && ui.is_mouse_dragging(MouseButton::Left) {
                    if !panel.is_dragging {
                        panel.start_drag();
                    }
                    
                    // Check for docking zones while dragging
                    if let Some(docked_state) = check_dock_zones(
                        (new_pos[0], new_pos[1]),
                        (new_size[0], new_size[1]),
                        window_size,
                        None,
                    ) {
                        // Visual feedback could be added here
                        debug!(panel = "inspector", edge = ?docked_state.edge, "Panel in dock zone");
                    }
                } else if panel.is_dragging && !ui.is_mouse_down(MouseButton::Left) {
                    // Mouse released - check if we should dock
                    panel.stop_drag();
                    
                    if let Some(docked_state) = check_dock_zones(
                        (new_pos[0], new_pos[1]),
                        (new_size[0], new_size[1]),
                        window_size,
                        None,
                    ) {
                        panel.dock(docked_state);
                    }
                }
                
                // Update position and size
                if !panel.is_dragging {
                    panel.position = (new_pos[0], new_pos[1]);
                }
                panel.size = (new_size[0], new_size[1]);
                
                // Check if we should undock (panel dragged away from edge)
                if panel.is_dragging && panel.docked.is_some() {
                    panel.check_undock((new_pos[0], new_pos[1]), window_size, 50.0);
                }
            }
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
