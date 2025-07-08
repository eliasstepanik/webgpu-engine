//! Entity ID mapping for scene serialization

use hecs::Entity;
use std::collections::HashMap;
use tracing::debug;

/// Maps old entity IDs to new entities during scene loading
///
/// When a scene is loaded, entity IDs from the serialized data need to be
/// mapped to the actual entities created in the world. This mapper maintains
/// the relationship between old IDs and new entities.
#[derive(Debug)]
pub struct EntityMapper {
    /// Maps old entity IDs to new entity handles
    mapping: HashMap<u64, Entity>,
}

impl EntityMapper {
    /// Create a new empty entity mapper
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    /// Register a mapping from an old entity ID to a new entity
    ///
    /// # Arguments
    /// * `old_id` - The entity ID from the serialized scene
    /// * `new_entity` - The actual entity created in the world
    pub fn register(&mut self, old_id: u64, new_entity: Entity) {
        debug!(old_id = old_id, new_entity = ?new_entity, "Registering entity mapping");
        self.mapping.insert(old_id, new_entity);
    }

    /// Look up the new entity for an old entity ID
    ///
    /// # Arguments
    /// * `old_id` - The entity ID from the serialized scene
    ///
    /// # Returns
    /// The new entity if found, or None if the ID wasn't registered
    pub fn remap(&self, old_id: u64) -> Option<Entity> {
        self.mapping.get(&old_id).copied()
    }

    /// Get the number of mapped entities
    pub fn len(&self) -> usize {
        self.mapping.len()
    }

    /// Check if the mapper is empty
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    /// Get all old IDs that have been registered
    pub fn old_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.mapping.keys().copied()
    }

    /// Get all new entities that have been registered
    pub fn new_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.mapping.values().copied()
    }

    /// Get an iterator over all (old_id, new_entity) pairs
    pub fn iter(&self) -> impl Iterator<Item = (u64, Entity)> + '_ {
        self.mapping.iter().map(|(&id, &entity)| (id, entity))
    }

    /// Clear all mappings
    pub fn clear(&mut self) {
        self.mapping.clear();
    }
}

impl Default for EntityMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_mapper_basic() {
        let mut mapper = EntityMapper::new();
        assert!(mapper.is_empty());
        assert_eq!(mapper.len(), 0);

        // Create a mock entity (in real code this would come from World::spawn)
        let entity = Entity::DANGLING; // Using DANGLING as a test entity
        mapper.register(42, entity);

        assert!(!mapper.is_empty());
        assert_eq!(mapper.len(), 1);
        assert_eq!(mapper.remap(42), Some(entity));
        assert_eq!(mapper.remap(99), None);
    }

    #[test]
    fn test_entity_mapper_multiple() {
        let mut mapper = EntityMapper::new();
        let entity1 = Entity::DANGLING;
        let entity2 = Entity::DANGLING;

        mapper.register(1, entity1);
        mapper.register(2, entity2);

        assert_eq!(mapper.len(), 2);
        assert_eq!(mapper.remap(1), Some(entity1));
        assert_eq!(mapper.remap(2), Some(entity2));
        assert_eq!(mapper.remap(3), None);
    }

    #[test]
    fn test_entity_mapper_clear() {
        let mut mapper = EntityMapper::new();
        mapper.register(1, Entity::DANGLING);
        mapper.register(2, Entity::DANGLING);

        assert_eq!(mapper.len(), 2);
        mapper.clear();
        assert_eq!(mapper.len(), 0);
        assert!(mapper.is_empty());
    }

    #[test]
    fn test_entity_mapper_iterators() {
        let mut mapper = EntityMapper::new();
        let entity1 = Entity::DANGLING;
        let entity2 = Entity::DANGLING;

        mapper.register(10, entity1);
        mapper.register(20, entity2);

        let old_ids: Vec<u64> = mapper.old_ids().collect();
        assert_eq!(old_ids.len(), 2);
        assert!(old_ids.contains(&10));
        assert!(old_ids.contains(&20));

        let entities: Vec<Entity> = mapper.new_entities().collect();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&entity1));
        assert!(entities.contains(&entity2));

        let pairs: Vec<(u64, Entity)> = mapper.iter().collect();
        assert_eq!(pairs.len(), 2);
        assert!(pairs.contains(&(10, entity1)));
        assert!(pairs.contains(&(20, entity2)));
    }

    #[test]
    fn test_entity_mapper_overwrite() {
        let mut mapper = EntityMapper::new();
        let entity1 = Entity::DANGLING;
        let entity2 = Entity::DANGLING;

        mapper.register(1, entity1);
        mapper.register(1, entity2); // Overwrite

        assert_eq!(mapper.len(), 1);
        assert_eq!(mapper.remap(1), Some(entity2));
    }
}
