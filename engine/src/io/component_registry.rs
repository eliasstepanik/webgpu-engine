//! Component registry for dynamic component deserialization

use crate::component_system::{ComponentMetadata, ComponentRegistryExt};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// A function that can deserialize a component from a JSON value
pub type ComponentDeserializerFn = Arc<
    dyn Fn(&serde_json::Value) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

/// Registry for component deserializers
///
/// This registry allows for dynamic component deserialization by registering
/// type-specific deserializer functions. While the current implementation uses
/// a hardcoded match statement in Scene::instantiate, this registry provides
/// a foundation for more extensible component systems.
#[derive(Default)]
pub struct ComponentRegistry {
    /// Maps component type names to their deserializer functions
    deserializers: HashMap<String, ComponentDeserializerFn>,
    /// Maps TypeId to component metadata
    metadata: HashMap<TypeId, ComponentMetadata>,
    /// Maps component names to TypeId for lookup
    name_to_type: HashMap<String, TypeId>,
}

impl ComponentRegistry {
    /// Create a new empty component registry
    pub fn new() -> Self {
        Self {
            deserializers: HashMap::new(),
            metadata: HashMap::new(),
            name_to_type: HashMap::new(),
        }
    }

    /// Register a component deserializer
    ///
    /// # Arguments
    /// * `type_name` - The name of the component type (e.g., "Transform")
    /// * `deserializer` - Function that deserializes the component from JSON
    pub fn register<T: 'static + serde::de::DeserializeOwned>(&mut self, type_name: &str) {
        let deserializer: ComponentDeserializerFn = Arc::new(move |value| {
            let component: T = serde_json::from_value(value.clone())?;
            Ok(Box::new(component))
        });

        self.deserializers
            .insert(type_name.to_string(), deserializer);
        debug!(type_name = type_name, "Registered component deserializer");
    }

    /// Deserialize a component from a JSON value
    ///
    /// # Arguments
    /// * `type_name` - The name of the component type
    /// * `value` - The JSON value to deserialize
    ///
    /// # Returns
    /// The deserialized component as a boxed Any, or an error if deserialization fails
    pub fn deserialize_component(
        &self,
        type_name: &str,
        value: &serde_json::Value,
    ) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(deserializer) = self.deserializers.get(type_name) {
            deserializer(value)
        } else {
            Err(format!("Unknown component type: {type_name}").into())
        }
    }

    /// Check if a component type is registered
    pub fn is_registered(&self, type_name: &str) -> bool {
        self.deserializers.contains_key(type_name)
    }

    /// Get all registered component type names
    pub fn registered_types(&self) -> impl Iterator<Item = &str> {
        self.deserializers.keys().map(|s| s.as_str())
    }

    /// Get the number of registered component types
    pub fn len(&self) -> usize {
        self.deserializers.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.deserializers.is_empty()
    }

    /// Create a registry with all default engine components registered
    pub fn with_default_components() -> Self {
        use crate::component_system::Component;
        use crate::core::camera::{Camera, CameraWorldPosition};
        use crate::core::coordinates::WorldTransform;
        use crate::core::entity::components::{
            GlobalTransform, GlobalWorldTransform, Name, ParentData, PreviousTransform, Transform,
        };
        use crate::graphics::{Material, MeshId};
        use crate::scripting::{ScriptProperties, ScriptRef};

        let mut registry = Self::new();

        // Register core components using the new Component trait
        Transform::register(&mut registry);
        PreviousTransform::register(&mut registry);
        GlobalTransform::register(&mut registry);
        GlobalWorldTransform::register(&mut registry);
        WorldTransform::register(&mut registry);
        Camera::register(&mut registry);
        CameraWorldPosition::register(&mut registry);
        ParentData::register(&mut registry);
        Name::register(&mut registry);

        // Register graphics components
        MeshId::register(&mut registry);
        Material::register(&mut registry);

        // Register scripting components
        ScriptRef::register(&mut registry);
        ScriptProperties::register(&mut registry);

        debug!(
            component_count = registry.len(),
            "Created registry with default components"
        );

        registry
    }
}

impl std::fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field(
                "registered_types",
                &self.deserializers.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl ComponentRegistryExt for ComponentRegistry {
    fn register_with_metadata(&mut self, metadata: ComponentMetadata) {
        let type_id = metadata.type_id;
        let name = metadata.name.to_string();
        let has_ui_metadata = metadata.ui_metadata.is_some();
        let ui_field_count = metadata
            .ui_metadata
            .as_ref()
            .map(|m| m.fields.len())
            .unwrap_or(0);

        // Store the deserializer function
        self.deserializers
            .insert(name.clone(), metadata.deserializer.clone());

        // Store the metadata
        self.name_to_type.insert(name.clone(), type_id);
        self.metadata.insert(type_id, metadata);

        debug!(
            component_name = %name,
            has_ui_metadata = has_ui_metadata,
            ui_field_count = ui_field_count,
            "Registered component with metadata"
        );
    }

    fn get_metadata(&self, type_id: TypeId) -> Option<&ComponentMetadata> {
        self.metadata.get(&type_id)
    }

    fn get_metadata_by_name(&self, name: &str) -> Option<&ComponentMetadata> {
        self.name_to_type
            .get(name)
            .and_then(|type_id| self.metadata.get(type_id))
    }

    fn iter_metadata(&self) -> impl Iterator<Item = &ComponentMetadata> {
        self.metadata.values()
    }

    fn component_names(&self) -> Vec<&'static str> {
        self.metadata.values().map(|meta| meta.name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestComponent {
        value: i32,
    }

    #[test]
    fn test_component_registry_basic() {
        let mut registry = ComponentRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register::<TestComponent>("TestComponent");
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered("TestComponent"));
        assert!(!registry.is_registered("UnknownComponent"));
    }

    #[test]
    fn test_component_registry_deserialize() {
        let mut registry = ComponentRegistry::new();
        registry.register::<TestComponent>("TestComponent");

        let json_value = serde_json::json!({
            "value": 42
        });

        let result = registry.deserialize_component("TestComponent", &json_value);
        assert!(result.is_ok());

        let component = result.unwrap();
        let test_component = component.downcast_ref::<TestComponent>().unwrap();
        assert_eq!(test_component.value, 42);
    }

    #[test]
    fn test_component_registry_unknown_type() {
        let registry = ComponentRegistry::new();
        let json_value = serde_json::json!({});

        let result = registry.deserialize_component("UnknownType", &json_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_component_registry_default() {
        let registry = ComponentRegistry::with_default_components();
        assert!(!registry.is_empty());
        assert!(registry.is_registered("Transform"));
        assert!(registry.is_registered("GlobalTransform"));
        assert!(registry.is_registered("Camera"));
        assert!(registry.is_registered("Parent"));
        assert!(registry.is_registered("Name"));
        assert!(registry.is_registered("MeshId"));
        assert!(registry.is_registered("Material"));
    }

    #[test]
    fn test_component_registry_registered_types() {
        let mut registry = ComponentRegistry::new();
        registry.register::<TestComponent>("TestComponent");
        registry.register::<TestComponent>("AnotherComponent");

        let types: Vec<&str> = registry.registered_types().collect();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"TestComponent"));
        assert!(types.contains(&"AnotherComponent"));
    }

    #[test]
    fn test_component_registry_with_metadata() {
        use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt};

        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        struct MetadataTestComponent {
            value: String,
        }

        impl Component for MetadataTestComponent {
            fn component_name() -> &'static str {
                "MetadataTestComponent"
            }

            fn register(registry: &mut ComponentRegistry) {
                let metadata = ComponentMetadata::new::<Self>(Self::component_name());
                registry.register_with_metadata(metadata);
            }
        }

        let mut registry = ComponentRegistry::new();
        MetadataTestComponent::register(&mut registry);

        // Check metadata is stored
        let metadata = registry
            .get_metadata_by_name("MetadataTestComponent")
            .unwrap();
        assert_eq!(metadata.name, "MetadataTestComponent");

        // Test serialization through metadata
        let component = MetadataTestComponent {
            value: "test value".to_string(),
        };
        let serialized = (metadata.serializer)(&component).unwrap();
        assert_eq!(serialized["value"], "test value");

        // Test deserialization through metadata
        let json = serde_json::json!({ "value": "deserialized value" });
        let deserialized = (metadata.deserializer)(&json).unwrap();
        let result = deserialized
            .downcast_ref::<MetadataTestComponent>()
            .unwrap();
        assert_eq!(result.value, "deserialized value");
    }

    #[test]
    fn test_registry_metadata_iteration() {
        use crate::component_system::ComponentRegistryExt;

        // Use the default components registry which uses the new system
        let registry = ComponentRegistry::with_default_components();

        // Count metadata entries
        let metadata_count = registry.iter_metadata().count();
        assert!(metadata_count > 0);

        // Check component names
        let names = registry.component_names();
        assert!(names.contains(&"Transform"));
        assert!(names.contains(&"Camera"));
        assert!(names.contains(&"Material"));
    }
}
