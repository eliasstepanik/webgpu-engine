//! Component registry UI utilities for the editor

use engine::component_system::{ComponentMetadata, ComponentRegistryExt};
use engine::io::component_registry::ComponentRegistry;
use imgui::*;
use tracing::debug;

/// Render the "Add Component" UI using the component registry
pub fn render_add_component_popup(
    ui: &Ui,
    registry: &ComponentRegistry,
    world: &mut engine::core::entity::World,
    entity: hecs::Entity,
    filter: &str,
) -> bool {
    let mut component_added = false;

    // Get all available components from the registry
    let components: Vec<&ComponentMetadata> = registry.iter_metadata().collect();

    for metadata in components {
        // Filter by name
        if !filter.is_empty()
            && !metadata
                .name
                .to_lowercase()
                .contains(&filter.to_lowercase())
        {
            continue;
        }

        // Check if entity already has this component
        // TODO: Need a way to check if entity has component by TypeId

        if ui.selectable(metadata.name) {
            // Add default component to entity
            if let Err(e) = (metadata.add_default)(world, entity) {
                debug!("Failed to add component {}: {}", metadata.name, e);
            } else {
                debug!("Added component {} to entity {:?}", metadata.name, entity);
                component_added = true;
            }
        }
    }

    component_added
}

/// Render components using the registry
pub fn render_components_with_registry(
    ui: &Ui,
    registry: &ComponentRegistry,
    _world: &mut engine::core::entity::World,
    entity: hecs::Entity,
) -> bool {
    let any_modified = false;

    // Iterate through all registered components
    for metadata in registry.iter_metadata() {
        // Check if entity has this component type
        // For now, we'll need to use the UI builder if available
        if let Some(_ui_builder) = &metadata.ui_builder {
            // Try to render the component UI
            // The UI builder will check internally if the component exists
            let header_id = format!("{}##{:?}", metadata.name, entity);

            if ui.collapsing_header(&header_id, TreeNodeFlags::DEFAULT_OPEN) {
                // The UI builder needs a mutable reference, but we can't safely cast
                // For now, we'll skip the UI rendering
                // TODO: Implement a proper solution for UI rendering

                // if (ui_builder)(world, entity, ui_any_mut) {
                //     any_modified = true;
                // }

                // Add remove button
                if ui.small_button(format!("Remove {}", metadata.name)) {
                    // TODO: Need a way to remove component by TypeId
                    debug!("Remove component {} requested", metadata.name);
                }
            }
        }
    }

    any_modified
}
