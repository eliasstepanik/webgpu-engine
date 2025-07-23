//! Modular component system with automatic registration and UI generation

use crate::io::component_registry::ComponentRegistry;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::sync::Arc;

pub mod field_access;
pub mod ui_metadata;
use ui_metadata::ComponentUIMetadata;

/// Type alias for UI builder function
pub type UIBuilderFn =
    Arc<dyn Fn(&mut crate::core::entity::World, hecs::Entity, &mut dyn Any) -> bool + Send + Sync>;

/// Type alias for component serializer function  
pub type SerializerFn = Arc<
    dyn Fn(&dyn Any) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

/// Type alias for component deserializer function
pub type DeserializerFn = Arc<
    dyn Fn(&serde_json::Value) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

/// Type alias for add default component function
pub type AddDefaultFn = Arc<
    dyn Fn(
            &mut crate::core::entity::World,
            hecs::Entity,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

/// Trait for components that can be automatically registered and managed
pub trait Component: Any + Send + Sync + 'static {
    /// Get the name of this component type
    fn component_name() -> &'static str
    where
        Self: Sized;

    /// Register this component type with the registry
    fn register(registry: &mut ComponentRegistry)
    where
        Self: Sized;
}

/// Trait for components that can generate their own editor UI
pub trait EditorUI: Component {
    /// Build the editor UI for this component
    /// Returns true if the component was modified
    /// The ui parameter is a type-erased pointer to the actual UI type
    fn build_ui(component: &mut Self, ui: &mut dyn Any, entity: hecs::Entity) -> bool
    where
        Self: Sized;

    /// Get the UI metadata for this component type
    ///
    /// This is used by the editor to generate UI based on field attributes
    fn ui_metadata() -> Option<ComponentUIMetadata>
    where
        Self: Sized,
    {
        None
    }
}

/// Metadata for a component type including UI builder and serialization functions
pub struct ComponentMetadata {
    /// The display name of the component
    pub name: &'static str,

    /// The TypeId of the component
    pub type_id: TypeId,

    /// Function to build the UI for this component
    /// Returns true if the component was modified
    /// The ui parameter is a type-erased pointer to the actual UI type (imgui::Ui in editor)
    pub ui_builder: Option<UIBuilderFn>,

    /// UI metadata for automatic UI generation
    pub ui_metadata: Option<ComponentUIMetadata>,

    /// Function to serialize the component to JSON
    pub serializer: SerializerFn,

    /// Function to deserialize the component from JSON
    pub deserializer: DeserializerFn,

    /// Function to add a default instance of this component to an entity
    pub add_default: AddDefaultFn,
}

impl ComponentMetadata {
    /// Create metadata for a component type that implements Serialize, Deserialize, and Default
    pub fn new<T>(name: &'static str) -> Self
    where
        T: Component + Serialize + for<'de> Deserialize<'de> + Default + 'static,
    {
        Self {
            name,
            type_id: TypeId::of::<T>(),
            ui_builder: None,
            ui_metadata: None,
            serializer: Arc::new(|component| {
                let typed_component = component
                    .downcast_ref::<T>()
                    .ok_or("Failed to downcast component for serialization")?;
                serde_json::to_value(typed_component)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }),
            deserializer: Arc::new(|value| {
                let component: T = serde_json::from_value(value.clone())
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(Box::new(component) as Box<dyn Any>)
            }),
            add_default: Arc::new(|world, entity| {
                world
                    .insert_one(entity, T::default())
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }),
        }
    }

    /// Create metadata for a component type that also implements EditorUI
    pub fn new_with_ui<T>(name: &'static str) -> Self
    where
        T: Component + EditorUI + Serialize + for<'de> Deserialize<'de> + Default + 'static,
    {
        let mut metadata = Self::new::<T>(name);

        // Add UI builder that uses the EditorUI trait
        metadata.ui_builder = Some(Arc::new(move |world, entity, ui| {
            // We need to temporarily remove the component to get mutable access
            if let Ok(mut component) = world.inner_mut().remove_one::<T>(entity) {
                let modified = T::build_ui(&mut component, ui, entity);

                // Always re-insert the component
                let _ = world.insert_one(entity, component);

                modified
            } else {
                false
            }
        }));

        // Get UI metadata from the component type
        metadata.ui_metadata = T::ui_metadata();

        tracing::debug!(
            component_name = name,
            has_ui_metadata = metadata.ui_metadata.is_some(),
            ui_field_count = metadata
                .ui_metadata
                .as_ref()
                .map(|m| m.fields.len())
                .unwrap_or(0),
            "Created component metadata with UI"
        );

        metadata
    }
}

/// Extension trait for ComponentRegistry to support metadata
pub trait ComponentRegistryExt {
    /// Register a component with full metadata
    fn register_with_metadata(&mut self, metadata: ComponentMetadata);

    /// Get metadata for a component type
    fn get_metadata(&self, type_id: TypeId) -> Option<&ComponentMetadata>;

    /// Get metadata for a component by name
    fn get_metadata_by_name(&self, name: &str) -> Option<&ComponentMetadata>;

    /// Iterate over all registered component metadata
    fn iter_metadata(&self) -> impl Iterator<Item = &ComponentMetadata>;

    /// Get a list of all registered component names
    fn component_names(&self) -> Vec<&'static str>;
}

#[cfg(test)]
mod tests;
