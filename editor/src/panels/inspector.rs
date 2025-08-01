//! Component inspector panel
//!
//! Displays and allows editing of components for the selected entity.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use engine::component_system::ComponentRegistryExt;
use engine::prelude::{Camera, Material, MeshId, Name, Parent, ScriptProperties, Transform};
use engine::profile_zone;
use engine::scripting::property_types::PropertyValue;
use engine::scripting::ScriptRef;
use imgui::*;
use tracing::{debug, warn};

/// State for tracking euler angles per entity to avoid recalculation
static mut INSPECTOR_STATE: Option<InspectorState> = None;

#[derive(Default)]
struct InspectorState {
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
    profile_zone!("render_inspector_panel");

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
            // Create a child window that fills the entire inspector and acts as a drop target
            let available_size = ui.content_region_avail();

            // First, create a background child that handles drops
            ui.child_window("##inspector_drop_bg")
                .size(available_size)
                .flags(WindowFlags::NO_SCROLLBAR | WindowFlags::NO_SCROLL_WITH_MOUSE | WindowFlags::NO_INPUTS)
                .build(|| {
                    // Make the entire child window a drop target
                    ui.invisible_button("##drop_area", ui.content_region_avail());

                    if let Some(target) = ui.drag_drop_target() {
                        // Visual feedback when hovering
                        if ui.is_item_hovered() {
                            ui.get_window_draw_list()
                                .add_rect(
                                    [ui.window_pos()[0], ui.window_pos()[1]],
                                    [ui.window_pos()[0] + ui.window_size()[0], ui.window_pos()[1] + ui.window_size()[1]],
                                    [0.0, 1.0, 0.0, 0.2],
                                )
                                .filled(true)
                                .build();
                        }

                        if target.accept_payload_empty("ASSET_FILE", DragDropFlags::empty()).is_some() {
                            if let Some(entity) = shared_state.selected_entity() {
                                    // Get dragged file from asset browser state
                                    if let Some(file_path) = crate::panels::assets::AssetBrowserState::take_dragged_file() {
                                        if file_path.ends_with(".rhai") {
                                            // Handle script drop
                                            if let Some(name) = std::path::Path::new(&file_path)
                                                .file_stem()
                                                .and_then(|n| n.to_str()) {

                                                // Check if entity already has a script
                                                let has_script = shared_state.with_world_read(|world| {
                                                    world.get::<ScriptRef>(entity).is_ok()
                                                }).unwrap_or(false);

                                                if has_script {
                                                    // Update existing script
                                                    shared_state.with_world_write(|world| {
                                                        if let Ok(mut script) = world.inner_mut().remove_one::<ScriptRef>(entity) {
                                                            script.name = name.to_string();
                                                            debug!(entity = ?entity, script = %name, "Updated script via inspector drop");
                                                            let _ = world.insert_one(entity, script);
                                                        }
                                                    });
                                                } else {
                                                    // Add new script component
                                                    shared_state.with_world_write(|world| {
                                                        let _ = world.insert_one(entity, ScriptRef::new(name));
                                                        debug!(entity = ?entity, script = %name, "Added script via inspector drop");
                                                    });
                                                }
                                                shared_state.mark_scene_modified();
                                            }
                                        } else if file_path.ends_with(".obj") {
                                            // Handle mesh drop
                                            let mesh_path = format!("game/assets/{file_path}");

                                            // Check if entity already has a mesh
                                            let has_mesh = shared_state.with_world_read(|world| {
                                                world.get::<MeshId>(entity).is_ok()
                                            }).unwrap_or(false);

                                            if has_mesh {
                                                // Update existing mesh
                                                shared_state.with_world_write(|world| {
                                                    if let Ok(mut mesh) = world.inner_mut().remove_one::<MeshId>(entity) {
                                                        mesh.0 = mesh_path.clone();
                                                        debug!(entity = ?entity, mesh = %mesh_path, "Updated mesh via inspector drop");
                                                        let _ = world.insert_one(entity, mesh);
                                                    }
                                                });
                                            } else {
                                                // Add new mesh component
                                                shared_state.with_world_write(|world| {
                                                    let _ = world.insert_one(entity, MeshId(mesh_path.clone()));
                                                    debug!(entity = ?entity, mesh = %mesh_path, "Added mesh via inspector drop");
                                                });
                                            }
                                            shared_state.mark_scene_modified();
                                        }
                                }
                            }
                        }
                        target.pop();
                    }
                });

            // Now render the actual content on top in a separate child
            ui.set_cursor_pos([0.0, 0.0]); // Reset position to overlap
            ui.child_window("##inspector_content")
                .size(available_size)
                .build(|| {
                    if let Some(entity) = shared_state.selected_entity() {
                        ui.text(format!("Entity: {entity:?}"));
                        ui.separator();

                // Get all component types on this entity
                let component_types = shared_state.with_world_read(|world| {
                    world.inner().entity(entity)
                        .ok()
                        .map(|e| e.component_types().collect::<Vec<_>>())
                        .unwrap_or_default()
                }).unwrap_or_default();

                debug!(entity = ?entity, component_count = component_types.len(), "Entity components");

                // Render components dynamically using metadata
                let registry = shared_state.component_registry.clone();

                // Sort components by name for consistent ordering
                let mut components_with_metadata: Vec<_> = component_types.iter()
                    .filter_map(|&type_id| {
                        registry.get_metadata(type_id)
                            .map(|metadata| (type_id, metadata))
                    })
                    .collect();
                components_with_metadata.sort_by_key(|(_, metadata)| metadata.name);

                // Render each component that has metadata
                for (type_id, metadata) in components_with_metadata {
                    use std::any::TypeId;

                    // Skip certain internal components
                    if matches!(metadata.name,
                        "GlobalTransform" | "GlobalWorldTransform" |
                        "PreviousTransform" | "ParentData" | "CameraWorldPosition"
                    ) {
                        continue;
                    }

                    // Use type-specific rendering for components with custom UI
                    match type_id {
                        t if t == TypeId::of::<Name>() => {
                            render_component_with_metadata::<Name>(
                                ui, entity, "Name", shared_state, &registry
                            );
                        }
                        t if t == TypeId::of::<Transform>() => {
                            render_component_with_metadata::<Transform>(
                                ui, entity, "Transform", shared_state, &registry
                            );
                        }
                        t if t == TypeId::of::<Camera>() => {
                            render_component_with_metadata::<Camera>(
                                ui, entity, "Camera", shared_state, &registry
                            );
                        }
                        t if t == TypeId::of::<Material>() => {
                            render_component_with_metadata::<Material>(
                                ui, entity, "Material", shared_state, &registry
                            );
                        }
                        t if t == TypeId::of::<engine::physics::components::RigidBody>() => {
                            render_component_with_metadata::<engine::physics::components::RigidBody>(
                                ui, entity, "RigidBody", shared_state, &registry
                            );
                        }
                        t if t == TypeId::of::<engine::physics::components::Collider>() => {
                            render_component_with_metadata::<engine::physics::components::Collider>(
                                ui, entity, "Collider", shared_state, &registry
                            );
                        }
                        t if t == TypeId::of::<engine::physics::components::PhysicsVelocity>() => {
                            render_component_with_metadata::<engine::physics::components::PhysicsVelocity>(
                                ui, entity, "PhysicsVelocity", shared_state, &registry
                            );
                        }
                        t if t == TypeId::of::<engine::physics::components::PhysicsMass>() => {
                            render_component_with_metadata::<engine::physics::components::PhysicsMass>(
                                ui, entity, "PhysicsMass", shared_state, &registry
                            );
                        }
                        _ => {
                            // Skip components without UI metadata
                        }
                    }
                }

                // Special handling for Parent component (read-only)
                if let Some(parent_entity) = shared_state.with_world_read(|world| {
                    world.get::<Parent>(entity).map(|p| p.0).ok()
                }).flatten() {
                    ui.text("Parent:");
                    ui.same_line();
                    ui.text(format!("{parent_entity:?}"));
                }

                // MeshId component - special handling for drag/drop
                let has_mesh = shared_state.with_world_read(|world| world.get::<MeshId>(entity).is_ok()).unwrap_or(false);
                if has_mesh
                    && ui.collapsing_header("Mesh", TreeNodeFlags::DEFAULT_OPEN) {
                        let mut remove_component = false;

                        shared_state.with_world_write(|world| {
                            if let Ok(mut mesh_id) = world.inner_mut().remove_one::<MeshId>(entity) {
                                let mut mesh_name = mesh_id.0.clone();
                                let input_changed = ui.input_text("Mesh ID", &mut mesh_name)
                                    .hint("e.g. cube, sphere, or path/to/model.obj")
                                    .build();

                                // Add drop targe
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

                                // Remove component button
                                if ui.small_button("Remove Mesh") {
                                    remove_component = true;
                                    debug!(entity = ?entity, "Removed MeshId component");
                                } else {
                                    // Re-insert the component only if not removing
                                    let _ = world.insert_one(entity, mesh_id);
                                }
                            }
                        });

                        if remove_component {
                            shared_state.mark_scene_modified();
                        }
                    }

                // Script component - special handling for drag/drop and properties
                let has_script = shared_state.with_world_read(|world| world.get::<ScriptRef>(entity).is_ok()).unwrap_or(false);
                if has_script
                    && ui.collapsing_header("Script", TreeNodeFlags::DEFAULT_OPEN) {
                        let mut remove_component = false;

                        shared_state.with_world_write(|world| {
                            if let Ok(mut script) = world.inner_mut().remove_one::<ScriptRef>(entity) {
                                let mut script_name = script.name.clone();
                                let input_changed = ui.input_text("Script Name", &mut script_name)
                                    .hint("e.g. fly_camera, rotating_cube")
                                    .build();

                                // Add drop targe
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

                                // Remove component button
                                if ui.small_button("Remove Script") {
                                    remove_component = true;
                                    debug!(entity = ?entity, "Removed ScriptRef component");
                                } else {
                                    // Re-insert the component only if not removing
                                    let _ = world.insert_one(entity, script);
                                }
                            }
                        });

                        if remove_component {
                            shared_state.mark_scene_modified();
                        }

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
                                        // Update script name if not se
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
                                                        "\n💡💡💡 INSPECTOR CHANGED PROPERTY! 💡💡💡\n
                                                        Entity: {:?}\n
                                                        Property: {}\n
                                                        Old Value: {}\n
                                                        New Value: {}\n
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

                    // Filter inpu
                    if ui.input_text("Filter", &mut state.component_filter)
                        .hint("Type to filter...")
                        .build()
                    {
                        // Filter changed
                    }

                    ui.separator();

                    // Component list - dynamic from registry
                    let filter = state.component_filter.to_lowercase();
                    let mut component_added = false;

                    // Get existing component types for this entity
                    let existing_types = shared_state.with_world_read(|world| {
                        world.inner().entity(entity)
                            .ok()
                            .map(|e| e.component_types().collect::<Vec<_>>())
                            .unwrap_or_default()
                    }).unwrap_or_default();

                    // Get all registered components from the registry
                    let registry = shared_state.component_registry.clone();
                    let mut available_components: Vec<_> = registry.iter_metadata()
                        .filter(|metadata| {
                            // Filter by search term
                            metadata.name.to_lowercase().contains(&filter)
                                // Don't show components that are already on the entity
                                && !existing_types.contains(&metadata.type_id)
                                // Don't show internal components that shouldn't be manually added
                                && !matches!(metadata.name,
                                    "GlobalTransform" | "GlobalWorldTransform" |
                                    "PreviousTransform" | "Parent" | "ParentData" |
                                    "ScriptProperties" | "CameraWorldPosition"
                                )
                        })
                        .collect();

                    // Sort components alphabetically for better UX
                    available_components.sort_by_key(|m| m.name);

                    // Display each available component
                    for metadata in available_components {
                        if ui.selectable(metadata.name) {
                            shared_state.with_world_write(|world| {
                                // Use the add_default function from metadata
                                match (metadata.add_default)(world, entity) {
                                    Ok(_) => {
                                        debug!(entity = ?entity, component = metadata.name, "Added component via dynamic registry");

                                        // Special handling for Transform - also add GlobalTransform
                                        if metadata.name == "Transform" {
                                            let _ = world.insert_one(entity, engine::prelude::GlobalTransform::default());
                                        }

                                        shared_state.mark_scene_modified();
                                        component_added = true;
                                    }
                                    Err(e) => {
                                        warn!(entity = ?entity, component = metadata.name, error = %e, "Failed to add component");
                                    }
                                }
                            });
                            break; // Close popup after adding
                        }
                    }

                    ui.separator();

                    if ui.button("Cancel") || component_added {
                        state.show_add_component_popup = false;
                        ui.close_current_popup();
                    }
                }

                // Entity controls section
                ui.separator();
                ui.text("Entity Actions:");

                // Delete entity button
                if ui.button("Delete Entity") {
                    shared_state.with_world_write(|world| {
                        if world.despawn(entity).is_ok() {
                            debug!(entity = ?entity, "Deleted entity");
                            shared_state.set_selected_entity(None);
                            shared_state.mark_scene_modified();
                        }
                    });
                }

                ui.same_line();

                // Duplicate entity button
                if ui.button("Duplicate Entity") {
                    shared_state.with_world_write(|world| {
                        match duplicate_entity(world, entity, &registry) {
                            Ok(new_entity) => {
                                debug!(original = ?entity, new = ?new_entity, "Duplicated entity");
                                shared_state.set_selected_entity(Some(new_entity));
                                shared_state.mark_scene_modified();
                            }
                            Err(e) => {
                                warn!(entity = ?entity, error = %e, "Failed to duplicate entity");
                            }
                        }
                    });
                }
                    } else {
                        ui.text("No entity selected");
                        ui.text("Select an entity from the hierarchy to inspect its components.");

                        ui.separator();

                        // Create new entity button
                        if ui.button("Create New Entity") {
                            shared_state.with_world_write(|world| {
                                let new_entity = world.spawn((
                                    Name::new("New Entity"),
                                    Transform::default(),
                                    engine::prelude::GlobalTransform::default(),
                                ));

                                debug!(entity = ?new_entity, "Created new entity");
                                shared_state.set_selected_entity(Some(new_entity));
                                shared_state.mark_scene_modified();
                            });
                        }
                    }
                }); // End of inspector_content child window
        });
}

/// Render a component using its metadata if available
fn render_component_with_metadata<
    T: 'static + Send + Sync + engine::component_system::field_access::FieldAccess,
>(
    ui: &imgui::Ui,
    entity: hecs::Entity,
    component_name: &str,
    shared_state: &EditorSharedState,
    registry: &engine::io::component_registry::ComponentRegistry,
) -> bool {
    use std::any::TypeId;

    let mut component_modified = false;

    // Get component metadata
    let metadata = registry.get_metadata(TypeId::of::<T>());
    debug!(
        component = component_name,
        has_metadata = metadata.is_some(),
        has_ui_metadata = metadata
            .as_ref()
            .and_then(|m| m.ui_metadata.as_ref())
            .is_some(),
        "Checking component metadata"
    );

    if ui.collapsing_header(component_name, TreeNodeFlags::DEFAULT_OPEN) {
        let mut remove_component = false;

        // Check if we have UI metadata for this component
        if let Some(metadata) = metadata {
            if let Some(ui_metadata) = &metadata.ui_metadata {
                debug!(
                    component = component_name,
                    field_count = ui_metadata.fields.len(),
                    "Using metadata-based rendering"
                );
                // Use metadata-based rendering
                shared_state.with_world_write(|world| {
                    // We need to temporarily remove the component to get mutable access
                    if let Ok(mut component) = world.inner_mut().remove_one::<T>(entity) {
                        // Use the metadata renderer directly
                        let modified = crate::ui_metadata_renderer::render_component_ui(
                            ui,
                            &mut component,
                            ui_metadata,
                            entity,
                        );

                        if modified {
                            component_modified = true;
                            shared_state.mark_scene_modified();
                            debug!(
                                component = component_name,
                                "Component modified via metadata UI"
                            );
                        }

                        // Always re-insert the component
                        let _ = world.insert_one(entity, component);
                    }
                });
            } else {
                // No UI metadata, show placeholder
                ui.text(format!("No UI metadata for {component_name}"));
            }
        } else {
            // Component not registered
            ui.text(format!("{component_name} not registered"));
        }

        // Remove component button
        ui.separator();
        if ui.small_button(format!("Remove##{component_name}")) {
            shared_state.with_world_write(|world| {
                world.inner_mut().remove_one::<T>(entity).ok();
                remove_component = true;
                debug!(entity = ?entity, component = component_name, "Removed component");
            });
        }

        if remove_component {
            shared_state.mark_scene_modified();
        }
    }

    component_modified
}

/// Duplicate an entity with all its components
fn duplicate_entity(
    world: &mut engine::core::entity::World,
    entity: hecs::Entity,
    _registry: &engine::io::component_registry::ComponentRegistry,
) -> Result<hecs::Entity, String> {
    use engine::prelude::*;
    use serde_json;

    // Create a temporary scene with just this entity
    let mut entity_components = std::collections::HashMap::new();

    // Manually serialize each component type we know about
    // This is a workaround until we have better type-erased component access

    // Transform components
    if let Ok(transform) = world.get::<Transform>(entity) {
        entity_components.insert(
            "Transform".to_string(),
            serde_json::to_value(*transform).map_err(|e| e.to_string())?,
        );
    }

    if let Ok(global_transform) = world.get::<GlobalTransform>(entity) {
        entity_components.insert(
            "GlobalTransform".to_string(),
            serde_json::to_value(*global_transform).map_err(|e| e.to_string())?,
        );
    }

    // Name component
    if let Ok(name) = world.get::<Name>(entity) {
        let new_name = Name::new(format!("{} (Copy)", name.0));
        entity_components.insert(
            "Name".to_string(),
            serde_json::to_value(&new_name).map_err(|e| e.to_string())?,
        );
    }

    // Graphics components
    if let Ok(mesh_id) = world.get::<engine::graphics::MeshId>(entity) {
        entity_components.insert(
            "MeshId".to_string(),
            serde_json::to_value((*mesh_id).clone()).map_err(|e| e.to_string())?,
        );
    }

    if let Ok(material) = world.get::<engine::graphics::Material>(entity) {
        entity_components.insert(
            "Material".to_string(),
            serde_json::to_value(*material).map_err(|e| e.to_string())?,
        );
    }

    // Camera
    if let Ok(camera) = world.get::<engine::core::camera::Camera>(entity) {
        entity_components.insert(
            "Camera".to_string(),
            serde_json::to_value(*camera).map_err(|e| e.to_string())?,
        );
    }

    // Physics components
    if let Ok(rigid_body) = world.get::<engine::physics::RigidBody>(entity) {
        entity_components.insert(
            "RigidBody".to_string(),
            serde_json::to_value((*rigid_body).clone()).map_err(|e| e.to_string())?,
        );
    }

    if let Ok(collider) = world.get::<engine::physics::Collider>(entity) {
        entity_components.insert(
            "Collider".to_string(),
            serde_json::to_value((*collider).clone()).map_err(|e| e.to_string())?,
        );
    }

    if let Ok(velocity) = world.get::<engine::physics::PhysicsVelocity>(entity) {
        entity_components.insert(
            "PhysicsVelocity".to_string(),
            serde_json::to_value(*velocity).map_err(|e| e.to_string())?,
        );
    }

    if let Ok(mass) = world.get::<engine::physics::PhysicsMass>(entity) {
        entity_components.insert(
            "PhysicsMass".to_string(),
            serde_json::to_value(*mass).map_err(|e| e.to_string())?,
        );
    }

    // Scripting components
    if let Ok(script_ref) = world.get::<engine::scripting::ScriptRef>(entity) {
        entity_components.insert(
            "ScriptRef".to_string(),
            serde_json::to_value((*script_ref).clone()).map_err(|e| e.to_string())?,
        );
    }

    if let Ok(script_props) = world.get::<engine::scripting::ScriptProperties>(entity) {
        entity_components.insert(
            "ScriptProperties".to_string(),
            serde_json::to_value((*script_props).clone()).map_err(|e| e.to_string())?,
        );
    }

    // Now create a new entity and deserialize the components onto it
    let new_entity = world.spawn(());

    // Use the instantiate logic from Scene
    for (component_type, value) in entity_components {
        match component_type.as_str() {
            "Transform" => {
                let transform: Transform = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize Transform: {e}"))?;
                world
                    .insert_one(new_entity, transform)
                    .map_err(|e| format!("Failed to insert Transform: {e:?}"))?;
            }
            "GlobalTransform" => {
                let global_transform: GlobalTransform = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize GlobalTransform: {e}"))?;
                world
                    .insert_one(new_entity, global_transform)
                    .map_err(|e| format!("Failed to insert GlobalTransform: {e:?}"))?;
            }
            "Name" => {
                let name: Name = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize Name: {e}"))?;
                world
                    .insert_one(new_entity, name)
                    .map_err(|e| format!("Failed to insert Name: {e:?}"))?;
            }
            "MeshId" => {
                let mesh_id: engine::graphics::MeshId = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize MeshId: {e}"))?;
                world
                    .insert_one(new_entity, mesh_id)
                    .map_err(|e| format!("Failed to insert MeshId: {e:?}"))?;
            }
            "Material" => {
                let material: engine::graphics::Material = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize Material: {e}"))?;
                world
                    .insert_one(new_entity, material)
                    .map_err(|e| format!("Failed to insert Material: {e:?}"))?;
            }
            "Camera" => {
                let camera: engine::core::camera::Camera = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize Camera: {e}"))?;
                world
                    .insert_one(new_entity, camera)
                    .map_err(|e| format!("Failed to insert Camera: {e:?}"))?;
            }
            "RigidBody" => {
                let rigid_body: engine::physics::RigidBody = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize RigidBody: {e}"))?;
                world
                    .insert_one(new_entity, rigid_body)
                    .map_err(|e| format!("Failed to insert RigidBody: {e:?}"))?;
            }
            "Collider" => {
                let collider: engine::physics::Collider = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize Collider: {e}"))?;
                world
                    .insert_one(new_entity, collider)
                    .map_err(|e| format!("Failed to insert Collider: {e:?}"))?;
            }
            "PhysicsVelocity" => {
                let velocity: engine::physics::PhysicsVelocity = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize PhysicsVelocity: {e}"))?;
                world
                    .insert_one(new_entity, velocity)
                    .map_err(|e| format!("Failed to insert PhysicsVelocity: {e:?}"))?;
            }
            "PhysicsMass" => {
                let mass: engine::physics::PhysicsMass = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize PhysicsMass: {e}"))?;
                world
                    .insert_one(new_entity, mass)
                    .map_err(|e| format!("Failed to insert PhysicsMass: {e:?}"))?;
            }
            "ScriptRef" => {
                let script_ref: engine::scripting::ScriptRef = serde_json::from_value(value)
                    .map_err(|e| format!("Failed to deserialize ScriptRef: {e}"))?;
                world
                    .insert_one(new_entity, script_ref)
                    .map_err(|e| format!("Failed to insert ScriptRef: {e:?}"))?;
            }
            "ScriptProperties" => {
                let script_props: engine::scripting::ScriptProperties =
                    serde_json::from_value(value)
                        .map_err(|e| format!("Failed to deserialize ScriptProperties: {e}"))?;
                world
                    .insert_one(new_entity, script_props)
                    .map_err(|e| format!("Failed to insert ScriptProperties: {e:?}"))?;
            }
            _ => {
                debug!(
                    component_type = component_type,
                    "Skipping unknown component type during duplication"
                );
            }
        }
    }

    // Note: Parent-child relationships are not duplicated to avoid hierarchy issues
    // The duplicated entity will be a standalone entity

    Ok(new_entity)
}
