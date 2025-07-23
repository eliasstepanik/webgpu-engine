//! Global lifecycle tracker for script entities

use hecs::Entity;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// Global tracker for script lifecycle state
pub static SCRIPT_LIFECYCLE_TRACKER: OnceLock<Mutex<ScriptLifecycleTracker>> = OnceLock::new();

/// Get or initialize the global script lifecycle tracker
pub fn get_tracker() -> &'static Mutex<ScriptLifecycleTracker> {
    SCRIPT_LIFECYCLE_TRACKER.get_or_init(|| {
        tracing::info!("Initializing SCRIPT_LIFECYCLE_TRACKER OnceLock");
        let tracker = ScriptLifecycleTracker::default();
        Mutex::new(tracker)
    })
}

/// Test function to verify global state
pub fn test_global_state() {
    let mut tracker = get_tracker().lock().unwrap();
    tracker.debug_counter += 100;
    tracing::info!("Test: incremented counter to {}", tracker.debug_counter);
}

/// Tracks script lifecycle state for entities
#[derive(Default)]
pub struct ScriptLifecycleTracker {
    /// Entities that have had on_start called
    pub started_entities: HashSet<Entity>,
    /// Entities that need on_destroy called
    pub active_entities: HashSet<Entity>,
    /// Debug counter to track function calls
    pub debug_counter: u32,
}

impl ScriptLifecycleTracker {
    /// Check if an entity has been started
    pub fn has_started(&self, entity: Entity) -> bool {
        let result = self.started_entities.contains(&entity);
        tracing::trace!(
            "Checking if entity {:?} (ID: {}) has started: {} (total started: {}, counter: {})",
            entity,
            entity.to_bits().get(),
            result,
            self.started_entities.len(),
            self.debug_counter
        );
        result
    }

    /// Mark an entity as started
    pub fn mark_started(&mut self, entity: Entity) {
        self.debug_counter += 1;
        let was_new = self.started_entities.insert(entity);
        let was_active = self.active_entities.insert(entity);
        if was_new {
            tracing::debug!(
                "Added entity {:?} (bits: {}) to started_entities. Total: {}, counter: {}",
                entity,
                entity.to_bits().get(),
                self.started_entities.len(),
                self.debug_counter
            );

            // Debug: print all entities in the set
            tracing::trace!(
                "All started entities: {:?}",
                self.started_entities
                    .iter()
                    .map(|e| format!("{:?}({})", e, e.to_bits().get()))
                    .collect::<Vec<_>>()
            );
        } else {
            tracing::warn!("Entity {:?} was already in started_entities!", entity);
        }

        if was_active {
            tracing::trace!(
                "Added entity {:?} (bits: {}) to active_entities. Total active: {}",
                entity,
                entity.to_bits().get(),
                self.active_entities.len()
            );
        }
    }

    /// Remove an entity from tracking
    pub fn remove_entity(&mut self, entity: Entity) {
        let was_in_started = self.started_entities.remove(&entity);
        let was_in_active = self.active_entities.remove(&entity);
        if was_in_started || was_in_active {
            tracing::debug!(
                "Removed entity {:?} from tracker (was_started: {}, was_active: {})",
                entity,
                was_in_started,
                was_in_active
            );
        }
    }

    /// Clear all tracked entities
    pub fn clear(&mut self) {
        let started_count = self.started_entities.len();
        let active_count = self.active_entities.len();
        self.started_entities.clear();
        self.active_entities.clear();
        self.debug_counter = 0;
        tracing::warn!(
            "Cleared lifecycle tracker! Had {} started and {} active entities",
            started_count,
            active_count
        );
    }

    /// Get the number of started entities
    pub fn started_count(&self) -> usize {
        self.started_entities.len()
    }

    /// Validate that the tracker state is consistent
    pub fn validate_consistency(&self) -> bool {
        // All started entities should be in active set
        let consistent = self.started_entities.is_subset(&self.active_entities);
        if !consistent {
            tracing::warn!(
                "Lifecycle tracker inconsistency detected! Started entities not subset of active entities"
            );
        }
        consistent
    }

    /// Debug the current state of the tracker
    pub fn debug_state(&self) {
        tracing::debug!(
            started_count = self.started_entities.len(),
            active_count = self.active_entities.len(),
            consistent = self.validate_consistency(),
            debug_counter = self.debug_counter,
            "Lifecycle tracker state"
        );

        // Log detailed information at trace level
        tracing::trace!(
            started_entities = ?self.started_entities.iter()
                .map(|e| format!("{:?}({})", e, e.to_bits().get()))
                .collect::<Vec<_>>(),
            active_entities = ?self.active_entities.iter()
                .map(|e| format!("{:?}({})", e, e.to_bits().get()))
                .collect::<Vec<_>>(),
            "Detailed lifecycle tracker entities"
        );
    }
}
