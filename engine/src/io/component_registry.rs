//! Component registry for dynamic component deserialization

use std::any::Any;
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
}

impl ComponentRegistry {
    /// Create a new empty component registry
    pub fn new() -> Self {
        Self {
            deserializers: HashMap::new(),
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
        use crate::core::camera::Camera;
        use crate::core::entity::components::{GlobalTransform, ParentData, Transform};

        let mut registry = Self::new();

        // Register core components
        registry.register::<Transform>("Transform");
        registry.register::<GlobalTransform>("GlobalTransform");
        registry.register::<Camera>("Camera");
        registry.register::<ParentData>("Parent");

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
}
