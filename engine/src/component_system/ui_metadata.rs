//! UI metadata for components that can be used by the editor
//!
//! This module defines the metadata structure that the derive macro
//! generates and the editor uses to create UI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a single field's UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIFieldMetadata {
    /// Field name
    pub name: String,

    /// Display label (defaults to field name)
    pub label: Option<String>,

    /// Widget type
    pub widget: UIWidgetType,

    /// Tooltip text
    pub tooltip: Option<String>,

    /// Whether this field is hidden from UI
    pub hidden: bool,

    /// Whether this field is readonly
    pub readonly: bool,

    /// Additional widget-specific properties
    pub properties: HashMap<String, UIPropertyValue>,
}

/// Types of UI widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIWidgetType {
    /// Drag input for floating point numbers
    DragFloat {
        min: f32,
        max: f32,
        speed: f32,
        format: String,
    },

    /// Drag input for integers
    DragInt {
        min: i32,
        max: i32,
        speed: f32,
        format: String,
    },

    /// Text input
    InputText {
        multiline: bool,
        max_length: Option<usize>,
    },

    /// Checkbox for boolean values
    Checkbox,

    /// Color picker
    ColorEdit { alpha: bool },

    /// 3D vector input (3 drag floats)
    Vec3Input { speed: f32, format: String },

    /// Quaternion input (euler angles)
    QuatInput { speed: f32, format: String },

    /// Custom widget (function name)
    Custom { function: String },
}

/// Property values for UI metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIPropertyValue {
    String(String),
    Float(f32),
    Int(i32),
    Bool(bool),
}

impl UIPropertyValue {
    /// Get as string reference
    pub fn as_str(&self) -> Option<&str> {
        match self {
            UIPropertyValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// Complete UI metadata for a component
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentUIMetadata {
    /// Metadata for each field
    pub fields: Vec<UIFieldMetadata>,
}

impl ComponentUIMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Add field metadata
    pub fn add_field(&mut self, field: UIFieldMetadata) {
        self.fields.push(field);
    }
}
