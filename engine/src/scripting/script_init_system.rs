//! System for initializing script properties when scripts are assigned to entities

use crate::core::entity::World;
use crate::scripting::{ScriptEngine, ScriptProperties, ScriptRef};
use tracing::{debug, error, warn};

/// System that ensures entities with ScriptRef also have appropriate ScriptProperties
pub fn script_initialization_system(world: &mut World, script_engine: &mut ScriptEngine) {
    // Collect entities that need property initialization
    let mut entities_needing_properties = Vec::new();

    // First pass: collect entities that might need initialization using compound query
    for (entity, (script_ref, properties)) in world
        .query::<(&ScriptRef, Option<&ScriptProperties>)>()
        .iter()
    {
        let needs_init = if let Some(props) = properties {
            // Check if the script has changed
            let script_changed = props.script_name.as_ref() != Some(&script_ref.name);
            if script_changed {
                debug!(
                    entity = ?entity,
                    old_script = ?props.script_name,
                    new_script = %script_ref.name,
                    has_values = !props.values.is_empty(),
                    "Script name mismatch - checking if initialization needed"
                );
            }
            script_changed
        } else {
            // No properties component exists
            debug!(entity = ?entity, script = %script_ref.name, "No ScriptProperties component");
            true
        };

        if needs_init {
            entities_needing_properties.push((entity, script_ref.name.clone()));
        }
    }

    // Early exit if no entities need processing
    if entities_needing_properties.is_empty() {
        return;
    }

    debug!(
        count = entities_needing_properties.len(),
        "Processing entities that need script property initialization"
    );

    // Process each entity that needs properties
    for (entity, script_name) in entities_needing_properties {
        // Double-check if properties exist and just need name update
        let (has_props, needs_name_only, script_name_matches) =
            if let Ok(props) = world.get::<&ScriptProperties>(entity) {
                (
                    true,
                    props.script_name.is_none() && !props.values.is_empty(),
                    props.script_name.as_ref() == Some(&script_name),
                )
            } else {
                (false, false, false)
            };

        if has_props {
            if needs_name_only {
                // Just update the script name
                if let Ok(mut existing_props) =
                    world.inner_mut().remove_one::<ScriptProperties>(entity)
                {
                    existing_props.script_name = Some(script_name.clone());

                    if let Err(e) = world.insert_one(entity, existing_props) {
                        error!(entity = ?entity, error = ?e, "Failed to update ScriptProperties");
                    } else {
                        debug!(
                            entity = ?entity,
                            script = %script_name,
                            "Updated script name on existing ScriptProperties"
                        );
                    }
                }
                continue;
            } else if script_name_matches {
                // Properties already exist with correct script name - skip
                debug!(
                    entity = ?entity,
                    script = %script_name,
                    "Skipping initialization - properties already exist with correct script name"
                );
                continue;
            }
        }

        // Ensure script is loaded
        if !script_engine.is_loaded(&script_name) {
            match script_engine.load_script_by_name(&script_name) {
                Ok(_) => {
                    debug!(script = %script_name, "Loaded script for property initialization");
                }
                Err(e) => {
                    error!(script = %script_name, error = %e, "Failed to load script");
                    continue;
                }
            }
        }

        // Get property definitions from the script
        if let Some(definitions) = script_engine.get_property_definitions(&script_name) {
            if !definitions.is_empty() {
                // Check if we need to preserve existing values
                let properties = if has_props {
                    // Entity already has properties - preserve existing values where possible
                    if let Ok(existing_props) =
                        world.inner_mut().remove_one::<ScriptProperties>(entity)
                    {
                        let mut new_properties = ScriptProperties::new();
                        new_properties.script_name = Some(script_name.clone());

                        // For each property in the new script's definitions
                        for def in &definitions {
                            // Check if we have an existing value for this property
                            if let Some(existing_value) = existing_props.values.get(&def.name) {
                                // Preserve the existing value
                                new_properties
                                    .values
                                    .insert(def.name.clone(), existing_value.clone());
                                debug!(
                                    entity = ?entity,
                                    property = %def.name,
                                    "Preserved existing property value"
                                );
                            } else {
                                // Use default value for new properties
                                new_properties
                                    .values
                                    .insert(def.name.clone(), def.default_value.clone());
                                debug!(
                                    entity = ?entity,
                                    property = %def.name,
                                    "Using default value for new property"
                                );
                            }
                        }
                        new_properties
                    } else {
                        // Failed to remove existing properties, create from defaults
                        ScriptProperties::from_definitions_for_script(&definitions, &script_name)
                    }
                } else {
                    // No existing properties, create from defaults
                    ScriptProperties::from_definitions_for_script(&definitions, &script_name)
                };

                // Add the component to the entity
                if let Err(e) = world.insert_one(entity, properties) {
                    error!(entity = ?entity, error = ?e, "Failed to add ScriptProperties");
                } else {
                    debug!(
                        entity = ?entity,
                        script = %script_name,
                        property_count = definitions.len(),
                        "Updated ScriptProperties component"
                    );
                }
            } else {
                // Script has no properties, add empty component with script name
                let mut properties = ScriptProperties::new();
                properties.script_name = Some(script_name.clone());
                if let Err(e) = world.insert_one(entity, properties) {
                    error!(entity = ?entity, error = ?e, "Failed to add empty ScriptProperties");
                }
            }
        } else {
            warn!(
                entity = ?entity,
                script = %script_name,
                "Script not found or has no property definitions"
            );
        }
    }
}
