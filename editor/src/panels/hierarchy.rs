//! Scene hierarchy panel
//!
//! Displays all entities in the scene in a tree structure,
//! allowing selection and basic operations.

use engine::prelude::{Parent, Transform, World};
use imgui::*;
use std::collections::HashMap;
use tracing::debug;

/// Render the scene hierarchy panel
pub fn render_hierarchy_panel(
    ui: &imgui::Ui,
    world: &World,
    selected_entity: &mut Option<hecs::Entity>,
) {
    ui.window("Scene Hierarchy").resizable(true).build(|| {
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
            if world.get::<&Parent>(entity).is_err() {
                root_entities.push(entity);
            }
        }

        // Render the hierarchy tree
        for root_entity in root_entities {
            render_entity_tree(ui, world, root_entity, &parent_map, selected_entity, 0);
        }

        // Also show entities without Transform components
        ui.separator();
        ui.text("Other Entities:");
        for entity in world
            .query::<()>()
            .without::<&Transform>()
            .iter()
            .map(|(e, _)| e)
        {
            let is_selected = Some(entity) == *selected_entity;
            if ui
                .selectable_config(format!("Entity {entity:?}"))
                .selected(is_selected)
                .build()
            {
                *selected_entity = Some(entity);
                debug!("Selected entity: {:?}", entity);
            }
        }
    });
}

/// Recursively render an entity and its children
fn render_entity_tree(
    ui: &imgui::Ui,
    world: &World,
    entity: hecs::Entity,
    parent_map: &HashMap<hecs::Entity, Vec<hecs::Entity>>,
    selected_entity: &mut Option<hecs::Entity>,
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
    let entity_name = get_entity_name(world, entity);

    // Show tree node if has children
    if has_children {
        let node_flags = if Some(entity) == *selected_entity {
            TreeNodeFlags::SELECTED | TreeNodeFlags::DEFAULT_OPEN
        } else {
            TreeNodeFlags::DEFAULT_OPEN
        };

        let is_open = ui.tree_node_config(&entity_name).flags(node_flags).push();

        // Check if the node was clicked
        if ui.is_item_clicked() {
            *selected_entity = Some(entity);
            debug!("Selected entity: {:?}", entity);
        }

        if let Some(_token) = is_open {
            if let Some(children) = parent_map.get(&entity) {
                for &child in children {
                    render_entity_tree(ui, world, child, parent_map, selected_entity, depth + 1);
                }
            }
        }
    } else {
        // Leaf node - just show selectable
        let is_selected = Some(entity) == *selected_entity;
        if ui
            .selectable_config(&entity_name)
            .selected(is_selected)
            .build()
        {
            *selected_entity = Some(entity);
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
    // Check for common components to create a meaningful name
    if world.get::<&Transform>(entity).is_ok() {
        // For now, just use entity ID with Transform indicator
        format!("Entity {entity:?} [Transform]")
    } else {
        format!("Entity {entity:?}")
    }
}
