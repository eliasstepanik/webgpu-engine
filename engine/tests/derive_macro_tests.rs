//! Integration tests for the derive macros

use engine::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use engine::io::component_registry::ComponentRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, engine_derive::Component)]
#[component(name = "DerivedTestComponent")]
struct DerivedTestComponent {
    value: i32,
    text: String,
}

// Manual implementation for testing
impl EditorUI for DerivedTestComponent {
    fn build_ui(_component: &mut Self, _ui: &mut dyn std::any::Any, _entity: hecs::Entity) -> bool {
        false
    }
}

#[test]
fn test_derived_component_trait() {
    // Test that component_name works
    assert_eq!(
        DerivedTestComponent::component_name(),
        "DerivedTestComponent"
    );
}

#[test]
fn test_derived_component_registration() {
    let mut registry = ComponentRegistry::new();

    // Register using derived Component trait
    DerivedTestComponent::register(&mut registry);

    // Check that component is registered
    assert!(registry.is_registered("DerivedTestComponent"));

    // Check metadata
    let metadata = registry
        .get_metadata_by_name("DerivedTestComponent")
        .unwrap();
    assert_eq!(metadata.name, "DerivedTestComponent");
    assert_eq!(
        metadata.type_id,
        std::any::TypeId::of::<DerivedTestComponent>()
    );
}

#[test]
fn test_derived_component_serialization() {
    let mut registry = ComponentRegistry::new();
    DerivedTestComponent::register(&mut registry);

    let component = DerivedTestComponent {
        value: 42,
        text: "Hello, World!".to_string(),
    };

    // Serialize to JSON
    let json = serde_json::to_value(&component).unwrap();

    // Deserialize using registry
    let deserialized = registry
        .deserialize_component("DerivedTestComponent", &json)
        .unwrap();
    let result = deserialized.downcast_ref::<DerivedTestComponent>().unwrap();

    assert_eq!(result.value, 42);
    assert_eq!(result.text, "Hello, World!");
}

#[test]
fn test_multiple_derived_components() {
    #[derive(Debug, Clone, Serialize, Deserialize, Default, engine_derive::Component)]
    #[component(name = "FirstComponent")]
    struct FirstComponent {
        a: f32,
    }

    impl EditorUI for FirstComponent {
        fn build_ui(
            _component: &mut Self,
            _ui: &mut dyn std::any::Any,
            _entity: hecs::Entity,
        ) -> bool {
            false
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default, engine_derive::Component)]
    #[component(name = "SecondComponent")]
    struct SecondComponent {
        b: bool,
    }

    impl EditorUI for SecondComponent {
        fn build_ui(
            _component: &mut Self,
            _ui: &mut dyn std::any::Any,
            _entity: hecs::Entity,
        ) -> bool {
            false
        }
    }

    let mut registry = ComponentRegistry::new();

    // Register both components
    FirstComponent::register(&mut registry);
    SecondComponent::register(&mut registry);

    // Check both are registered
    assert!(registry.is_registered("FirstComponent"));
    assert!(registry.is_registered("SecondComponent"));

    // Check component names
    assert_eq!(FirstComponent::component_name(), "FirstComponent");
    assert_eq!(SecondComponent::component_name(), "SecondComponent");
}

#[test]
fn test_derived_editor_ui_trait() {
    use engine::core::entity::World;

    // Create a test component
    let mut component = DerivedTestComponent {
        value: 123,
        text: "test".to_string(),
    };

    // Dummy UI context
    let mut ui_context = ();

    // Create a valid entity
    let mut world = World::default();
    let entity = world.spawn(());

    // Call build_ui (should return false as it's not implemented)
    let modified = DerivedTestComponent::build_ui(
        &mut component,
        &mut ui_context as &mut dyn std::any::Any,
        entity,
    );

    assert!(!modified);
}

#[test]
fn test_component_without_explicit_name() {
    // Test that if no name attribute is provided, it uses the struct name
    #[derive(Debug, Clone, Serialize, Deserialize, Default, engine_derive::Component)]
    struct UnnamedComponent {
        data: i32,
    }

    impl EditorUI for UnnamedComponent {
        fn build_ui(
            _component: &mut Self,
            _ui: &mut dyn std::any::Any,
            _entity: hecs::Entity,
        ) -> bool {
            false
        }
    }

    assert_eq!(UnnamedComponent::component_name(), "UnnamedComponent");
}

#[test]
fn test_complex_component_types() {
    use glam::{Quat, Vec3};

    #[derive(Debug, Clone, Serialize, Deserialize, engine_derive::Component)]
    #[component(name = "ComplexTransform")]
    struct ComplexTransform {
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
        children: Vec<u64>,
        metadata: std::collections::HashMap<String, String>,
    }

    impl EditorUI for ComplexTransform {
        fn build_ui(
            _component: &mut Self,
            _ui: &mut dyn std::any::Any,
            _entity: hecs::Entity,
        ) -> bool {
            false
        }
    }

    impl Default for ComplexTransform {
        fn default() -> Self {
            Self {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
                children: Vec::new(),
                metadata: std::collections::HashMap::new(),
            }
        }
    }

    let mut registry = ComponentRegistry::new();
    ComplexTransform::register(&mut registry);

    // Test serialization of complex types
    let mut component = ComplexTransform {
        position: Vec3::new(1.0, 2.0, 3.0),
        children: vec![1, 2, 3],
        ..Default::default()
    };
    component
        .metadata
        .insert("key".to_string(), "value".to_string());

    let json = serde_json::to_value(&component).unwrap();
    let deserialized = registry
        .deserialize_component("ComplexTransform", &json)
        .unwrap();
    let result = deserialized.downcast_ref::<ComplexTransform>().unwrap();

    assert_eq!(result.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(result.children, vec![1, 2, 3]);
    assert_eq!(result.metadata.get("key").unwrap(), "value");
}
