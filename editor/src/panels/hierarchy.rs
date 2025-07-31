//! Scene hierarchy panel
//!
//! Displays all entities in the scene in a tree structure,
//! allowing selection and basic operations.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use engine::prelude::{Camera, GlobalTransform, Material, MeshId, Name, Parent, Transform, World};
use engine::profile_zone;
use imgui::*;
use std::collections::{HashMap, HashSet};
use tracing::{debug, trace, warn};

/// State for drag-drop operations
#[derive(Debug)]
struct HierarchyDragState {
    dragged_entity: Option<hecs::Entity>,
    drag_source_name: String,
}

impl HierarchyDragState {
    fn new() -> Self {
        Self {
            dragged_entity: None,
            drag_source_name: String::new(),
        }
    }
}

// Global state for drag-drop (following assets.rs pattern)
static mut HIERARCHY_DRAG_STATE: Option<HierarchyDragState> = None;

/// Get the hierarchy drag state, creating it if necessary
#[allow(static_mut_refs)]
fn get_hierarchy_drag_state() -> &'static mut HierarchyDragState {
    unsafe {
        if HIERARCHY_DRAG_STATE.is_none() {
            HIERARCHY_DRAG_STATE = Some(HierarchyDragState::new());
        }
        HIERARCHY_DRAG_STATE.as_mut().unwrap()
    }
}

/// Check if potential_ancestor is an ancestor of potential_descendan
fn is_ancestor_of(
    world: &World,
    potential_ancestor: hecs::Entity,
    potential_descendant: hecs::Entity,
) -> bool {
    let mut current = Some(potential_descendant);
    let mut visited = HashSet::new();

    while let Some(entity) = current {
        if entity == potential_ancestor {
            return true;
        }

        if !visited.insert(entity) {
            // Cycle detected
            return false;
        }

        // Get parent of current entity
        current = world.get::<Parent>(entity).ok().map(|parent| parent.0);
    }

    false
}

/// Render the scene hierarchy panel
pub fn render_hierarchy_panel(
    ui: &imgui::Ui,
    shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
) {
    profile_zone!("render_hierarchy_panel");

    let panel_id = PanelId("hierarchy".to_string());

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
        .size([280.0, 400.0], Condition::FirstUseEver)
        .position([20.0, 50.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
            // Access the world through shared state
            if let Some((parent_map, root_entities, _other_entities, selected_entity)) = shared_state
                .with_world_read(|world| {
                    // Debug info for troubleshooting
                    let total_entities = world.query::<()>().iter().count();
                    let entities_with_transform = world.query::<&Transform>().iter().count();
                    let entities_with_name = world.query::<&Name>().iter().count();
                    let entities_with_camera = world.query::<&Camera>().iter().count();
                    let entities_with_material = world.query::<&Material>().iter().count();
                    let entities_with_mesh = world.query::<&MeshId>().iter().count();

                    debug!(total_entities, entities_with_name, entities_with_transform, "Hierarchy entity counts");
                    debug!(entities_with_camera, entities_with_material, entities_with_mesh, "Hierarchy component counts");

                    // Log detailed component info for first few entities
                    for (entity, _) in world.query::<()>().iter().take(5) {
                        let has_name = world.get::<Name>(entity).is_ok();
                        let has_transform = world.get::<Transform>(entity).is_ok();
                        let has_camera = world.get::<Camera>(entity).is_ok();
                        let has_material = world.get::<Material>(entity).is_ok();
                        let has_mesh = world.get::<MeshId>(entity).is_ok();
                        let name = if has_name {
                            world.get::<Name>(entity).map(|n| n.0.clone()).unwrap_or_default()
                        } else { "NO_NAME".to_string() };

                        debug!(entity = ?entity, name = %name, has_name, has_transform, has_camera, has_material, has_mesh, "Entity details");
                    }

                    // Build parent-child relationships
                    let mut parent_map: HashMap<hecs::Entity, Vec<hecs::Entity>> = HashMap::new();
                    let mut root_entities = Vec::new();

                    // First pass: identify all entities and their parents
                    // Get all entities with transforms
                    let all_entities_with_transform: Vec<hecs::Entity> =
                        world.query::<&Transform>().iter().map(|(e, _)| e).collect();

                    // Check which entities have parents
                    for (entity, parent) in world.query::<&Parent>().iter() {
                        parent_map.entry(parent.0).or_default().push(entity);
                    }

                    // Entities with transforms but no parents are roots
                    for entity in all_entities_with_transform {
                        if world.get::<Parent>(entity).is_err() {
                            root_entities.push(entity);
                        }
                    }

                    // Get entities without Transform components
                    let other_entities: Vec<hecs::Entity> = world
                        .query::<()>()
                        .without::<&Transform>()
                        .iter()
                        .map(|(e, _)| e)
                        .collect();

                    // Get current selection
                    let selected_entity = shared_state.selected_entity();

                    (parent_map, root_entities, other_entities, selected_entity)
                })
            {
                // Render the hierarchy tree
                for root_entity in root_entities {
                    render_entity_tree(
                        ui,
                        shared_state,
                        root_entity,
                        &parent_map,
                        selected_entity,
                        0,
                    );
                }

                // Add drop targett for root level (to remove parent)
                ui.spacing();
                ui.spacing();

                // Create an invisible drop target for removing paren
                let drop_area_size = [ui.content_region_avail()[0], 50.0];
                let cursor_pos = ui.cursor_screen_pos();
                ui.invisible_button("root_drop_area", drop_area_size);

                if let Some(target) = ui.drag_drop_target() {
                    let state = get_hierarchy_drag_state();

                    if let Some(dragged) = state.dragged_entity {
                        // Visual feedback when hovering
                        if ui.is_item_hovered() {
                            ui.get_window_draw_list()
                                .add_rect(
                                    cursor_pos,
                                    [cursor_pos[0] + drop_area_size[0], cursor_pos[1] + drop_area_size[1]],
                                    [0.5, 0.5, 1.0, 0.5], // Blue for root drop
                                )
                                .build();

                            ui.tooltip_text("Drop here to remove parent");
                        }

                        // Accept drop to remove paren
                        if target.accept_payload_empty("ENTITY_PARENT", DragDropFlags::empty()).is_some() {
                            shared_state.with_world_write(|world| {
                                // First ensure hierarchy is up to date
                                engine::core::entity::update_hierarchy_system(world);

                                // Get the current world transform before removing paren
                                let world_matrix = if let Ok(global_transform) = world.get::<GlobalTransform>(dragged) {
                                    Some(global_transform.matrix)
                                } else if let Ok(transform) = world.get::<Transform>(dragged) {
                                    // If no GlobalTransform exists yet, compute it from Transform
                                    Some(transform.to_matrix())
                                } else {
                                    None
                                };

                                // Remove parent componen
                                let _ = world.inner_mut().remove_one::<Parent>(dragged);

                                // Update the local transform to maintain world position
                                if let Some(world_mat) = world_matrix {
                                    if let Ok(transform) = world.query_one_mut::<&mut Transform>(dragged) {
                                        let (scale, rotation, translation) = world_mat.to_scale_rotation_translation();
                                        transform.position = translation;
                                        transform.rotation = rotation;
                                        transform.scale = scale;
                                    }
                                }

                                // Update hierarchy immediately
                                engine::core::entity::update_hierarchy_system(world);
                            });

                            // Clear drag state
                            state.dragged_entity = None;
                            state.drag_source_name.clear();

                            debug!("Removed parent from entity {:?}", dragged);
                        }
                    }
                }

                ui.text_colored([0.5, 0.5, 0.8, 1.0], "↑ Drop here to remove parent");

                // Add Create Entity button
                ui.separator();
                if ui.button("Create New Entity") {
                    shared_state.with_world_write(|world| {
                        let new_entity = world.spawn((
                            Name::new("New Entity"),
                            Transform::default(),
                            GlobalTransform::default(),
                        ));

                        debug!(entity = ?new_entity, "Created new entity from hierarchy panel");
                        shared_state.set_selected_entity(Some(new_entity));
                        shared_state.mark_scene_modified();
                    });
                }
            } else {
                ui.text("Failed to access world data");
            }

            // Panel position and size are now managed by ImGui's docking system
        });
}

/// Recursively render an entity and its children
fn render_entity_tree(
    ui: &imgui::Ui,
    shared_state: &EditorSharedState,
    entity: hecs::Entity,
    parent_map: &HashMap<hecs::Entity, Vec<hecs::Entity>>,
    selected_entity: Option<hecs::Entity>,
    depth: usize,
) {
    // Create indentation for tree structure
    if depth > 0 {
        for _ in 0..depth {
            ui.indent();
        }
    }

    // Check if this entity has children
    let has_children = parent_map.contains_key(&entity);
    let entity_name = shared_state
        .with_world_read(|world| get_entity_name(world, entity))
        .unwrap_or_else(|| format!("Entity {entity:?}"));

    // Show tree node if has children
    if has_children {
        let node_flags = if Some(entity) == selected_entity {
            TreeNodeFlags::SELECTED | TreeNodeFlags::DEFAULT_OPEN
        } else {
            TreeNodeFlags::DEFAULT_OPEN
        };

        // Use entity ID in the tree node ID to ensure uniqueness
        let tree_id = format!("{entity_name}##{entity:?}");
        let is_open = ui.tree_node_config(&tree_id).flags(node_flags).push();

        // Check if the node was clicked
        if ui.is_item_clicked() {
            shared_state.set_selected_entity(Some(entity));
            debug!("Selected entity: {:?}", entity);
        }

        // Add drag source
        if ui
            .drag_drop_source_config("ENTITY_PARENT")
            .condition(Condition::Once)
            .begin()
            .is_some()
        {
            let state = get_hierarchy_drag_state();
            state.dragged_entity = Some(entity);
            state.drag_source_name = entity_name.clone();

            // Visual feedback during drag
            ui.text(format!("🔗 {entity_name}"));
        }

        // Add drop target
        if let Some(target) = ui.drag_drop_target() {
            let state = get_hierarchy_drag_state();

            if let Some(dragged) = state.dragged_entity {
                // Visual feedback when hovering
                let can_drop = dragged != entity
                    && !shared_state
                        .with_world_read(|world| is_ancestor_of(world, dragged, entity))
                        .unwrap_or(false);

                if ui.is_item_hovered() {
                    let color = if can_drop {
                        [0.0, 1.0, 0.0, 0.5] // Green for valid
                    } else {
                        [1.0, 0.0, 0.0, 0.5] // Red for invalid
                    };

                    ui.get_window_draw_list()
                        .add_rect(ui.item_rect_min(), ui.item_rect_max(), color)
                        .build();

                    // Tooltip explaining why drop is invalid
                    if !can_drop {
                        if dragged == entity {
                            ui.tooltip_text("Cannot parent entity to itself");
                        } else {
                            ui.tooltip_text("Cannot create circular dependency");
                        }
                    }
                }

                // Accept drop
                if target
                    .accept_payload_empty("ENTITY_PARENT", DragDropFlags::empty())
                    .is_some()
                    && can_drop
                {
                    // Perform the parenting
                    shared_state.with_world_write(|world| {
                        // First ensure hierarchy is up to date so GlobalTransforms exis
                        engine::core::entity::update_hierarchy_system(world);

                        // Get the current world transforms before parenting
                        let child_world_matrix =
                            if let Ok(global_transform) = world.get::<GlobalTransform>(dragged) {
                                Some(global_transform.matrix)
                            } else if let Ok(transform) = world.get::<Transform>(dragged) {
                                // If no GlobalTransform exists yet, compute it from Transform
                                Some(transform.to_matrix())
                            } else {
                                None
                            };

                        let parent_world_matrix =
                            if let Ok(global_transform) = world.get::<GlobalTransform>(entity) {
                                Some(global_transform.matrix)
                            } else if let Ok(transform) = world.get::<Transform>(entity) {
                                // If no GlobalTransform exists yet, compute it from Transform
                                Some(transform.to_matrix())
                            } else {
                                None
                            };

                        // Store exact world position with f64 precision for cameras
                        let original_world_pos = child_world_matrix.map(|m| {
                            let (_, _, translation) = m.to_scale_rotation_translation();
                            glam::DVec3::new(
                                translation.x as f64,
                                translation.y as f64,
                                translation.z as f64,
                            )
                        });

                        // Remove existing parent if any
                        let _ = world.inner_mut().remove_one::<Parent>(dragged);
                        // Add new paren
                        let _ = world.insert_one(dragged, Parent(entity));

                        // Check if this is a camera entity before transform adjustmen
                        let is_camera = world.get::<Camera>(dragged).is_ok();

                        // Adjust the child's local transform to maintain its world position
                        if let (Some(child_world), Some(parent_world)) =
                            (child_world_matrix, parent_world_matrix)
                        {
                            if let Ok(child_transform) =
                                world.query_one_mut::<&mut Transform>(dragged)
                            {
                                // Calculate the local transform relative to the new paren
                                // child_world = parent_world * child_local
                                // child_local = parent_world^-1 * child_world
                                let parent_inverse = parent_world.inverse();
                                let new_local_matrix = parent_inverse * child_world;
                                let (scale, rotation, translation) =
                                    new_local_matrix.to_scale_rotation_translation();

                                if is_camera {
                                    let old_pos = child_transform.position;
                                    trace!(
                                        old_local_pos = ?old_pos,
                                        new_local_pos = ?translation,
                                        child_world_matrix = ?child_world,
                                        parent_world_matrix = ?parent_world,
                                        "Camera parenting transform calculation"
                                    );
                                }

                                child_transform.position = translation;
                                child_transform.rotation = rotation.normalize(); // Ensure rotation is normalized
                                child_transform.scale = scale;
                            }
                        }

                        // Update hierarchy immediately to ensure GlobalTransforms are correc
                        engine::core::entity::update_hierarchy_system(world);

                        // Verify the camera's world position after hierarchy update
                        if is_camera {
                            if let Some(original_pos) = original_world_pos {
                                // Get the new world position and calculate drif
                                let (new_world_pos, drift) =
                                    if let Ok(new_global) = world.get::<GlobalTransform>(dragged) {
                                        let pos = new_global.position();
                                        let d = ((pos.x as f64 - original_pos.x).abs()
                                            + (pos.y as f64 - original_pos.y).abs()
                                            + (pos.z as f64 - original_pos.z).abs())
                                            / 3.0;
                                        (pos, d)
                                    } else {
                                        return; // Skip if we can't get the transform
                                    };

                                if drift > 0.0001 {
                                    warn!(
                                        entity = ?dragged,
                                        drift = drift,
                                        original_pos = ?original_pos,
                                        new_pos = ?new_world_pos,
                                        "Camera position drifted after parenting"
                                    );

                                    // If drift is significant, attempt to correct i
                                    if drift > 0.001 {
                                        if let Ok(child_transform) =
                                            world.query_one_mut::<&mut Transform>(dragged)
                                        {
                                            // Recalculate with higher precision
                                            let parent_world_f64 = parent_world_matrix.map(|m| {
                                                glam::DMat4::from_cols(
                                                    m.x_axis.as_dvec4(),
                                                    m.y_axis.as_dvec4(),
                                                    m.z_axis.as_dvec4(),
                                                    m.w_axis.as_dvec4(),
                                                )
                                            });

                                            if let Some(parent_f64) = parent_world_f64 {
                                                let parent_inverse_f64 = parent_f64.inverse();
                                                let child_local_pos_f64 = parent_inverse_f64
                                                    .transform_point3(original_pos);
                                                child_transform.position =
                                                    child_local_pos_f64.as_vec3();

                                                debug!(
                                                    entity = ?dragged,
                                                    corrected_pos = ?child_local_pos_f64,
                                                    "Applied drift correction to camera"
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    trace!(
                                        entity = ?dragged,
                                        drift = drift,
                                        "Camera parenting completed with minimal drift"
                                    );
                                }
                            }
                        }
                    });

                    // Clear drag state
                    state.dragged_entity = None;
                    state.drag_source_name.clear();

                    debug!("Parented entity {:?} to {:?}", dragged, entity);
                }
            }
        }

        if let Some(_token) = is_open {
            if let Some(children) = parent_map.get(&entity) {
                for &child in children {
                    render_entity_tree(
                        ui,
                        shared_state,
                        child,
                        parent_map,
                        selected_entity,
                        depth + 1,
                    );
                }
            }
        }
    } else {
        // Leaf node - just show selectable
        let is_selected = Some(entity) == selected_entity;
        // Use entity ID in the selectable ID to ensure uniqueness
        let selectable_id = format!("{entity_name}##{entity:?}");
        if ui
            .selectable_config(&selectable_id)
            .selected(is_selected)
            .build()
        {
            shared_state.set_selected_entity(Some(entity));
            debug!("Selected entity: {:?}", entity);
        }

        // Add drag source for leaf nodes too
        if ui
            .drag_drop_source_config("ENTITY_PARENT")
            .condition(Condition::Once)
            .begin()
            .is_some()
        {
            let state = get_hierarchy_drag_state();
            state.dragged_entity = Some(entity);
            state.drag_source_name = entity_name.clone();

            // Visual feedback during drag
            ui.text(format!("🔗 {entity_name}"));
        }

        // Add drop targett for leaf nodes
        if let Some(target) = ui.drag_drop_target() {
            let state = get_hierarchy_drag_state();

            if let Some(dragged) = state.dragged_entity {
                // Visual feedback when hovering
                let can_drop = dragged != entity
                    && !shared_state
                        .with_world_read(|world| is_ancestor_of(world, dragged, entity))
                        .unwrap_or(false);

                if ui.is_item_hovered() {
                    let color = if can_drop {
                        [0.0, 1.0, 0.0, 0.5] // Green for valid
                    } else {
                        [1.0, 0.0, 0.0, 0.5] // Red for invalid
                    };

                    ui.get_window_draw_list()
                        .add_rect(ui.item_rect_min(), ui.item_rect_max(), color)
                        .build();

                    // Tooltip explaining why drop is invalid
                    if !can_drop {
                        if dragged == entity {
                            ui.tooltip_text("Cannot parent entity to itself");
                        } else {
                            ui.tooltip_text("Cannot create circular dependency");
                        }
                    }
                }

                // Accept drop
                if target
                    .accept_payload_empty("ENTITY_PARENT", DragDropFlags::empty())
                    .is_some()
                    && can_drop
                {
                    // Perform the parenting
                    shared_state.with_world_write(|world| {
                        // First ensure hierarchy is up to date so GlobalTransforms exis
                        engine::core::entity::update_hierarchy_system(world);

                        // Get the current world transforms before parenting
                        let child_world_matrix =
                            if let Ok(global_transform) = world.get::<GlobalTransform>(dragged) {
                                Some(global_transform.matrix)
                            } else if let Ok(transform) = world.get::<Transform>(dragged) {
                                // If no GlobalTransform exists yet, compute it from Transform
                                Some(transform.to_matrix())
                            } else {
                                None
                            };

                        let parent_world_matrix =
                            if let Ok(global_transform) = world.get::<GlobalTransform>(entity) {
                                Some(global_transform.matrix)
                            } else if let Ok(transform) = world.get::<Transform>(entity) {
                                // If no GlobalTransform exists yet, compute it from Transform
                                Some(transform.to_matrix())
                            } else {
                                None
                            };

                        // Store exact world position with f64 precision for cameras
                        let original_world_pos = child_world_matrix.map(|m| {
                            let (_, _, translation) = m.to_scale_rotation_translation();
                            glam::DVec3::new(
                                translation.x as f64,
                                translation.y as f64,
                                translation.z as f64,
                            )
                        });

                        // Remove existing parent if any
                        let _ = world.inner_mut().remove_one::<Parent>(dragged);
                        // Add new paren
                        let _ = world.insert_one(dragged, Parent(entity));

                        // Check if this is a camera entity before transform adjustmen
                        let is_camera = world.get::<Camera>(dragged).is_ok();

                        // Adjust the child's local transform to maintain its world position
                        if let (Some(child_world), Some(parent_world)) =
                            (child_world_matrix, parent_world_matrix)
                        {
                            if let Ok(child_transform) =
                                world.query_one_mut::<&mut Transform>(dragged)
                            {
                                // Calculate the local transform relative to the new paren
                                // child_world = parent_world * child_local
                                // child_local = parent_world^-1 * child_world
                                let parent_inverse = parent_world.inverse();
                                let new_local_matrix = parent_inverse * child_world;
                                let (scale, rotation, translation) =
                                    new_local_matrix.to_scale_rotation_translation();

                                if is_camera {
                                    let old_pos = child_transform.position;
                                    trace!(
                                        old_local_pos = ?old_pos,
                                        new_local_pos = ?translation,
                                        child_world_matrix = ?child_world,
                                        parent_world_matrix = ?parent_world,
                                        "Camera parenting transform calculation"
                                    );
                                }

                                child_transform.position = translation;
                                child_transform.rotation = rotation.normalize(); // Ensure rotation is normalized
                                child_transform.scale = scale;
                            }
                        }

                        // Update hierarchy immediately to ensure GlobalTransforms are correc
                        engine::core::entity::update_hierarchy_system(world);

                        // Verify the camera's world position after hierarchy update
                        if is_camera {
                            if let Some(original_pos) = original_world_pos {
                                // Get the new world position and calculate drif
                                let (new_world_pos, drift) =
                                    if let Ok(new_global) = world.get::<GlobalTransform>(dragged) {
                                        let pos = new_global.position();
                                        let d = ((pos.x as f64 - original_pos.x).abs()
                                            + (pos.y as f64 - original_pos.y).abs()
                                            + (pos.z as f64 - original_pos.z).abs())
                                            / 3.0;
                                        (pos, d)
                                    } else {
                                        return; // Skip if we can't get the transform
                                    };

                                if drift > 0.0001 {
                                    warn!(
                                        entity = ?dragged,
                                        drift = drift,
                                        original_pos = ?original_pos,
                                        new_pos = ?new_world_pos,
                                        "Camera position drifted after parenting"
                                    );

                                    // If drift is significant, attempt to correct i
                                    if drift > 0.001 {
                                        if let Ok(child_transform) =
                                            world.query_one_mut::<&mut Transform>(dragged)
                                        {
                                            // Recalculate with higher precision
                                            let parent_world_f64 = parent_world_matrix.map(|m| {
                                                glam::DMat4::from_cols(
                                                    m.x_axis.as_dvec4(),
                                                    m.y_axis.as_dvec4(),
                                                    m.z_axis.as_dvec4(),
                                                    m.w_axis.as_dvec4(),
                                                )
                                            });

                                            if let Some(parent_f64) = parent_world_f64 {
                                                let parent_inverse_f64 = parent_f64.inverse();
                                                let child_local_pos_f64 = parent_inverse_f64
                                                    .transform_point3(original_pos);
                                                child_transform.position =
                                                    child_local_pos_f64.as_vec3();

                                                debug!(
                                                    entity = ?dragged,
                                                    corrected_pos = ?child_local_pos_f64,
                                                    "Applied drift correction to camera"
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    trace!(
                                        entity = ?dragged,
                                        drift = drift,
                                        "Camera parenting completed with minimal drift"
                                    );
                                }
                            }
                        }
                    });

                    // Clear drag state
                    state.dragged_entity = None;
                    state.drag_source_name.clear();

                    debug!("Parented entity {:?} to {:?}", dragged, entity);
                }
            }
        }
    }

    // Unindent for next items at same level
    if depth > 0 {
        for _ in 0..depth {
            ui.unindent();
        }
    }
}

/// Get a display name for an entity
fn get_entity_name(world: &World, entity: hecs::Entity) -> String {
    // Try Name component firs
    if let Ok(name) = world.get::<Name>(entity) {
        if !name.0.is_empty() {
            debug!(name = %name.0, entity = ?entity, "Found entity name");
            return name.0.clone();
        }
    }

    debug!(entity = ?entity, "No name found for entity, checking Transform");

    // Fallback to ID with component indicator
    if world.get::<Transform>(entity).is_ok() {
        format!("Entity {entity:?}")
    } else {
        format!("Entity {entity:?} [No Transform]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::core::entity::{Name, Transform, World};

    #[test]
    fn test_get_entity_name_with_name_component() {
        let mut world = World::new();
        let entity = world.spawn((Name::new("Test Entity"),));

        let name = get_entity_name(&world, entity);
        assert_eq!(name, "Test Entity");
    }

    #[test]
    fn test_get_entity_name_with_transform_no_name() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(),));

        let name = get_entity_name(&world, entity);
        assert!(name.starts_with("Entity"));
        assert!(!name.contains("[No Transform]"));
    }

    #[test]
    fn test_get_entity_name_no_components() {
        let mut world = World::new();
        let entity = world.spawn(());

        let name = get_entity_name(&world, entity);
        assert!(name.contains("[No Transform]"));
    }

    #[test]
    fn test_get_entity_name_empty_name() {
        let mut world = World::new();
        let entity = world.spawn((Name::new(""), Transform::default()));

        let name = get_entity_name(&world, entity);
        assert!(name.starts_with("Entity"));
        assert!(!name.contains("[No Transform]"));
    }
}
