//! Tests for the modular component system

use super::*;
use crate::io::component_registry::ComponentRegistry;
use crate::prelude::{Transform, Name, Camera, Material};
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
    fn build_ui(_component: &mut Self, _ui: &mut dyn std::any::Any, _entity: hecs::Entity) -> bool {
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
fn test_builtin_components_have_ui_metadata() {
    // Test Transform UI metadata
    let transform_metadata = Transform::ui_metadata();
    assert!(transform_metadata.is_some(), "Transform should have UI metadata");
    if let Some(metadata) = transform_metadata {
        println!("Transform UI metadata: {} fields", metadata.fields.len());
        assert!(metadata.fields.len() > 0, "Transform should have fields");
        for field in &metadata.fields {
            println!("  - Field: {}, Widget: {:?}", field.name, field.widget);
        }
    }

    // Test Name UI metadata
    let name_metadata = Name::ui_metadata();
    assert!(name_metadata.is_some(), "Name should have UI metadata");
    if let Some(metadata) = name_metadata {
        println!("Name UI metadata: {} fields", metadata.fields.len());
        assert!(metadata.fields.len() > 0, "Name should have fields");
    }

    // Test Camera UI metadata
    let camera_metadata = Camera::ui_metadata();
    assert!(camera_metadata.is_some(), "Camera should have UI metadata");
    if let Some(metadata) = camera_metadata {
        println!("Camera UI metadata: {} fields", metadata.fields.len());
        assert!(metadata.fields.len() > 0, "Camera should have fields");
    }

    // Test Material UI metadata
    let material_metadata = Material::ui_metadata();
    assert!(material_metadata.is_some(), "Material should have UI metadata");
    if let Some(metadata) = material_metadata {
        println!("Material UI metadata: {} fields", metadata.fields.len());
        assert!(metadata.fields.len() > 0, "Material should have fields");
    }
}

#[test]
fn test_component_registry_has_ui_metadata() {
    use crate::io::component_registry::ComponentRegistry;
    use std::any::TypeId;
    
    let registry = ComponentRegistry::with_default_components();
    
    // Check Transform
    let transform_meta = registry.get_metadata(TypeId::of::<Transform>()).unwrap();
    println!("Transform component registered with UI metadata: {:?}", transform_meta.ui_metadata.is_some());
    
    // Check Name
    let name_meta = registry.get_metadata(TypeId::of::<Name>()).unwrap();
    println!("Name component registered with UI metadata: {:?}", name_meta.ui_metadata.is_some());
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

#[test]
fn test_ui_metadata_generation() {
    use crate::component_system::ui_metadata::UIWidgetType;
    use engine_derive::{Component, EditorUI};
    
    #[derive(Component, EditorUI, Default, Serialize, Deserialize)]
    #[component(name = "TestUIComponent")]
    struct TestUIComponent {
        #[ui(range = 0.0..100.0, speed = 0.5, tooltip = "Test float value")]
        pub value: f32,
        
        #[ui(tooltip = "Test name field")]
        pub name: String,
        
        #[ui(readonly)]
        pub id: u32,
        
        #[ui(hidden)]
        pub internal_state: bool,
    }
    
    // Get the UI metadata
    let metadata = TestUIComponent::ui_metadata();
    assert!(metadata.is_some(), "UI metadata should be generated");
    
    let metadata = metadata.unwrap();
    
    // Check that we have the right number of fields (hidden fields are filtered out during generation)
    assert_eq!(metadata.fields.len(), 3, "Should have 3 fields (hidden ones are excluded)");
    
    // All fields in metadata should be visible since hidden ones are filtered out
    let visible_count = metadata.fields.iter().filter(|f| !f.hidden).count();
    assert_eq!(visible_count, 3, "All fields in metadata should be visible");
    
    // Check the float field
    let value_field = metadata.fields.iter().find(|f| f.name == "value").unwrap();
    assert_eq!(value_field.tooltip, Some("Test float value".to_string()));
    assert!(!value_field.hidden);
    assert!(!value_field.readonly);
    
    // Verify it's a DragFloat widget
    match &value_field.widget {
        UIWidgetType::DragFloat { min, max, speed, .. } => {
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 100.0);
            assert_eq!(*speed, 0.5);
        }
        _ => panic!("Expected DragFloat widget for float field"),
    }
    
    // Check the string field
    let name_field = metadata.fields.iter().find(|f| f.name == "name").unwrap();
    assert_eq!(name_field.tooltip, Some("Test name field".to_string()));
    assert!(!name_field.readonly);
    
    // Check the readonly field
    let id_field = metadata.fields.iter().find(|f| f.name == "id").unwrap();
    assert!(id_field.readonly);
    
    // Verify that hidden fields are not included in metadata
    let internal_field = metadata.fields.iter().find(|f| f.name == "internal_state");
    assert!(internal_field.is_none(), "Hidden fields should not be in metadata");
}
