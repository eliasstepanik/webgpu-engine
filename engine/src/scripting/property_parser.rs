//! Parser for script property definitions
//!
//! This module parses property declarations from script comments in the format:
//! `//! @property name: type = default_value`

use crate::scripting::property_types::{
    PropertyDefinition, PropertyMetadata, PropertyType, PropertyValue,
};
use std::error::Error;
use std::fmt;
use tracing::debug;

/// Error type for property parsing
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line_number: Option<usize>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line_number {
            write!(f, "Parse error at line {}: {}", line, self.message)
        } else {
            write!(f, "Parse error: {}", self.message)
        }
    }
}

impl Error for ParseError {}

/// Parse property definitions from script content
pub fn parse_script_properties(
    script_content: &str,
) -> Result<Vec<PropertyDefinition>, ParseError> {
    let mut properties = Vec::new();

    for (line_num, line) in script_content.lines().enumerate() {
        // Look for property definition lines
        if let Some(prop_def) = line.strip_prefix("//! @property ") {
            match parse_property_line(prop_def, line_num + 1) {
                Ok(definition) => {
                    debug!(
                        name = definition.name,
                        property_type = ?definition.property_type,
                        "Parsed property definition"
                    );
                    properties.push(definition);
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(properties)
}

/// Parse a single property definition line
fn parse_property_line(line: &str, line_number: usize) -> Result<PropertyDefinition, ParseError> {
    // Expected format: "name: type = default_value"
    // Optional metadata: "@range(min, max) @step(value) @tooltip(text)"

    let line = line.trim();

    // Split by '=' to separate declaration from default value
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(ParseError {
            message: "Property definition must include '= default_value'".to_string(),
            line_number: Some(line_number),
        });
    }

    let declaration = parts[0].trim();
    let default_and_metadata = parts[1].trim();

    // Parse name and type from declaration
    let decl_parts: Vec<&str> = declaration.split(':').collect();
    if decl_parts.len() != 2 {
        return Err(ParseError {
            message: "Property declaration must be in format 'name: type'".to_string(),
            line_number: Some(line_number),
        });
    }

    let name = decl_parts[0].trim().to_string();
    let type_str = decl_parts[1].trim();

    // Validate property name (must start with letter or underscore, then alphanumeric or underscore)
    if name.is_empty() {
        return Err(ParseError {
            message: "Property name cannot be empty".to_string(),
            line_number: Some(line_number),
        });
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return Err(ParseError {
            message: format!(
                "Invalid property name: '{name}' (must start with letter or underscore)"
            ),
            line_number: Some(line_number),
        });
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ParseError {
            message: format!("Invalid property name: '{name}' (must contain only letters, numbers, and underscores)"),
            line_number: Some(line_number),
        });
    }

    // Parse property type
    let property_type = PropertyType::parse(type_str).ok_or_else(|| ParseError {
        message: format!("Unknown property type: '{type_str}'"),
        line_number: Some(line_number),
    })?;

    // Parse default value and optional metadata
    let (default_str, metadata) = parse_value_and_metadata(default_and_metadata);

    debug!(
        default_and_metadata = default_and_metadata,
        default_str = default_str,
        "Parsed value and metadata"
    );

    // Parse default value based on type
    let default_value =
        parse_default_value(default_str, property_type).map_err(|msg| ParseError {
            message: msg,
            line_number: Some(line_number),
        })?;

    Ok(PropertyDefinition {
        name,
        property_type,
        default_value,
        metadata,
    })
}

/// Parse default value and extract metadata annotations
fn parse_value_and_metadata(input: &str) -> (&str, PropertyMetadata) {
    let mut metadata = PropertyMetadata::default();
    let mut value_str = input;

    // Find the first @ that's not inside a string to separate value from metadata
    let chars: Vec<char> = input.chars().collect();
    let mut in_string = false;
    let mut escape_next = false;
    let mut first_at_pos = None;

    for (i, &ch) in chars.iter().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '@' if !in_string => {
                first_at_pos = Some(i);
                break;
            }
            _ => {}
        }
    }

    if let Some(at_pos) = first_at_pos {
        value_str = &input[..at_pos];
        let metadata_str = &input[at_pos..];
        metadata = parse_metadata(metadata_str);
    }

    (value_str.trim(), metadata)
}

/// Parse metadata annotations like @range(0, 10) @step(0.1) @tooltip("text")
fn parse_metadata(metadata_str: &str) -> PropertyMetadata {
    let mut metadata = PropertyMetadata::default();

    // Find all @annotation(value) patterns
    let mut current_pos = 0;
    while let Some(at_pos) = metadata_str[current_pos..].find('@') {
        let start = current_pos + at_pos;

        // Find the annotation name
        let remaining = &metadata_str[start + 1..];
        if let Some(paren_pos) = remaining.find('(') {
            let annotation = &remaining[..paren_pos];

            // Find matching closing parenthesis
            let content_start = start + 1 + paren_pos + 1;
            if let Some(close_pos) = find_matching_paren(&metadata_str[content_start..]) {
                let content = &metadata_str[content_start..content_start + close_pos];

                match annotation {
                    "range" => {
                        // Parse range(min, max)
                        let parts: Vec<&str> = content.split(',').collect();
                        if parts.len() == 2 {
                            if let (Ok(min), Ok(max)) = (
                                parts[0].trim().parse::<f32>(),
                                parts[1].trim().parse::<f32>(),
                            ) {
                                metadata.min = Some(min);
                                metadata.max = Some(max);
                            }
                        }
                    }
                    "step" => {
                        // Parse step(value)
                        if let Ok(step) = content.trim().parse::<f32>() {
                            metadata.step = Some(step);
                        }
                    }
                    "tooltip" => {
                        // Parse tooltip("text")
                        let trimmed = content.trim();
                        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 2 {
                            metadata.tooltip = Some(trimmed[1..trimmed.len() - 1].to_string());
                        }
                    }
                    _ => {} // Ignore unknown annotations
                }

                current_pos = content_start + close_pos + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    metadata
}

/// Find the position of the matching closing parenthesis
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in s.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '(' if !in_string => depth += 1,
            ')' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

/// Parse a default value string based on the expected type
fn parse_default_value(
    value_str: &str,
    property_type: PropertyType,
) -> Result<PropertyValue, String> {
    let value_str = value_str.trim();

    match property_type {
        PropertyType::Float => value_str
            .parse::<f32>()
            .map(PropertyValue::Float)
            .map_err(|_| format!("Invalid float value: '{value_str}'")),

        PropertyType::Integer => value_str
            .parse::<i32>()
            .map(PropertyValue::Integer)
            .map_err(|_| format!("Invalid integer value: '{value_str}'")),

        PropertyType::Boolean => match value_str.to_lowercase().as_str() {
            "true" => Ok(PropertyValue::Boolean(true)),
            "false" => Ok(PropertyValue::Boolean(false)),
            _ => Err(format!("Invalid boolean value: '{value_str}'")),
        },

        PropertyType::String => {
            // Handle quoted strings
            if value_str.starts_with('"') && value_str.ends_with('"') && value_str.len() >= 2 {
                Ok(PropertyValue::String(
                    value_str[1..value_str.len() - 1].to_string(),
                ))
            } else {
                Ok(PropertyValue::String(value_str.to_string()))
            }
        }

        PropertyType::Vector3 => {
            // Parse array notation: [x, y, z]
            let trimmed = value_str.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let inner = &trimmed[1..trimmed.len() - 1];
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 3 {
                    let x = parts[0]
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid x component: '{}'", parts[0]))?;
                    let y = parts[1]
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid y component: '{}'", parts[1]))?;
                    let z = parts[2]
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid z component: '{}'", parts[2]))?;
                    Ok(PropertyValue::Vector3([x, y, z]))
                } else {
                    Err(format!(
                        "Vector3 must have exactly 3 components, found {}",
                        parts.len()
                    ))
                }
            } else {
                Err("Vector3 must be in format [x, y, z]".to_string())
            }
        }

        PropertyType::Color => {
            // Parse array notation: [r, g, b] or [r, g, b, a]
            let trimmed = value_str.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let inner = &trimmed[1..trimmed.len() - 1];
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 3 || parts.len() == 4 {
                    let r = parts[0]
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid red component: '{}'", parts[0]))?;
                    let g = parts[1]
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid green component: '{}'", parts[1]))?;
                    let b = parts[2]
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| format!("Invalid blue component: '{}'", parts[2]))?;
                    let a = if parts.len() == 4 {
                        parts[3]
                            .trim()
                            .parse::<f32>()
                            .map_err(|_| format!("Invalid alpha component: '{}'", parts[3]))?
                    } else {
                        1.0
                    };
                    Ok(PropertyValue::Color([r, g, b, a]))
                } else {
                    Err(format!(
                        "Color must have 3 or 4 components, found {}",
                        parts.len()
                    ))
                }
            } else {
                Err("Color must be in format [r, g, b] or [r, g, b, a]".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_properties() {
        let script = r#"
//! @property speed: float = 1.5
//! @property count: int = 10
//! @property enabled: bool = true
//! @property name: string = "test"
fn on_update(entity, dt) {}
"#;

        let props = parse_script_properties(script).unwrap();
        assert_eq!(props.len(), 4);

        assert_eq!(props[0].name, "speed");
        assert_eq!(props[0].property_type, PropertyType::Float);
        assert_eq!(props[0].default_value, PropertyValue::Float(1.5));

        assert_eq!(props[1].name, "count");
        assert_eq!(props[1].property_type, PropertyType::Integer);
        assert_eq!(props[1].default_value, PropertyValue::Integer(10));

        assert_eq!(props[2].name, "enabled");
        assert_eq!(props[2].property_type, PropertyType::Boolean);
        assert_eq!(props[2].default_value, PropertyValue::Boolean(true));

        assert_eq!(props[3].name, "name");
        assert_eq!(props[3].property_type, PropertyType::String);
        assert_eq!(
            props[3].default_value,
            PropertyValue::String("test".to_string())
        );
    }

    #[test]
    fn test_parse_vector_and_color() {
        let script = r#"
//! @property position: vec3 = [1.0, 2.0, 3.0]
//! @property color: color = [0.5, 0.5, 0.5, 1.0]
//! @property tint: color = [1.0, 0.0, 0.0]
"#;

        let props = parse_script_properties(script).unwrap();
        assert_eq!(props.len(), 3);

        assert_eq!(props[0].name, "position");
        assert_eq!(props[0].property_type, PropertyType::Vector3);
        assert_eq!(
            props[0].default_value,
            PropertyValue::Vector3([1.0, 2.0, 3.0])
        );

        assert_eq!(props[1].name, "color");
        assert_eq!(props[1].property_type, PropertyType::Color);
        assert_eq!(
            props[1].default_value,
            PropertyValue::Color([0.5, 0.5, 0.5, 1.0])
        );

        assert_eq!(props[2].name, "tint");
        assert_eq!(
            props[2].default_value,
            PropertyValue::Color([1.0, 0.0, 0.0, 1.0])
        );
    }

    #[test]
    fn test_parse_with_metadata() {
        let script = r#"
//! @property speed: float = 1.0 @range(0, 10) @step(0.1)
//! @property name: string = "hello" @tooltip("Enter a name")
"#;

        let props = parse_script_properties(script).unwrap();
        assert_eq!(props.len(), 2);

        assert_eq!(props[0].metadata.min, Some(0.0));
        assert_eq!(props[0].metadata.max, Some(10.0));
        assert_eq!(props[0].metadata.step, Some(0.1));

        assert_eq!(props[1].metadata.tooltip, Some("Enter a name".to_string()));
    }

    #[test]
    fn test_invalid_property_definitions() {
        let script = "//! @property invalid";
        assert!(parse_script_properties(script).is_err());

        let script = "//! @property name: unknown_type = 1.0";
        assert!(parse_script_properties(script).is_err());

        let script = "//! @property 123invalid: float = 1.0";
        assert!(parse_script_properties(script).is_err());
    }
}
