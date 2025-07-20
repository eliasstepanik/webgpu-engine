//! Property type definitions for script parameters
//!
//! This module provides types for defining and storing script properties that can be
//! configured per-entity in the editor and accessed at runtime by scripts.

use crate::component_system::{Component, ComponentMetadata, ComponentRegistryExt, EditorUI};
use crate::io::component_registry::ComponentRegistry;
use rhai::Dynamic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A typed property value that can be edited in the inspector and passed to scripts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum PropertyValue {
    /// Floating point number
    Float(f32),
    /// Integer number
    Integer(i32),
    /// Boolean value
    Boolean(bool),
    /// String value
    String(String),
    /// 3D vector (x, y, z)
    Vector3([f32; 3]),
    /// RGBA color (r, g, b, a) in range 0.0-1.0
    Color([f32; 4]),
}

impl PropertyValue {
    /// Convert to Rhai Dynamic type for script access
    pub fn to_dynamic(&self) -> Dynamic {
        match self {
            PropertyValue::Float(v) => Dynamic::from(*v as f64),
            PropertyValue::Integer(v) => Dynamic::from(*v as i64),
            PropertyValue::Boolean(v) => Dynamic::from(*v),
            PropertyValue::String(v) => Dynamic::from(v.clone()),
            PropertyValue::Vector3(v) => {
                let mut map = rhai::Map::new();
                map.insert("x".into(), Dynamic::from(v[0] as f64));
                map.insert("y".into(), Dynamic::from(v[1] as f64));
                map.insert("z".into(), Dynamic::from(v[2] as f64));
                Dynamic::from(map)
            }
            PropertyValue::Color(v) => {
                let mut map = rhai::Map::new();
                map.insert("r".into(), Dynamic::from(v[0] as f64));
                map.insert("g".into(), Dynamic::from(v[1] as f64));
                map.insert("b".into(), Dynamic::from(v[2] as f64));
                map.insert("a".into(), Dynamic::from(v[3] as f64));
                Dynamic::from(map)
            }
        }
    }

    /// Try to create a PropertyValue from a Rhai Dynamic
    pub fn from_dynamic(value: &Dynamic, expected_type: PropertyType) -> Option<Self> {
        match expected_type {
            PropertyType::Float => value
                .as_float()
                .ok()
                .map(|f| PropertyValue::Float(f as f32)),
            PropertyType::Integer => value
                .as_int()
                .ok()
                .map(|i| PropertyValue::Integer(i as i32)),
            PropertyType::Boolean => value.as_bool().ok().map(PropertyValue::Boolean),
            PropertyType::String => value.clone().into_string().ok().map(PropertyValue::String),
            PropertyType::Vector3 => {
                // Try to extract a map with x, y, z fields
                if let Some(map) = value.read_lock::<rhai::Map>() {
                    let x = map.get("x").and_then(|v| v.as_float().ok())? as f32;
                    let y = map.get("y").and_then(|v| v.as_float().ok())? as f32;
                    let z = map.get("z").and_then(|v| v.as_float().ok())? as f32;
                    Some(PropertyValue::Vector3([x, y, z]))
                } else {
                    None
                }
            }
            PropertyType::Color => {
                // Try to extract a map with r, g, b, a fields
                if let Some(map) = value.read_lock::<rhai::Map>() {
                    let r = map.get("r").and_then(|v| v.as_float().ok())? as f32;
                    let g = map.get("g").and_then(|v| v.as_float().ok())? as f32;
                    let b = map.get("b").and_then(|v| v.as_float().ok())? as f32;
                    let a = map.get("a").and_then(|v| v.as_float().ok()).unwrap_or(1.0) as f32;
                    Some(PropertyValue::Color([r, g, b, a]))
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Float(v) => write!(f, "{v}"),
            PropertyValue::Integer(v) => write!(f, "{v}"),
            PropertyValue::Boolean(v) => write!(f, "{v}"),
            PropertyValue::String(v) => write!(f, "\"{v}\""),
            PropertyValue::Vector3(v) => write!(f, "({}, {}, {})", v[0], v[1], v[2]),
            PropertyValue::Color(v) => write!(f, "rgba({}, {}, {}, {})", v[0], v[1], v[2], v[3]),
        }
    }
}

/// Definition of a property that can be declared in a script
#[derive(Debug, Clone)]
pub struct PropertyDefinition {
    /// Name of the property
    pub name: String,
    /// Type of the property
    pub property_type: PropertyType,
    /// Default value
    pub default_value: PropertyValue,
    /// Additional metadata for editor UI
    pub metadata: PropertyMetadata,
}

/// The type of a property value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyType {
    Float,
    Integer,
    Boolean,
    String,
    Vector3,
    Color,
}

impl PropertyType {
    /// Parse a type string from script comment
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "float" | "f32" | "number" => Some(PropertyType::Float),
            "int" | "i32" | "integer" => Some(PropertyType::Integer),
            "bool" | "boolean" => Some(PropertyType::Boolean),
            "string" | "str" => Some(PropertyType::String),
            "vec3" | "vector3" => Some(PropertyType::Vector3),
            "color" | "rgba" | "rgb" => Some(PropertyType::Color),
            _ => None,
        }
    }
}

/// Metadata for property UI in the editor
#[derive(Debug, Clone, Default)]
pub struct PropertyMetadata {
    /// Minimum value (for numeric types)
    pub min: Option<f32>,
    /// Maximum value (for numeric types)
    pub max: Option<f32>,
    /// Step size for drag widgets
    pub step: Option<f32>,
    /// Tooltip to show in editor
    pub tooltip: Option<String>,
}

/// Component that stores script property values per entity
#[derive(Debug, Clone, Serialize, Deserialize, Default, engine_derive::Component, engine_derive::EditorUI)]
#[component(name = "ScriptProperties")]
pub struct ScriptProperties {
    /// Map of property name to value
    pub values: HashMap<String, PropertyValue>,
    /// The script this properties component was created for (used to detect script changes)
    #[serde(default)]
    pub script_name: Option<String>,
}

impl ScriptProperties {
    /// Create new empty properties
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            script_name: None,
        }
    }

    /// Create properties from definitions with default values
    pub fn from_definitions(definitions: &[PropertyDefinition]) -> Self {
        let mut values = HashMap::new();
        for def in definitions {
            values.insert(def.name.clone(), def.default_value.clone());
        }
        Self {
            values,
            script_name: None,
        }
    }

    /// Create properties for a specific script
    pub fn from_definitions_for_script(
        definitions: &[PropertyDefinition],
        script_name: &str,
    ) -> Self {
        let mut values = HashMap::new();
        for def in definitions {
            values.insert(def.name.clone(), def.default_value.clone());
        }
        Self {
            values,
            script_name: Some(script_name.to_string()),
        }
    }

    /// Convert all properties to a Rhai map for script access
    pub fn to_rhai_map(&self) -> rhai::Map {
        let mut map = rhai::Map::new();
        for (name, value) in &self.values {
            map.insert(name.clone().into(), value.to_dynamic());
        }
        map
    }

    /// Update a property value from a Rhai Dynamic
    pub fn update_from_dynamic(
        &mut self,
        name: &str,
        value: &Dynamic,
        expected_type: PropertyType,
    ) -> Result<(), String> {
        if let Some(prop_value) = PropertyValue::from_dynamic(value, expected_type) {
            self.values.insert(name.to_string(), prop_value);
            Ok(())
        } else {
            Err(format!(
                "Failed to convert dynamic value to {expected_type:?}"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_value_to_dynamic() {
        let float_val = PropertyValue::Float(3.5);
        let dynamic = float_val.to_dynamic();
        assert!((dynamic.as_float().unwrap() - 3.5).abs() < f64::EPSILON);

        let vec_val = PropertyValue::Vector3([1.0, 2.0, 3.0]);
        let dynamic = vec_val.to_dynamic();
        let map = dynamic.read_lock::<rhai::Map>().unwrap();
        assert_eq!(map.get("x").unwrap().as_float().unwrap(), 1.0);
        assert_eq!(map.get("y").unwrap().as_float().unwrap(), 2.0);
        assert_eq!(map.get("z").unwrap().as_float().unwrap(), 3.0);
    }

    #[test]
    fn test_property_type_parsing() {
        assert_eq!(PropertyType::parse("float"), Some(PropertyType::Float));
        assert_eq!(PropertyType::parse("vec3"), Some(PropertyType::Vector3));
        assert_eq!(PropertyType::parse("color"), Some(PropertyType::Color));
        assert_eq!(PropertyType::parse("unknown"), None);
    }

    #[test]
    fn test_script_properties_serialization() {
        let mut props = ScriptProperties::new();
        props
            .values
            .insert("speed".to_string(), PropertyValue::Float(1.5));
        props
            .values
            .insert("enabled".to_string(), PropertyValue::Boolean(true));

        let json = serde_json::to_string(&props).unwrap();
        let decoded: ScriptProperties = serde_json::from_str(&json).unwrap();

        assert_eq!(props.values.len(), decoded.values.len());
        assert_eq!(props.values.get("speed"), decoded.values.get("speed"));
    }
}
