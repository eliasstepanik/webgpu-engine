//! Component inspector panel
//!
//! Displays and allows editing of components for the selected entity.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use engine::prelude::{
    Camera, Material, MeshId, Name, Parent, ProjectionMode, Quat, ScriptProperties, Transform, Vec3,
};
use engine::scripting::property_types::PropertyValue;
use engine::scripting::ScriptRef;
use imgui::*;
use std::collections::HashMap;
use tracing::{debug, warn};

/// State for tracking euler angles per entity to avoid recalculation
static mut INSPECTOR_STATE: Option<InspectorState> = None;

#[derive(Default)]
struct InspectorState {
    euler_angles: HashMap<hecs::Entity, [f32; 3]>,
    component_filter: String,
    show_add_component_popup: bool,
}

#[allow(static_mut_refs)]
fn get_inspector_state() -> &'static mut InspectorState {
    unsafe {
        if INSPECTOR_STATE.is_none() {
            INSPECTOR_STATE = Some(InspectorState::default());
        }
        INSPECTOR_STATE.as_mut().unwrap()
    }
}

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

                // Check which components exist first
                let (has_name, has_transform, has_camera, has_material, has_mesh, has_script, _has_script_properties) = shared_state.with_world_read(|world| {
                    let components = (
                        world.get::<Name>(entity).is_ok(),
                        world.get::<Transform>(entity).is_ok(),
                        world.get::<Camera>(entity).is_ok(),
                        world.get::<Material>(entity).is_ok(),
                        world.get::<MeshId>(entity).is_ok(),
                        world.get::<ScriptRef>(entity).is_ok(),
                        world.get::<ScriptProperties>(entity).is_ok(),
                    );
                    debug!(entity = ?entity, has_name = components.0, has_transform = components.1, has_camera = components.2, has_material = components.3, has_mesh = components.4, has_script = components.5, has_script_properties = components.6, "Entity components");
                    components
                }).unwrap_or_else(|| {
                    warn!(entity = ?entity, "Failed to access world for entity");
                    (false, false, false, false, false, false, false)
                });



                // Name component
                if has_name {
                    if ui.collapsing_header("Name", TreeNodeFlags::DEFAULT_OPEN) {
                        shared_state.with_world_write(|world| {
                            if let Ok(mut name) = world.inner_mut().remove_one::<Name>(entity) {
                                let mut name_buffer = name.0.clone();
                                if ui.input_text("##name", &mut name_buffer).build() {
                                    name.0 = name_buffer;
                                    shared_state.mark_scene_modified();
                                    debug!(entity = ?entity, name = %name.0, "Modified entity name");
                                }
                                // Re-insert the component
                                let _ = world.insert_one(entity, name);
                            }
                        });
                    }
                } else {
                    warn!(entity = ?entity, "Entity missing Name component");
                }

                // Transform component
                if has_transform
                    && ui.collapsing_header("Transform", TreeNodeFlags::DEFAULT_OPEN) {
                        shared_state.with_world_write(|world| {
                            if let Ok(mut transform) = world.inner_mut().remove_one::<Transform>(entity) {
                                if render_editable_transform(ui, &mut transform, entity) {
                                    shared_state.mark_scene_modified();
                                    debug!(entity = ?entity, "Modified transform");
                                }
                                // Re-insert the component
                                let _ = world.insert_one(entity, transform);
                            }
                        });
                    }

                // Parent component (read-only)
                if let Some(parent_entity) = shared_state.with_world_read(|world| {
                    world.get::<Parent>(entity).map(|p| p.0).ok()
                }).flatten() {
                    ui.text("Parent:");
                    ui.same_line();
                    ui.text(format!("{parent_entity:?}"));
                }

                // Camera component
                if has_camera
                    && ui.collapsing_header("Camera", TreeNodeFlags::DEFAULT_OPEN) {
                        shared_state.with_world_write(|world| {
                            if let Ok(mut camera) = world.inner_mut().remove_one::<Camera>(entity) {
                                if render_editable_camera(ui, &mut camera) {
                                    shared_state.mark_scene_modified();
                                    debug!(entity = ?entity, "Modified camera");
                                }
                                // Re-insert the component
                                let _ = world.insert_one(entity, camera);
                            }
                        });
                    }

                // Material component
                if has_material
                    && ui.collapsing_header("Material", TreeNodeFlags::DEFAULT_OPEN) {
                        shared_state.with_world_write(|world| {
                            if let Ok(mut material) = world.inner_mut().remove_one::<Material>(entity) {
                                if render_editable_material(ui, &mut material) {
                                    shared_state.mark_scene_modified();
                                    debug!(entity = ?entity, "Modified material");
                                }
                                // Re-insert the component
                                let _ = world.insert_one(entity, material);
                            }
                        });
                    }

                // MeshId component
                if has_mesh
                    && ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
                        shared_state.with_world_write(|world| {
                            if let Ok(mut mesh_id) = world.inner_mut().remove_one::<MeshId>(entity) {
                                let mut mesh_name = mesh_id.0.clone();
                                let input_changed = ui.input_text("Mesh ID", &mut mesh_name)
                                    .hint("e.g. cube, sphere, or path/to/model.obj")
                                    .build();

                                // Add drop target
                                let mut drop_accepted = false;
                                if let Some(target) = ui.drag_drop_target() {
                                    // Visual feedback when hovering
                                    if ui.is_item_hovered() {
                                        ui.get_window_draw_list()
                                            .add_rect(
                                                ui.item_rect_min(),
                                                ui.item_rect_max(),
                                                [0.0, 1.0, 0.0, 0.5],
                                            )
                                            .build();
                                    }

                                    if target.accept_payload_empty("ASSET_FILE", DragDropFlags::empty()).is_some() {
                                        // Get dragged file from asset browser state
                                        if let Some(file_path) = crate::panels::assets::AssetBrowserState::take_dragged_file() {
                                            if file_path.ends_with(".obj") && crate::panels::assets::validate_asset_path(&file_path) {
                                                mesh_name = format!("game/assets/{file_path}");
                                                drop_accepted = true;
                                                debug!(entity = ?entity, "Accepted .obj drop: {}", mesh_name);
                                            } else {
                                                warn!("Invalid or unsafe asset path dropped: {}", file_path);
                                            }
                                        }
                                    }
                                    target.pop();
                                }

                                if input_changed || drop_accepted {
                                    mesh_id.0 = mesh_name;
                                    shared_state.mark_scene_modified();
                                    debug!(entity = ?entity, mesh = %mesh_id.0, "Modified mesh");
                                }
                                // Re-insert the component
                                let _ = world.insert_one(entity, mesh_id);
                            }
                        });
                    }

                // Script component
                if has_script
                    && ui.collapsing_header("Script", TreeNodeFlags::DEFAULT_OPEN) {
                        shared_state.with_world_write(|world| {
                            if let Ok(mut script) = world.inner_mut().remove_one::<ScriptRef>(entity) {
                                let mut script_name = script.name.clone();
                                let input_changed = ui.input_text("Script Name", &mut script_name)
                                    .hint("e.g. fly_camera, rotating_cube")
                                    .build();

                                // Add drop target
                                let mut drop_accepted = false;
                                if let Some(target) = ui.drag_drop_target() {
                                    // Visual feedback when hovering
                                    if ui.is_item_hovered() {
                                        ui.get_window_draw_list()
                                            .add_rect(
                                                ui.item_rect_min(),
                                                ui.item_rect_max(),
                                                [0.0, 1.0, 0.0, 0.5],
                                            )
                                            .build();
                                    }

                                    if target.accept_payload_empty("ASSET_FILE", DragDropFlags::empty()).is_some() {
                                        // Get dragged file from asset browser state
                                        if let Some(file_path) = crate::panels::assets::AssetBrowserState::take_dragged_file() {
                                            if file_path.ends_with(".rhai") {
                                                // Extract script name without extension and path
                                                if let Some(name) = std::path::Path::new(&file_path)
                                                    .file_stem()
                                                    .and_then(|n| n.to_str()) {
                                                    script_name = name.to_string();
                                                    drop_accepted = true;
                                                    debug!(entity = ?entity, "Accepted .rhai drop: {}", script_name);
                                                }
                                            }
                                        }
                                    }
                                    target.pop();
                                }

                                if input_changed || drop_accepted {
                                    script.name = script_name;
                                    shared_state.mark_scene_modified();
                                    debug!(entity = ?entity, script = %script.name, "Modified script");
                                }
                                // Re-insert the component
                                let _ = world.insert_one(entity, script);
                            }
                        });

                        // Script Properties
                        // Always check for properties when a script is present
                        if has_script {
                            ui.separator();
                            // Re-check if properties exist (they might have been added by the init system)
                            let has_props = shared_state.with_world_read(|world| {
                                world.get::<ScriptProperties>(entity).is_ok()
                            }).unwrap_or(false);
                            if has_props {
                                ui.text("Script Properties:");

                                shared_state.with_world_write(|world| {
                                    // Get the script name first before removing properties
                                    let script_name = world.get::<&ScriptRef>(entity)
                                        .map(|s| s.name.clone())
                                        .ok();
                                    if let Ok(mut properties) = world.inner_mut().remove_one::<ScriptProperties>(entity) {
                                        let mut properties_modified = false;
                                        // Update script name if not set
                                        if properties.script_name.is_none() && script_name.is_some() {
                                            properties.script_name = script_name;
                                            properties_modified = true;
                                        }

                                        // Render each property
                                        for (name, value) in properties.values.iter_mut() {
                                        let _id = ui.push_id(name);

                                        match value {
                                            PropertyValue::Float(f) => {
                                                ui.text(format!("{name}:"));
                                                ui.same_line();
                                                let old_val = *f;
                                                if Drag::new(format!("##{name}"))
                                                    .display_format("%.3f")
                                                    .speed(0.01)
                                                    .build(ui, f)
                                                {
                                                    warn!(
                                                        "\n💡💡💡 INSPECTOR CHANGED PROPERTY! 💡💡💡\n\
                                                        Entity: {:?}\n\
                                                        Property: {}\n\
                                                        Old Value: {}\n\
                                                        New Value: {}\n\
                                                        ================================",
                                                        entity, name, old_val, f
                                                    );
                                                    properties_modified = true;
                                                }
                                            }
                                            PropertyValue::Integer(i) => {
                                                ui.text(format!("{name}:"));
                                                ui.same_line();
                                                if Drag::new(format!("##{name}"))
                                                    .display_format("%d")
                                                    .speed(1.0)
                                                    .build(ui, i)
                                                {
                                                    properties_modified = true;
                                                }
                                            }
                                            PropertyValue::Boolean(b) => {
                                                if ui.checkbox(name, b) {
                                                    properties_modified = true;
                                                }
                                            }
                                            PropertyValue::String(s) => {
                                                ui.text(format!("{name}:"));
                                                ui.same_line();
                                                if ui.input_text(format!("##{name}"), s).build() {
                                                    properties_modified = true;
                                                }
                                            }
                                            PropertyValue::Vector3(v) => {
                                                ui.text(format!("{name}:"));

                                                let mut x = v[0];
                                                let mut y = v[1];
                                                let mut z = v[2];

                                                ui.text("X:");
                                                ui.same_line();
                                                ui.set_next_item_width(60.0);
                                                if Drag::new(format!("##{name}x"))
                                                    .display_format("%.3f")
                                                    .speed(0.01)
                                                    .build(ui, &mut x)
                                                {
                                                    v[0] = x;
                                                    properties_modified = true;
                                                }
                                                ui.same_line();
                                                ui.text("Y:");
                                                ui.same_line();
                                                ui.set_next_item_width(60.0);
                                                if Drag::new(format!("##{name}y"))
                                                    .display_format("%.3f")
                                                    .speed(0.01)
                                                    .build(ui, &mut y)
                                                {
                                                    v[1] = y;
                                                    properties_modified = true;
                                                }
                                                ui.same_line();
                                                ui.text("Z:");
                                                ui.same_line();
                                                ui.set_next_item_width(60.0);
                                                if Drag::new(format!("##{name}z"))
                                                    .display_format("%.3f")
                                                    .speed(0.01)
                                                    .build(ui, &mut z)
                                                {
                                                    v[2] = z;
                                                    properties_modified = true;
                                                }
                                            }
                                            PropertyValue::Color(c) => {
                                                ui.text(format!("{name}:"));
                                                ui.same_line();
                                                if ui.color_edit4(format!("##{name}"), c) {
                                                    properties_modified = true;
                                                }
                                            }
                                        }
                                    }

                                    if properties_modified {
                                        shared_state.mark_scene_modified();
                                        debug!(entity = ?entity, "Modified script properties");
                                    }

                                    // Re-insert the properties
                                    let _ = world.insert_one(entity, properties);
                                }
                            });
                            } else {
                                ui.text_disabled("Loading script properties...");
                            }
                        }
                    }

                // Add component button
                ui.separator();
                let state = get_inspector_state();
                if ui.button("Add Component") {
                    state.show_add_component_popup = true;
                    state.component_filter.clear();
                    debug!("Add component requested for entity {:?}", entity);
                }

                // Component addition popup
                if state.show_add_component_popup {
                    ui.open_popup("##add_component_popup");
                }

                if let Some(_token) = ui.modal_popup_config("##add_component_popup")
                    .resizable(false)
                    .movable(false)
                    .begin_popup()
                {
                    ui.text("Add Component");
                    ui.separator();

                    // Filter input
                    if ui.input_text("Filter", &mut state.component_filter)
                        .hint("Type to filter...")
                        .build()
                    {
                        // Filter changed
                    }

                    ui.separator();

                    // Component list
                    let filter = state.component_filter.to_lowercase();
                    let mut component_added = false;

                    // Check which components the entity already has
                    let has_transform = shared_state.with_world_read(|world| world.get::<Transform>(entity).is_ok()).unwrap_or(false);
                    let has_camera = shared_state.with_world_read(|world| world.get::<Camera>(entity).is_ok()).unwrap_or(false);
                    let has_material = shared_state.with_world_read(|world| world.get::<Material>(entity).is_ok()).unwrap_or(false);
                    let has_mesh = shared_state.with_world_read(|world| world.get::<MeshId>(entity).is_ok()).unwrap_or(false);
                    let has_name = shared_state.with_world_read(|world| world.get::<Name>(entity).is_ok()).unwrap_or(false);
                    let has_script = shared_state.with_world_read(|world| world.get::<ScriptRef>(entity).is_ok()).unwrap_or(false);

                    // Transform
                    if !has_transform && "transform".contains(&filter)
                        && ui.selectable("Transform") {
                            shared_state.with_world_write(|world| {
                                let _ = world.insert_one(entity, Transform::default());
                                let _ = world.insert_one(entity, engine::prelude::GlobalTransform::default());
                            });
                            shared_state.mark_scene_modified();
                            component_added = true;
                            debug!(entity = ?entity, "Added Transform component");
                        }

                    // Camera
                    if !has_camera && "camera".contains(&filter)
                        && ui.selectable("Camera") {
                            shared_state.with_world_write(|world| {
                                let _ = world.insert_one(entity, Camera::default());
                            });
                            shared_state.mark_scene_modified();
                            component_added = true;
                            debug!(entity = ?entity, "Added Camera component");
                        }

                    // Material
                    if !has_material && "material".contains(&filter)
                        && ui.selectable("Material") {
                            shared_state.with_world_write(|world| {
                                let _ = world.insert_one(entity, Material::default());
                            });
                            shared_state.mark_scene_modified();
                            component_added = true;
                            debug!(entity = ?entity, "Added Material component");
                        }

                    // MeshId
                    if !has_mesh && "mesh".contains(&filter)
                        && ui.selectable("Mesh (Cube)") {
                            shared_state.with_world_write(|world| {
                                let _ = world.insert_one(entity, MeshId("cube".to_string()));
                            });
                            shared_state.mark_scene_modified();
                            component_added = true;
                            debug!(entity = ?entity, "Added MeshId component");
                        }

                    // Name
                    if !has_name && "name".contains(&filter)
                        && ui.selectable("Name") {
                            shared_state.with_world_write(|world| {
                                let _ = world.insert_one(entity, Name::new("New Entity"));
                            });
                            shared_state.mark_scene_modified();
                            component_added = true;
                            debug!(entity = ?entity, "Added Name component");
                        }

                    // Script
                    if !has_script && "script".contains(&filter)
                        && ui.selectable("Script") {
                            shared_state.with_world_write(|world| {
                                let _ = world.insert_one(entity, ScriptRef::new("new_script"));
                            });
                            shared_state.mark_scene_modified();
                            component_added = true;
                            debug!(entity = ?entity, "Added Script component");
                        }

                    ui.separator();

                    if ui.button("Cancel") || component_added {
                        state.show_add_component_popup = false;
                        ui.close_current_popup();
                    }
                }
            } else {
                ui.text("No entity selected");
                ui.text("Select an entity from the hierarchy to inspect its components.");
            }

            // Panel position and size are now managed by ImGui's docking system
        });
}

/// Render editable transform component
fn render_editable_transform(
    ui: &imgui::Ui,
    transform: &mut Transform,
    entity: hecs::Entity,
) -> bool {
    let mut modified = false;
    let state = get_inspector_state();

    // Calculate available width and column layout
    let available_width = ui.content_region_avail()[0];
    let label_width = 20.0; // Width for "X:", "Y:", "Z:" labels
    let spacing = unsafe { ui.style().item_spacing[0] };
    let input_width = (available_width - label_width * 3.0 - spacing * 4.0) / 3.0;

    // Position
    ui.text("Position:");
    let mut pos_x = transform.position.x;
    let mut pos_y = transform.position.y;
    let mut pos_z = transform.position.z;

    // Position row with proper columns
    ui.text("X:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##pos_x")
        .speed(0.01)
        .display_format("%.3f")
        .build(ui, &mut pos_x)
    {
        transform.position.x = pos_x;
        modified = true;
    }
    ui.same_line();
    ui.text("Y:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##pos_y")
        .speed(0.01)
        .display_format("%.3f")
        .build(ui, &mut pos_y)
    {
        transform.position.y = pos_y;
        modified = true;
    }
    ui.same_line();
    ui.text("Z:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##pos_z")
        .speed(0.01)
        .display_format("%.3f")
        .build(ui, &mut pos_z)
    {
        transform.position.z = pos_z;
        modified = true;
    }

    // Rotation (using euler angles for editing)
    ui.text("Rotation (degrees):");

    // Get or calculate euler angles
    let mut euler_degrees = if let Some(cached) = state.euler_angles.get(&entity) {
        *cached
    } else {
        let (x, y, z) = transform.rotation.to_euler(glam::EulerRot::XYZ);
        [x.to_degrees(), y.to_degrees(), z.to_degrees()]
    };

    // Rotation row with proper columns
    ui.text("X:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##rot_x")
        .speed(0.5)
        .display_format("%.1f")
        .build(ui, &mut euler_degrees[0])
    {
        modified = true;
    }
    ui.same_line();
    ui.text("Y:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##rot_y")
        .speed(0.5)
        .display_format("%.1f")
        .build(ui, &mut euler_degrees[1])
    {
        modified = true;
    }
    ui.same_line();
    ui.text("Z:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##rot_z")
        .speed(0.5)
        .display_format("%.1f")
        .build(ui, &mut euler_degrees[2])
    {
        modified = true;
    }

    if modified {
        // Convert back to quaternion
        transform.rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            euler_degrees[0].to_radians(),
            euler_degrees[1].to_radians(),
            euler_degrees[2].to_radians(),
        );
        // Cache the euler angles
        state.euler_angles.insert(entity, euler_degrees);
    }

    // Scale
    ui.text("Scale:");
    let mut scale_x = transform.scale.x;
    let mut scale_y = transform.scale.y;
    let mut scale_z = transform.scale.z;

    // Scale row with proper columns
    ui.text("X:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##scale_x")
        .speed(0.01)
        .display_format("%.3f")
        .build(ui, &mut scale_x)
    {
        transform.scale.x = scale_x;
        modified = true;
    }
    ui.same_line();
    ui.text("Y:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##scale_y")
        .speed(0.01)
        .display_format("%.3f")
        .build(ui, &mut scale_y)
    {
        transform.scale.y = scale_y;
        modified = true;
    }
    ui.same_line();
    ui.text("Z:");
    ui.same_line();
    ui.set_next_item_width(input_width);
    if Drag::new("##scale_z")
        .speed(0.01)
        .display_format("%.3f")
        .build(ui, &mut scale_z)
    {
        transform.scale.z = scale_z;
        modified = true;
    }

    // Reset buttons - calculate button widths
    let button_count = 3.0;
    let button_width = (available_width - spacing * (button_count - 1.0)) / button_count;

    ui.set_next_item_width(button_width);
    if ui.button("Reset Position") {
        transform.position = Vec3::ZERO;
        modified = true;
    }
    ui.same_line();
    ui.set_next_item_width(button_width);
    if ui.button("Reset Rotation") {
        transform.rotation = Quat::IDENTITY;
        state.euler_angles.insert(entity, [0.0, 0.0, 0.0]);
        modified = true;
    }
    ui.same_line();
    ui.set_next_item_width(button_width);
    if ui.button("Reset Scale") {
        transform.scale = Vec3::ONE;
        modified = true;
    }

    modified
}

/// Render editable camera component
fn render_editable_camera(ui: &imgui::Ui, camera: &mut Camera) -> bool {
    let mut modified = false;

    // Projection mode selector
    let mut is_perspective = matches!(camera.projection_mode, ProjectionMode::Perspective);
    if ui.checkbox("Perspective", &mut is_perspective) {
        if is_perspective && !matches!(camera.projection_mode, ProjectionMode::Perspective) {
            camera.projection_mode = ProjectionMode::Perspective;
            modified = true;
        } else if !is_perspective && matches!(camera.projection_mode, ProjectionMode::Perspective) {
            camera.projection_mode = ProjectionMode::Orthographic { height: 10.0 };
            modified = true;
        }
    }

    match &mut camera.projection_mode {
        ProjectionMode::Perspective => {
            let mut fov_degrees = camera.fov_y_radians.to_degrees();
            ui.text("Field of View:");
            if ui.slider("##fov", 30.0, 120.0, &mut fov_degrees) {
                camera.fov_y_radians = fov_degrees.to_radians();
                modified = true;
            }
        }
        ProjectionMode::Orthographic { height } => {
            ui.text("Orthographic Height:");
            if Drag::new("##ortho_height")
                .speed(0.1)
                .range(0.1, 100.0)
                .display_format("%.1f")
                .build(ui, height)
            {
                modified = true;
            }
        }
    }

    // Near/Far planes
    ui.text("Near Plane:");
    if Drag::new("##near")
        .speed(0.01)
        .range(0.001, camera.z_far - 0.001)
        .display_format("%.3f")
        .build(ui, &mut camera.z_near)
    {
        modified = true;
    }

    ui.text("Far Plane:");
    if Drag::new("##far")
        .speed(1.0)
        .range(camera.z_near + 0.001, 10000.0)
        .display_format("%.1f")
        .build(ui, &mut camera.z_far)
    {
        modified = true;
    }

    // Aspect ratio (read-only, set by viewport)
    ui.text(format!("Aspect Ratio: {:.3}", camera.aspect_ratio));
    ui.text_disabled("(Set by viewport size)");

    modified
}

/// Render editable material component
fn render_editable_material(ui: &imgui::Ui, material: &mut Material) -> bool {
    let mut modified = false;

    ui.text("Color:");
    if ui.color_edit4("##color", &mut material.color) {
        modified = true;
    }

    // Quick color presets - calculate button widths
    ui.text("Presets:");
    let available_width = ui.content_region_avail()[0];
    let spacing = unsafe { ui.style().item_spacing[0] };
    let button_count = 4.0;
    let button_width = (available_width - spacing * (button_count - 1.0)) / button_count;

    ui.set_next_item_width(button_width);
    if ui.button("White") {
        material.color = [1.0, 1.0, 1.0, 1.0];
        modified = true;
    }
    ui.same_line();
    ui.set_next_item_width(button_width);
    if ui.button("Red") {
        material.color = [1.0, 0.0, 0.0, 1.0];
        modified = true;
    }
    ui.same_line();
    ui.set_next_item_width(button_width);
    if ui.button("Green") {
        material.color = [0.0, 1.0, 0.0, 1.0];
        modified = true;
    }
    ui.same_line();
    ui.set_next_item_width(button_width);
    if ui.button("Blue") {
        material.color = [0.0, 0.0, 1.0, 1.0];
        modified = true;
    }

    modified
}

/// Render mesh component viewer
#[allow(dead_code)]
fn render_mesh_inspector(ui: &imgui::Ui, mesh_id: &MeshId) {
    ui.text(format!("Mesh ID: {}", mesh_id.0));
}
