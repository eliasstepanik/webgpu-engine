//! Tests for the modular component system

use super::*;
use crate::io::component_registry::ComponentRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
struct TestComponent {
    value: i32,
    name: String,
}

// Manually implement Component trait for testing
impl Component for TestComponent {
    fn component_name() -> &'static str {
        "TestComponent"
    }

    fn register(registry: &mut ComponentRegistry) {
        let metadata = ComponentMetadata::new::<Self>(Self::component_name());
        registry.register_with_metadata(metadata);
    }
}

// Manually implement EditorUI trait for testing
impl EditorUI for TestComponent {
    fn build_ui(
        _component: &mut Self,
        _ui: &mut dyn std::any::Any,
        _entity: hecs::Entity,
    ) -> bool {
        false
    }
}

#[test]
fn test_component_metadata_creation() {
    let metadata = ComponentMetadata::new::<TestComponent>("TestComponent");
    
    assert_eq!(metadata.name, "TestComponent");
    assert_eq!(metadata.type_id, std::any::TypeId::of::<TestComponent>());
    assert!(metadata.ui_builder.is_none());
}

#[test]
fn test_component_metadata_with_ui() {
    let metadata = ComponentMetadata::new_with_ui::<TestComponent>("TestComponent");
    
    assert_eq!(metadata.name, "TestComponent");
    assert_eq!(metadata.type_id, std::any::TypeId::of::<TestComponent>());
    assert!(metadata.ui_builder.is_some());
}

#[test]
fn test_component_serialization() {
    let metadata = ComponentMetadata::new::<TestComponent>("TestComponent");
    let component = TestComponent {
        value: 42,
        name: "test".to_string(),
    };
    
    // Test serialization
    let serialized = (metadata.serializer)(&component).unwrap();
    assert_eq!(serialized["value"], 42);
    assert_eq!(serialized["name"], "test");
}

#[test]
fn test_component_deserialization() {
    let metadata = ComponentMetadata::new::<TestComponent>("TestComponent");
    let json = serde_json::json!({
        "value": 123,
        "name": "deserialized"
    });
    
    // Test deserialization
    let deserialized = (metadata.deserializer)(&json).unwrap();
    let component = deserialized.downcast_ref::<TestComponent>().unwrap();
    assert_eq!(component.value, 123);
    assert_eq!(component.name, "deserialized");
}

#[test]
fn test_component_registration() {
    let mut registry = ComponentRegistry::new();
    
    // Register using Component trait
    TestComponent::register(&mut registry);
    
    // Check that component is registered
    assert!(registry.is_registered("TestComponent"));
    
    // Check metadata
    let metadata = registry.get_metadata_by_name("TestComponent").unwrap();
    assert_eq!(metadata.name, "TestComponent");
}

#[test]
fn test_add_default_component() {
    use crate::core::entity::World;
    
    let mut world = World::default();
    let entity = world.spawn(());
    
    let metadata = ComponentMetadata::new::<TestComponent>("TestComponent");
    
    // Add default component
    (metadata.add_default)(&mut world, entity).unwrap();
    
    // Check that component was added
    assert!(world.get::<TestComponent>(entity).is_ok());
    let component = world.get::<TestComponent>(entity).unwrap();
    assert_eq!(component.value, 0);
    assert_eq!(component.name, "");
}

#[test]
fn test_registry_iteration() {
    let mut registry = ComponentRegistry::new();
    
    // Register multiple components
    TestComponent::register(&mut registry);
    
    // Add another test component type
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct AnotherComponent {
        data: f32,
    }
    
    impl Component for AnotherComponent {
        fn component_name() -> &'static str {
            "AnotherComponent"
        }
        
        fn register(registry: &mut ComponentRegistry) {
            let metadata = ComponentMetadata::new::<Self>(Self::component_name());
            registry.register_with_metadata(metadata);
        }
    }
    
    AnotherComponent::register(&mut registry);
    
    // Test iteration
    let names: Vec<&str> = registry.component_names();
    assert!(names.contains(&"TestComponent"));
    assert!(names.contains(&"AnotherComponent"));
    
    // Test metadata iteration
    let count = registry.iter_metadata().count();
    assert_eq!(count, 2);
}

#[test]
fn test_component_ui_builder() {
    use crate::core::entity::World;
    
    let mut world = World::default();
    let entity = world.spawn((TestComponent {
        value: 42,
        name: "test".to_string(),
    },));
    
    let metadata = ComponentMetadata::new_with_ui::<TestComponent>("TestComponent");
    
    // UI builder should exist
    assert!(metadata.ui_builder.is_some());
    
    // Test UI building (returns false as we don't have actual UI)
    let ui_builder = metadata.ui_builder.unwrap();
    let mut dummy_ui = ();
    let modified = ui_builder(&mut world, entity, &mut dummy_ui as &mut dyn Any);
    assert!(!modified);
}