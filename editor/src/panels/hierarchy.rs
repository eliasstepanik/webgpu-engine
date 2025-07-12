//! Scene hierarchy panel
//!
//! Displays all entities in the scene in a tree structure,
//! allowing selection and basic operations.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use engine::prelude::{Camera, Material, MeshId, Name, Parent, Transform, World};
use imgui::*;
use std::collections::HashMap;
use tracing::debug;

/// Render the scene hierarchy panel
pub fn render_hierarchy_panel(
    ui: &imgui::Ui,
    shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
) {
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
            if let Some((parent_map, root_entities, other_entities, selected_entity)) = shared_state
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

                // Also show entities without Transform components
                ui.separator();
                ui.text("Other Entities:");
                for entity in other_entities {
                    let is_selected = Some(entity) == selected_entity;
                    if ui
                        .selectable_config(format!("Entity {entity:?}"))
                        .selected(is_selected)
                        .build()
                    {
                        shared_state.set_selected_entity(Some(entity));
                        debug!("Selected entity: {:?}", entity);
                    }
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

        let is_open = ui.tree_node_config(&entity_name).flags(node_flags).push();

        // Check if the node was clicked
        if ui.is_item_clicked() {
            shared_state.set_selected_entity(Some(entity));
            debug!("Selected entity: {:?}", entity);
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
        if ui
            .selectable_config(&entity_name)
            .selected(is_selected)
            .build()
        {
            shared_state.set_selected_entity(Some(entity));
            debug!("Selected entity: {:?}", entity);
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
    // Try Name component first
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
