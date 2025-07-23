//! Tests for the script lifecycle tracker

use crate::core::entity::{Transform, World};
use crate::scripting::lifecycle_tracker::{get_tracker, ScriptLifecycleTracker};
use crate::scripting::ScriptRef;

#[test]
fn test_entity_started_once_per_lifecycle() {
    let mut world = World::new();
    let mut tracker = ScriptLifecycleTracker::default();

    let entity = world.spawn((ScriptRef::new("test_script"), Transform::default()));

    // First call should mark as started
    assert!(!tracker.has_started(entity));
    tracker.mark_started(entity);
    assert!(tracker.has_started(entity));

    // Subsequent calls should not change state
    let initial_count = tracker.started_count();
    tracker.mark_started(entity); // Should be no-op
    assert!(tracker.has_started(entity));
    assert_eq!(tracker.started_count(), initial_count);
}

#[test]
fn test_entity_stays_in_tracker_across_frames() {
    let mut world = World::new();
    let mut tracker = ScriptLifecycleTracker::default();

    let entity = world.spawn((ScriptRef::new("test_script"), Transform::default()));

    // Mark as started
    tracker.mark_started(entity);
    assert!(tracker.has_started(entity));

    // Simulate multiple frames - entity should remain in tracker
    for i in 0..10 {
        assert!(tracker.has_started(entity), "Entity lost in frame {i}");
        assert!(
            tracker.active_entities.contains(&entity),
            "Entity not active in frame {i}"
        );
    }
}

#[test]
fn test_entity_properly_removed_when_destroyed() {
    let mut world = World::new();
    let mut tracker = ScriptLifecycleTracker::default();

    let entity = world.spawn((ScriptRef::new("test_script"), Transform::default()));

    // Mark as started
    tracker.mark_started(entity);
    assert!(tracker.has_started(entity));
    assert!(tracker.active_entities.contains(&entity));

    // Remove entity from tracker (simulating destruction)
    tracker.remove_entity(entity);

    // Entity should no longer be tracked
    assert!(!tracker.has_started(entity));
    assert!(!tracker.active_entities.contains(&entity));
}

#[test]
fn test_no_false_positive_removals() {
    let mut world = World::new();
    let mut tracker = ScriptLifecycleTracker::default();

    // Create multiple entities
    let entity1 = world.spawn((ScriptRef::new("script1"), Transform::default()));
    let entity2 = world.spawn((ScriptRef::new("script2"), Transform::default()));
    let entity3 = world.spawn((ScriptRef::new("script3"), Transform::default()));

    // Mark all as started
    tracker.mark_started(entity1);
    tracker.mark_started(entity2);
    tracker.mark_started(entity3);

    assert_eq!(tracker.started_count(), 3);
    assert_eq!(tracker.active_entities.len(), 3);

    // Remove only one entity
    tracker.remove_entity(entity2);

    // Other entities should still be tracked
    assert!(tracker.has_started(entity1));
    assert!(tracker.has_started(entity3));
    assert!(!tracker.has_started(entity2));

    assert_eq!(tracker.started_count(), 2);
    assert_eq!(tracker.active_entities.len(), 2);
}

#[test]
fn test_tracker_clear() {
    let mut world = World::new();
    let mut tracker = ScriptLifecycleTracker::default();

    // Create and track multiple entities
    for i in 0..5 {
        let entity = world.spawn((ScriptRef::new(format!("script{i}")), Transform::default()));
        tracker.mark_started(entity);
    }

    assert_eq!(tracker.started_count(), 5);
    assert_eq!(tracker.active_entities.len(), 5);
    assert!(tracker.debug_counter > 0);

    // Clear the tracker
    tracker.clear();

    assert_eq!(tracker.started_count(), 0);
    assert_eq!(tracker.active_entities.len(), 0);
    assert_eq!(tracker.debug_counter, 0);
}

#[test]
fn test_global_tracker_singleton() {
    // Test that the global tracker is a singleton
    let tracker1 = get_tracker();
    let tracker2 = get_tracker();

    // Both should be the same instance
    assert!(std::ptr::eq(tracker1, tracker2));
}

#[test]
fn test_entity_lifecycle_consistency() {
    let mut world = World::new();
    let mut tracker = ScriptLifecycleTracker::default();

    let entity = world.spawn((ScriptRef::new("test_script"), Transform::default()));

    // Entity should not be in any sets initially
    assert!(!tracker.has_started(entity));
    assert!(!tracker.active_entities.contains(&entity));

    // After marking as started, should be in both sets
    tracker.mark_started(entity);
    assert!(tracker.has_started(entity));
    assert!(tracker.active_entities.contains(&entity));

    // Both sets should be consistent
    assert!(tracker.started_entities.is_subset(&tracker.active_entities));
}
