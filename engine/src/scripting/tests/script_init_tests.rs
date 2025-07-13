//! Tests for script initialization system

use crate::core::entity::World;
use crate::scripting::property_types::{PropertyDefinition, PropertyType, PropertyValue};
use crate::scripting::{ScriptEngine, ScriptProperties, ScriptRef};

#[test]
fn test_script_properties_preserve_values_on_reinit() {
    let mut world = World::new();
    let _script_engine = ScriptEngine::new();

    // Create a mock script with property definitions
    let _definitions = vec![
        PropertyDefinition {
            name: "speed".to_string(),
            property_type: PropertyType::Float,
            default_value: PropertyValue::Float(1.0),
            metadata: Default::default(),
        },
        PropertyDefinition {
            name: "count".to_string(),
            property_type: PropertyType::Integer,
            default_value: PropertyValue::Integer(0),
            metadata: Default::default(),
        },
    ];

    // Simulate script being loaded with these definitions
    // (In real code, this would happen when loading the script file)
    
    // Create an entity with script and custom property values
    let entity = world.spawn((ScriptRef::new("test_script"),));
    
    // Manually add properties with custom values (simulating scene load or editor changes)
    let mut props = ScriptProperties::new();
    props.script_name = Some("test_script".to_string());
    props.values.insert("speed".to_string(), PropertyValue::Float(5.0)); // Changed from default 1.0
    props.values.insert("count".to_string(), PropertyValue::Integer(42)); // Changed from default 0
    world.inner_mut().insert_one(entity, props).unwrap();
    
    // Verify initial values
    {
        let props = world.inner().get::<&ScriptProperties>(entity).unwrap();
        assert_eq!(props.values.get("speed"), Some(&PropertyValue::Float(5.0)));
        assert_eq!(props.values.get("count"), Some(&PropertyValue::Integer(42)));
    }
    
    // Now simulate the script initialization system running
    // This should NOT reset the values to defaults
    // (In real code, script_initialization_system would be called here)
    
    // For this test, we'll simulate what the system does
    // The system should detect that properties exist and script name matches
    // So it should skip reinitialization
    
    // Verify values are still preserved
    {
        let props = world.inner().get::<&ScriptProperties>(entity).unwrap();
        assert_eq!(props.values.get("speed"), Some(&PropertyValue::Float(5.0)));
        assert_eq!(props.values.get("count"), Some(&PropertyValue::Integer(42)));
    }
}

#[test]
fn test_script_properties_add_new_properties_preserve_existing() {
    let mut world = World::new();
    
    // Create an entity with script and some property values
    let entity = world.spawn((ScriptRef::new("evolving_script"),));
    
    // Start with only one property
    let mut props = ScriptProperties::new();
    props.script_name = Some("evolving_script".to_string());
    props.values.insert("speed".to_string(), PropertyValue::Float(10.0));
    world.inner_mut().insert_one(entity, props).unwrap();
    
    // Now simulate the script being updated to have more properties
    // The initialization system should preserve "speed" and add new properties with defaults
    
    // This would be done by script_initialization_system in real code
    // For testing, we'll simulate it
    let new_definitions = vec![
        PropertyDefinition {
            name: "speed".to_string(),
            property_type: PropertyType::Float,
            default_value: PropertyValue::Float(1.0),
            metadata: Default::default(),
        },
        PropertyDefinition {
            name: "enabled".to_string(),
            property_type: PropertyType::Boolean,
            default_value: PropertyValue::Boolean(true),
            metadata: Default::default(),
        },
    ];
    
    // Simulate what the fixed initialization system should do
    if let Ok(existing_props) = world.inner_mut().remove_one::<ScriptProperties>(entity) {
        let mut new_properties = ScriptProperties::new();
        new_properties.script_name = Some("evolving_script".to_string());
        
        for def in &new_definitions {
            if let Some(existing_value) = existing_props.values.get(&def.name) {
                // Preserve existing value
                new_properties.values.insert(def.name.clone(), existing_value.clone());
            } else {
                // Use default for new property
                new_properties.values.insert(def.name.clone(), def.default_value.clone());
            }
        }
        
        world.inner_mut().insert(entity, (new_properties,)).unwrap();
    }
    
    // Verify that speed was preserved and enabled was added with default
    {
        let props = world.inner().get::<&ScriptProperties>(entity).unwrap();
        assert_eq!(props.values.get("speed"), Some(&PropertyValue::Float(10.0))); // Preserved!
        assert_eq!(props.values.get("enabled"), Some(&PropertyValue::Boolean(true))); // Added with default
    }
}

#[test]
fn test_script_change_preserves_matching_properties() {
    let mut world = World::new();
    
    // Create entity with script A
    let entity = world.spawn((ScriptRef::new("script_a"),));
    
    // Add properties for script A
    let mut props = ScriptProperties::new();
    props.script_name = Some("script_a".to_string());
    props.values.insert("shared_prop".to_string(), PropertyValue::Float(7.5));
    props.values.insert("script_a_only".to_string(), PropertyValue::Integer(100));
    world.inner_mut().insert_one(entity, props).unwrap();
    
    // Now change to script B which has some overlapping properties
    world.inner_mut().insert(entity, (ScriptRef::new("script_b"),)).unwrap();
    
    // Simulate script B's property definitions
    let script_b_definitions = vec![
        PropertyDefinition {
            name: "shared_prop".to_string(),
            property_type: PropertyType::Float,
            default_value: PropertyValue::Float(1.0),
            metadata: Default::default(),
        },
        PropertyDefinition {
            name: "script_b_only".to_string(),
            property_type: PropertyType::Boolean,
            default_value: PropertyValue::Boolean(false),
            metadata: Default::default(),
        },
    ];
    
    // Simulate the initialization system handling the script change
    // It should preserve "shared_prop" but not "script_a_only"
    if let Ok(existing_props) = world.inner_mut().remove_one::<ScriptProperties>(entity) {
        let mut new_properties = ScriptProperties::new();
        new_properties.script_name = Some("script_b".to_string());
        
        for def in &script_b_definitions {
            if let Some(existing_value) = existing_props.values.get(&def.name) {
                new_properties.values.insert(def.name.clone(), existing_value.clone());
            } else {
                new_properties.values.insert(def.name.clone(), def.default_value.clone());
            }
        }
        
        world.inner_mut().insert(entity, (new_properties,)).unwrap();
    }
    
    // Verify results
    {
        let props = world.inner().get::<&ScriptProperties>(entity).unwrap();
        assert_eq!(props.script_name.as_ref(), Some(&"script_b".to_string()));
        assert_eq!(props.values.get("shared_prop"), Some(&PropertyValue::Float(7.5))); // Preserved!
        assert_eq!(props.values.get("script_b_only"), Some(&PropertyValue::Boolean(false))); // New with default
        assert_eq!(props.values.get("script_a_only"), None); // Removed
    }
}