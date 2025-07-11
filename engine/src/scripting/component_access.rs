use crate::core::entity::components::Transform;
use crate::core::entity::{Entity, Name};
use crate::graphics::material::Material;
use crate::scripting::commands::ComponentCache;
use hecs::World;
use tracing::{debug, error, trace};

/// Query a specific component from the world and populate the cache
pub fn query_component(
    world: &World,
    entity_id: u64,
    component_type: &str,
    cache: &mut ComponentCache,
) -> Result<(), String> {
    let entity = match Entity::from_bits(entity_id) {
        Some(ent) => ent,
        None => return Err(format!("Invalid entity ID: {entity_id}")),
    };

    if !world.contains(entity) {
        return Err(format!("Entity {entity_id} does not exist"));
    }

    match component_type {
        "Transform" => {
            if let Ok(transform) = world.get::<&Transform>(entity) {
                cache.transforms.insert(entity_id, *transform);
                trace!(entity = entity_id, "Cached Transform component");
                Ok(())
            } else {
                Err(format!("Entity {entity_id} missing Transform component"))
            }
        }
        "Material" => {
            if let Ok(material) = world.get::<&Material>(entity) {
                cache.materials.insert(entity_id, *material);
                trace!(entity = entity_id, "Cached Material component");
                Ok(())
            } else {
                Err(format!("Entity {entity_id} missing Material component"))
            }
        }
        "Name" => {
            if let Ok(name) = world.get::<&Name>(entity) {
                cache.names.insert(entity_id, name.0.clone());
                trace!(entity = entity_id, "Cached Name component");
                Ok(())
            } else {
                Err(format!("Entity {entity_id} missing Name component"))
            }
        }
        _ => Err(format!("Unknown component type: {component_type}")),
    }
}

/// Query all entities with a specific component type and populate the cache
pub fn query_entities_with_component(
    world: &World,
    component_type: &str,
    cache: &mut ComponentCache,
) -> Vec<u64> {
    let mut entities = Vec::new();

    match component_type {
        "Transform" => {
            for (entity, transform) in world.query::<&Transform>().iter() {
                let entity_id = entity.to_bits().get();
                cache.transforms.insert(entity_id, *transform);
                entities.push(entity_id);
            }
            debug!(count = entities.len(), "Queried entities with Transform");
        }
        "Material" => {
            for (entity, material) in world.query::<&Material>().iter() {
                let entity_id = entity.to_bits().get();
                cache.materials.insert(entity_id, *material);
                entities.push(entity_id);
            }
            debug!(count = entities.len(), "Queried entities with Material");
        }
        "Name" => {
            for (entity, name) in world.query::<&Name>().iter() {
                let entity_id = entity.to_bits().get();
                cache.names.insert(entity_id, name.0.clone());
                entities.push(entity_id);
            }
            debug!(count = entities.len(), "Queried entities with Name");
        }
        _ => {
            error!(component_type, "Unknown component type in query");
        }
    }

    entities
}

/// Populate cache with all relevant components for scripting
pub fn populate_cache_for_scripts(world: &World, cache: &mut ComponentCache) {
    cache.clear();

    // Query all entities with scriptable components
    for (entity, (transform, material, name)) in world
        .query::<(&Transform, Option<&Material>, Option<&Name>)>()
        .iter()
    {
        let entity_id = entity.to_bits().get();

        // Always cache transform
        cache.transforms.insert(entity_id, *transform);

        // Cache material if present
        if let Some(mat) = material {
            cache.materials.insert(entity_id, *mat);
        }

        // Cache name if present
        if let Some(n) = name {
            cache.names.insert(entity_id, n.0.clone());
        }
    }

    debug!(
        transforms = cache.transforms.len(),
        materials = cache.materials.len(),
        names = cache.names.len(),
        "Populated component cache for scripts"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_component() {
        let mut world = World::new();
        let mut cache = ComponentCache::new();

        // Create test entity
        let entity = world.spawn((
            Transform::default(),
            Material::default(),
            Name::new("Test Entity"),
        ));
        let entity_id = entity.to_bits().get();

        // Test querying each component type
        assert!(query_component(&world, entity_id, "Transform", &mut cache).is_ok());
        assert!(cache.transforms.contains_key(&entity_id));

        assert!(query_component(&world, entity_id, "Material", &mut cache).is_ok());
        assert!(cache.materials.contains_key(&entity_id));

        assert!(query_component(&world, entity_id, "Name", &mut cache).is_ok());
        assert_eq!(
            cache.names.get(&entity_id),
            Some(&"Test Entity".to_string())
        );

        // Test querying non-existent entity
        assert!(query_component(&world, 9999, "Transform", &mut cache).is_err());

        // Test unknown component type
        assert!(query_component(&world, entity_id, "Unknown", &mut cache).is_err());
    }

    #[test]
    fn test_query_entities_with_component() {
        let mut world = World::new();
        let mut cache = ComponentCache::new();

        // Create test entities
        let _e1 = world.spawn((Transform::default(), Material::default()));
        let _e2 = world.spawn((Transform::default(), Name::new("Entity 2")));
        let _e3 = world.spawn((
            Transform::default(),
            Material::default(),
            Name::new("Entity 3"),
        ));

        // Query transforms
        let transform_entities = query_entities_with_component(&world, "Transform", &mut cache);
        assert_eq!(transform_entities.len(), 3);
        assert_eq!(cache.transforms.len(), 3);

        // Clear and query materials
        cache.clear();
        let material_entities = query_entities_with_component(&world, "Material", &mut cache);
        assert_eq!(material_entities.len(), 2);
        assert_eq!(cache.materials.len(), 2);

        // Clear and query names
        cache.clear();
        let name_entities = query_entities_with_component(&world, "Name", &mut cache);
        assert_eq!(name_entities.len(), 2);
        assert_eq!(cache.names.len(), 2);
    }

    #[test]
    fn test_populate_cache() {
        let mut world = World::new();
        let mut cache = ComponentCache::new();

        // Create various entities
        world.spawn((Transform::default(), Material::default(), Name::new("Full")));
        world.spawn((Transform::default(), Material::default()));
        world.spawn((Transform::default(), Name::new("TransformName")));
        world.spawn((Transform::default(),));

        populate_cache_for_scripts(&world, &mut cache);

        assert_eq!(cache.transforms.len(), 4);
        assert_eq!(cache.materials.len(), 2);
        assert_eq!(cache.names.len(), 2);
    }
}
