//! Tests for borrow safety in the scripting system

use crate::core::entity::{Name, Transform, World};
use crate::scripting::property_types::ScriptProperties;
use crate::scripting::ScriptRef;

#[test]
fn test_multiple_entity_queries_dont_conflict() {
    let mut world = World::new();

    // Create multiple entities with scripts
    let _entity1 = world.spawn((
        ScriptRef::new("script1"),
        Transform::default(),
        Name::new("Entity1"),
    ));

    let _entity2 = world.spawn((
        ScriptRef::new("script2"),
        Transform::default(),
        Name::new("Entity2"),
    ));

    // Query for ScriptRef components
    let mut entities = Vec::new();
    for (entity, script_ref) in world.query::<&ScriptRef>().iter() {
        entities.push((entity, script_ref.name.clone()));
    }

    // After query is dropped, we should be able to query again
    let mut count = 0;
    for (_entity, _) in world.query::<&ScriptRef>().iter() {
        count += 1;
        // Should be able to query for other components
        let _ = world.query::<&Transform>().iter().count();
    }

    assert_eq!(count, 2);
}

#[test]
fn test_collection_first_pattern_works() {
    let mut world = World::new();

    // Create entities with scripts and properties
    let _entity1 = world.spawn((
        ScriptRef::new("script1"),
        ScriptProperties::new(),
        Transform::default(),
    ));

    let _entity2 = world.spawn((
        ScriptRef::new("script2"),
        Transform::default(), // No properties
    ));

    let _entity3 = world.spawn((
        ScriptRef::new("script3"),
        ScriptProperties::new(),
        Transform::default(),
    ));

    // CORRECT PATTERN: Collect all data first
    let mut collected_data = Vec::new();
    for (entity, (script_ref, properties)) in world
        .query::<(&ScriptRef, Option<&ScriptProperties>)>()
        .iter()
    {
        collected_data.push((entity, script_ref.name.clone(), properties.cloned()));
    }
    // Query is dropped here

    // Now we can safely access components individually
    assert_eq!(collected_data.len(), 3);

    // Verify we can now access components without conflicts
    for (entity, _script_name, properties) in &collected_data {
        // This should work without panic
        if let Ok(transform) = world.get::<Transform>(*entity) {
            // Successfully accessed component
            assert!(transform.position[0].is_finite());
        }

        // Check properties match what we collected
        match world.get::<ScriptProperties>(*entity) {
            Ok(_) => assert!(properties.is_some()),
            Err(_) => assert!(properties.is_none()),
        }
    }
}

#[test]
fn test_component_access_after_collection_is_safe() {
    let mut world = World::new();

    // Create test entities
    let entities: Vec<_> = (0..5)
        .map(|i| {
            world.spawn((
                ScriptRef::new(format!("script{i}")),
                Transform::default(),
                Name::new(format!("Entity{i}")),
            ))
        })
        .collect();

    // Collect entity IDs from query
    let entity_ids: Vec<_> = world.query::<&ScriptRef>().iter().map(|(e, _)| e).collect();
    // Query borrow is dropped here

    // Now we can safely access components
    for entity in &entity_ids {
        // All of these should work without panic
        assert!(world.get::<ScriptRef>(*entity).is_ok());
        assert!(world.get::<Transform>(*entity).is_ok());
        assert!(world.get::<Name>(*entity).is_ok());
    }

    assert_eq!(entity_ids.len(), entities.len());
}

#[test]
fn test_compound_query_avoids_conflicts() {
    let mut world = World::new();

    // Create entities with various component combinations
    world.spawn((ScriptRef::new("script1"), ScriptProperties::new()));
    world.spawn((ScriptRef::new("script2"),)); // No properties
    world.spawn((ScriptRef::new("script3"), ScriptProperties::new()));

    // Use compound query to get all needed components at once
    let mut results = Vec::new();
    for (entity, (script_ref, properties)) in world
        .query::<(&ScriptRef, Option<&ScriptProperties>)>()
        .iter()
    {
        results.push((entity, script_ref.name.clone(), properties.is_some()));
    }

    // Verify results
    assert_eq!(results.len(), 3);
    assert_eq!(
        results
            .iter()
            .filter(|(_, _, has_props)| *has_props)
            .count(),
        2
    );
}

#[test]
#[should_panic(expected = "query borrow conflict")]
#[ignore] // This test demonstrates the problem but will panic
fn test_borrow_conflict_demonstration() {
    let mut world = World::new();

    world.spawn((ScriptRef::new("script1"), ScriptProperties::new()));

    // This pattern causes a borrow conflict
    for (entity, _script_ref) in world.query::<&ScriptRef>().iter() {
        // This will panic due to borrow conflict
        let _ = world.get::<&ScriptProperties>(entity);
    }
}

#[test]
fn test_destruction_check_without_conflicts() {
    let mut world = World::new();

    // Create entities
    let _entity1 = world.spawn((ScriptRef::new("script1"), Transform::default()));
    let _entity2 = world.spawn((ScriptRef::new("script2"), Transform::default()));
    let _entity3 = world.spawn((Transform::default(),)); // No script

    // Collect entities with scripts
    let entities_with_scripts: Vec<_> =
        world.query::<&ScriptRef>().iter().map(|(e, _)| e).collect();

    assert_eq!(entities_with_scripts.len(), 2);

    // Later, check if entities still have scripts (without conflict)
    let mut still_have_scripts = Vec::new();
    for entity in &entities_with_scripts {
        if world.get::<ScriptRef>(*entity).is_ok() {
            still_have_scripts.push(*entity);
        }
    }

    assert_eq!(still_have_scripts.len(), 2);
}
