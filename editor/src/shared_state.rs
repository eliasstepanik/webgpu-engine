//! Shared state for multi-window editor synchronization
//!
//! This module provides thread-safe shared state that can be accessed
//! from both the main editor window and detached panel windows.

use engine::core::entity::World;
use std::sync::{Arc, Mutex};
use tracing::{debug, warn};

/// Shared editor state that needs to be synchronized between windows
#[derive(Debug, Default)]
pub struct SharedEditorState {
    /// Currently selected entity in the hierarchy
    pub selected_entity: Option<hecs::Entity>,
    /// Whether the scene has been modified since last save
    pub scene_modified: bool,
    /// Current scene file path
    pub current_scene_path: Option<std::path::PathBuf>,
}

impl SharedEditorState {
    /// Create a new shared editor state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the selected entity
    pub fn set_selected_entity(&mut self, entity: Option<hecs::Entity>) {
        if self.selected_entity != entity {
            debug!(
                "Selected entity changed: {:?} -> {:?}",
                self.selected_entity, entity
            );
            self.selected_entity = entity;
        }
    }

    /// Mark the scene as modified
    pub fn mark_scene_modified(&mut self) {
        if !self.scene_modified {
            debug!("Scene marked as modified");
            self.scene_modified = true;
        }
    }

    /// Mark the scene as saved
    pub fn mark_scene_saved(&mut self) {
        if self.scene_modified {
            debug!("Scene marked as saved");
            self.scene_modified = false;
        }
    }

    /// Set the current scene path
    pub fn set_scene_path(&mut self, path: Option<std::path::PathBuf>) {
        if self.current_scene_path != path {
            debug!(
                "Scene path changed: {:?} -> {:?}",
                self.current_scene_path, path
            );
            self.current_scene_path = path;
        }
    }
}

/// Thread-safe wrapper for shared editor state
pub type SharedEditorStateHandle = Arc<Mutex<SharedEditorState>>;

/// Create a new shared editor state handle
pub fn create_shared_state() -> SharedEditorStateHandle {
    Arc::new(Mutex::new(SharedEditorState::new()))
}

/// Safely update the selected entity across all windows
pub fn update_selected_entity(
    shared_state: &SharedEditorStateHandle,
    entity: Option<hecs::Entity>,
) {
    match shared_state.lock() {
        Ok(mut state) => {
            state.set_selected_entity(entity);
        }
        Err(e) => {
            warn!("Failed to lock shared state for entity selection: {}", e);
        }
    }
}

/// Safely mark the scene as modified across all windows
pub fn mark_scene_modified(shared_state: &SharedEditorStateHandle) {
    match shared_state.lock() {
        Ok(mut state) => {
            state.mark_scene_modified();
        }
        Err(e) => {
            warn!("Failed to lock shared state for scene modification: {}", e);
        }
    }
}

/// Safely get the current selected entity
pub fn get_selected_entity(shared_state: &SharedEditorStateHandle) -> Option<hecs::Entity> {
    match shared_state.lock() {
        Ok(state) => state.selected_entity,
        Err(e) => {
            warn!(
                "Failed to lock shared state for reading selected entity: {}",
                e
            );
            None
        }
    }
}

/// Safely check if the scene is modified
pub fn is_scene_modified(shared_state: &SharedEditorStateHandle) -> bool {
    match shared_state.lock() {
        Ok(state) => state.scene_modified,
        Err(e) => {
            warn!(
                "Failed to lock shared state for reading scene modification: {}",
                e
            );
            false
        }
    }
}

/// Shared world state for multi-window access
/// Note: World access requires careful coordination to avoid conflicts
pub type SharedWorldHandle = Arc<Mutex<World>>;

/// Create a shared world handle
pub fn create_shared_world(world: World) -> SharedWorldHandle {
    Arc::new(Mutex::new(world))
}

/// Safely perform a read-only operation on the world
pub fn with_world_read<F, R>(shared_world: &SharedWorldHandle, f: F) -> Option<R>
where
    F: FnOnce(&World) -> R,
{
    match shared_world.lock() {
        Ok(world) => Some(f(&world)),
        Err(e) => {
            warn!("Failed to lock world for reading: {}", e);
            None
        }
    }
}

/// Safely perform a mutable operation on the world
pub fn with_world_write<F, R>(shared_world: &SharedWorldHandle, f: F) -> Option<R>
where
    F: FnOnce(&mut World) -> R,
{
    match shared_world.lock() {
        Ok(mut world) => Some(f(&mut world)),
        Err(e) => {
            warn!("Failed to lock world for writing: {}", e);
            None
        }
    }
}

/// Combined shared state for the entire editor
#[derive(Clone)]
pub struct EditorSharedState {
    /// Shared editor-specific state
    pub editor_state: SharedEditorStateHandle,
    /// Shared world state
    pub world: SharedWorldHandle,
}

impl EditorSharedState {
    /// Create a new combined shared state
    pub fn new(world: World) -> Self {
        Self {
            editor_state: create_shared_state(),
            world: create_shared_world(world),
        }
    }

    /// Get the selected entity safely
    pub fn selected_entity(&self) -> Option<hecs::Entity> {
        get_selected_entity(&self.editor_state)
    }

    /// Set the selected entity safely
    pub fn set_selected_entity(&self, entity: Option<hecs::Entity>) {
        update_selected_entity(&self.editor_state, entity);
    }

    /// Mark the scene as modified safely
    pub fn mark_scene_modified(&self) {
        mark_scene_modified(&self.editor_state);
    }

    /// Check if the scene is modified safely
    pub fn is_scene_modified(&self) -> bool {
        is_scene_modified(&self.editor_state)
    }

    /// Perform a read-only operation on the world
    pub fn with_world_read<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&World) -> R,
    {
        with_world_read(&self.world, f)
    }

    /// Perform a mutable operation on the world
    pub fn with_world_write<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut World) -> R,
    {
        with_world_write(&self.world, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::core::entity::{Name, Transform, World};

    #[test]
    fn test_shared_state_world_access() {
        let world = World::new();
        let shared_state = EditorSharedState::new(world);

        // Test read access works
        let result = shared_state.with_world_read(|world| world.query::<()>().iter().count());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_shared_state_world_write() {
        let world = World::new();
        let shared_state = EditorSharedState::new(world);

        // Test write access works
        let entity =
            shared_state.with_world_write(|world| world.spawn((Name::new("Test Entity"),)));
        assert!(entity.is_some());

        // Verify entity was created
        let count = shared_state.with_world_read(|world| world.query::<&Name>().iter().count());
        assert_eq!(count.unwrap(), 1);
    }

    #[test]
    fn test_entity_with_name_component() {
        let mut world = World::new();
        let entity = world.spawn((Name::new("Test Entity"), Transform::default()));

        // Verify component exists
        assert!(world.get::<Name>(entity).is_ok());
        let name = world.get::<Name>(entity).unwrap();
        assert_eq!(name.0, "Test Entity");
    }

    #[test]
    fn test_selected_entity_management() {
        let world = World::new();
        let shared_state = EditorSharedState::new(world);

        // Initially no entity selected
        assert_eq!(shared_state.selected_entity(), None);

        // Create and select an entity
        let entity = shared_state
            .with_world_write(|world| world.spawn((Name::new("Selected Entity"),)))
            .unwrap();

        shared_state.set_selected_entity(Some(entity));
        assert_eq!(shared_state.selected_entity(), Some(entity));

        // Clear selection
        shared_state.set_selected_entity(None);
        assert_eq!(shared_state.selected_entity(), None);
    }

    #[test]
    fn test_scene_modification_tracking() {
        let world = World::new();
        let shared_state = EditorSharedState::new(world);

        // Initially not modified
        assert!(!shared_state.is_scene_modified());

        // Mark as modified
        shared_state.mark_scene_modified();
        assert!(shared_state.is_scene_modified());
    }
}
