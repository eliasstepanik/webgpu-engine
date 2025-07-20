//! Field access trait for UI metadata-driven rendering
//!
//! This module provides traits and types for accessing component fields dynamically
//! to support metadata-driven UI rendering.

use glam::{Quat, Vec3};

/// A value that can be displayed and edited in the UI
#[derive(Debug, Clone)]
pub enum FieldValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Vec3(Vec3),
    Quat(Quat),
    ColorRGB([f32; 3]),
    ColorRGBA([f32; 4]),
    /// For types that don't map to a standard UI widget
    Unsupported,
}

/// Trait for components that support field access for UI rendering
pub trait FieldAccess {
    /// Get a field value by name
    fn get_field(&self, field_name: &str) -> Option<FieldValue>;
    
    /// Set a field value by name
    /// Returns true if the field was successfully set
    fn set_field(&mut self, field_name: &str, value: FieldValue) -> bool;
}

impl FieldValue {
    /// Try to get as f32
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            FieldValue::Float(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as i32
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            FieldValue::Int(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Bool(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as String
    pub fn as_string(&self) -> Option<&String> {
        match self {
            FieldValue::String(v) => Some(v),
            _ => None,
        }
    }
    
    /// Try to get as Vec3
    pub fn as_vec3(&self) -> Option<Vec3> {
        match self {
            FieldValue::Vec3(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as Quat
    pub fn as_quat(&self) -> Option<Quat> {
        match self {
            FieldValue::Quat(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as RGB color
    pub fn as_color_rgb(&self) -> Option<[f32; 3]> {
        match self {
            FieldValue::ColorRGB(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as RGBA color
    pub fn as_color_rgba(&self) -> Option<[f32; 4]> {
        match self {
            FieldValue::ColorRGBA(v) => Some(*v),
            _ => None,
        }
    }
}