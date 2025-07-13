//! Tests for the script property system

use crate::scripting::property_types::{
    PropertyDefinition, PropertyType, PropertyValue, ScriptProperties,
};

#[test]
fn test_property_value_to_dynamic_and_back() {
    // Test float
    let float_val = PropertyValue::Float(3.5);
    let dynamic = float_val.to_dynamic();
    assert!((dynamic.as_float().unwrap() - 3.5).abs() < f64::EPSILON);
    let converted = PropertyValue::from_dynamic(&dynamic, PropertyType::Float);
    assert_eq!(converted, Some(PropertyValue::Float(3.5)));

    // Test integer
    let int_val = PropertyValue::Integer(42);
    let dynamic = int_val.to_dynamic();
    assert_eq!(dynamic.as_int().unwrap(), 42);
    let converted = PropertyValue::from_dynamic(&dynamic, PropertyType::Integer);
    assert_eq!(converted, Some(PropertyValue::Integer(42)));

    // Test boolean
    let bool_val = PropertyValue::Boolean(true);
    let dynamic = bool_val.to_dynamic();
    assert!(dynamic.as_bool().unwrap());
    let converted = PropertyValue::from_dynamic(&dynamic, PropertyType::Boolean);
    assert_eq!(converted, Some(PropertyValue::Boolean(true)));

    // Test string
    let string_val = PropertyValue::String("hello".to_string());
    let dynamic = string_val.to_dynamic();
    assert_eq!(dynamic.into_string().unwrap(), "hello");

    // Test vector3
    let vec_val = PropertyValue::Vector3([1.0, 2.0, 3.0]);
    let dynamic = vec_val.to_dynamic();
    let map = dynamic.read_lock::<rhai::Map>().unwrap();
    assert!((map.get("x").unwrap().as_float().unwrap() - 1.0).abs() < f64::EPSILON);
    assert!((map.get("y").unwrap().as_float().unwrap() - 2.0).abs() < f64::EPSILON);
    assert!((map.get("z").unwrap().as_float().unwrap() - 3.0).abs() < f64::EPSILON);

    // Test color
    let color_val = PropertyValue::Color([1.0, 0.5, 0.0, 0.8]);
    let dynamic = color_val.to_dynamic();
    let map = dynamic.read_lock::<rhai::Map>().unwrap();
    assert!((map.get("r").unwrap().as_float().unwrap() - 1.0).abs() < f64::EPSILON);
    assert!((map.get("g").unwrap().as_float().unwrap() - 0.5).abs() < f64::EPSILON);
    assert!((map.get("b").unwrap().as_float().unwrap() - 0.0).abs() < f64::EPSILON);
    assert!((map.get("a").unwrap().as_float().unwrap() - 0.8).abs() < 0.0001);
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
    props.values.insert(
        "position".to_string(),
        PropertyValue::Vector3([10.0, 20.0, 30.0]),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&props).unwrap();
    assert!(json.contains("speed"));
    assert!(json.contains("1.5"));
    assert!(json.contains("enabled"));
    assert!(json.contains("true"));

    // Deserialize back
    let decoded: ScriptProperties = serde_json::from_str(&json).unwrap();
    assert_eq!(props.values.len(), decoded.values.len());
    assert_eq!(props.values.get("speed"), decoded.values.get("speed"));
    assert_eq!(props.values.get("enabled"), decoded.values.get("enabled"));
    assert_eq!(props.values.get("position"), decoded.values.get("position"));
}

#[test]
fn test_script_properties_from_definitions() {
    let definitions = vec![
        PropertyDefinition {
            name: "speed".to_string(),
            property_type: PropertyType::Float,
            default_value: PropertyValue::Float(5.0),
            metadata: Default::default(),
        },
        PropertyDefinition {
            name: "active".to_string(),
            property_type: PropertyType::Boolean,
            default_value: PropertyValue::Boolean(false),
            metadata: Default::default(),
        },
    ];

    let props = ScriptProperties::from_definitions(&definitions);
    assert_eq!(props.values.len(), 2);
    assert_eq!(props.values.get("speed"), Some(&PropertyValue::Float(5.0)));
    assert_eq!(
        props.values.get("active"),
        Some(&PropertyValue::Boolean(false))
    );
}

#[test]
fn test_script_properties_to_rhai_map() {
    let mut props = ScriptProperties::new();
    props
        .values
        .insert("test".to_string(), PropertyValue::Float(42.0));
    props.values.insert(
        "name".to_string(),
        PropertyValue::String("example".to_string()),
    );

    let rhai_map = props.to_rhai_map();
    assert_eq!(rhai_map.len(), 2);
    assert!(rhai_map.contains_key("test"));
    assert!(rhai_map.contains_key("name"));
}

#[test]
fn test_property_type_parsing() {
    assert_eq!(PropertyType::parse("float"), Some(PropertyType::Float));
    assert_eq!(PropertyType::parse("f32"), Some(PropertyType::Float));
    assert_eq!(PropertyType::parse("number"), Some(PropertyType::Float));
    assert_eq!(PropertyType::parse("int"), Some(PropertyType::Integer));
    assert_eq!(PropertyType::parse("i32"), Some(PropertyType::Integer));
    assert_eq!(PropertyType::parse("bool"), Some(PropertyType::Boolean));
    assert_eq!(PropertyType::parse("boolean"), Some(PropertyType::Boolean));
    assert_eq!(PropertyType::parse("string"), Some(PropertyType::String));
    assert_eq!(PropertyType::parse("str"), Some(PropertyType::String));
    assert_eq!(PropertyType::parse("vec3"), Some(PropertyType::Vector3));
    assert_eq!(PropertyType::parse("vector3"), Some(PropertyType::Vector3));
    assert_eq!(PropertyType::parse("color"), Some(PropertyType::Color));
    assert_eq!(PropertyType::parse("rgba"), Some(PropertyType::Color));
    assert_eq!(PropertyType::parse("unknown_type"), None);
}

#[test]
fn test_property_value_display() {
    assert_eq!(PropertyValue::Float(3.5).to_string(), "3.5");
    assert_eq!(PropertyValue::Integer(42).to_string(), "42");
    assert_eq!(PropertyValue::Boolean(true).to_string(), "true");
    assert_eq!(
        PropertyValue::String("test".to_string()).to_string(),
        "\"test\""
    );
    assert_eq!(
        PropertyValue::Vector3([1.0, 2.0, 3.0]).to_string(),
        "(1, 2, 3)"
    );
    assert_eq!(
        PropertyValue::Color([1.0, 0.5, 0.0, 1.0]).to_string(),
        "rgba(1, 0.5, 0, 1)"
    );
}

// Integration test with script execution would go here but requires
// the full script engine setup which is more complex

#[test]
fn test_property_value_round_trip_all_types() {
    // Test that all property types can be converted to Dynamic and back
    let test_cases = vec![
        (PropertyValue::Float(123.456), PropertyType::Float),
        (PropertyValue::Integer(-999), PropertyType::Integer),
        (PropertyValue::Boolean(false), PropertyType::Boolean),
        (
            PropertyValue::String("test string".to_string()),
            PropertyType::String,
        ),
        (
            PropertyValue::Vector3([1.1, 2.2, 3.3]),
            PropertyType::Vector3,
        ),
        (
            PropertyValue::Color([0.1, 0.2, 0.3, 0.4]),
            PropertyType::Color,
        ),
    ];

    for (original_value, prop_type) in test_cases {
        let dynamic = original_value.to_dynamic();
        let converted = PropertyValue::from_dynamic(&dynamic, prop_type);
        assert_eq!(
            converted,
            Some(original_value.clone()),
            "Failed to round-trip {original_value:?}"
        );
    }
}

#[test]
fn test_property_value_type_mismatch() {
    // Test that from_dynamic returns None for type mismatches
    let float_dynamic = PropertyValue::Float(1.0).to_dynamic();
    assert_eq!(
        PropertyValue::from_dynamic(&float_dynamic, PropertyType::Boolean),
        None
    );

    let bool_dynamic = PropertyValue::Boolean(true).to_dynamic();
    assert_eq!(
        PropertyValue::from_dynamic(&bool_dynamic, PropertyType::Float),
        None
    );

    let string_dynamic = PropertyValue::String("test".to_string()).to_dynamic();
    assert_eq!(
        PropertyValue::from_dynamic(&string_dynamic, PropertyType::Integer),
        None
    );
}

#[test]
fn test_script_properties_update_from_dynamic() {
    let mut props = ScriptProperties::new();
    props
        .values
        .insert("speed".to_string(), PropertyValue::Float(1.0));
    props
        .values
        .insert("count".to_string(), PropertyValue::Integer(5));

    // Test successful update
    let new_speed = rhai::Dynamic::from(2.5_f64);
    let result = props.update_from_dynamic("speed", &new_speed, PropertyType::Float);
    assert!(result.is_ok());
    assert_eq!(props.values.get("speed"), Some(&PropertyValue::Float(2.5)));

    // Test type mismatch
    let wrong_type = rhai::Dynamic::from("not a float");
    let result = props.update_from_dynamic("speed", &wrong_type, PropertyType::Float);
    assert!(result.is_err());
    assert_eq!(props.values.get("speed"), Some(&PropertyValue::Float(2.5))); // Unchanged
}

#[test]
fn test_property_value_equality() {
    // Test that PropertyValue implements PartialEq correctly
    assert_eq!(PropertyValue::Float(1.0), PropertyValue::Float(1.0));
    assert_ne!(PropertyValue::Float(1.0), PropertyValue::Float(1.1));

    assert_eq!(PropertyValue::Integer(42), PropertyValue::Integer(42));
    assert_ne!(PropertyValue::Integer(42), PropertyValue::Integer(43));

    assert_eq!(PropertyValue::Boolean(true), PropertyValue::Boolean(true));
    assert_ne!(PropertyValue::Boolean(true), PropertyValue::Boolean(false));

    assert_eq!(
        PropertyValue::String("test".to_string()),
        PropertyValue::String("test".to_string())
    );
    assert_ne!(
        PropertyValue::String("test".to_string()),
        PropertyValue::String("other".to_string())
    );

    assert_eq!(
        PropertyValue::Vector3([1.0, 2.0, 3.0]),
        PropertyValue::Vector3([1.0, 2.0, 3.0])
    );
    assert_ne!(
        PropertyValue::Vector3([1.0, 2.0, 3.0]),
        PropertyValue::Vector3([1.0, 2.0, 3.1])
    );

    assert_eq!(
        PropertyValue::Color([0.1, 0.2, 0.3, 0.4]),
        PropertyValue::Color([0.1, 0.2, 0.3, 0.4])
    );
    assert_ne!(
        PropertyValue::Color([0.1, 0.2, 0.3, 0.4]),
        PropertyValue::Color([0.1, 0.2, 0.3, 0.5])
    );
}
