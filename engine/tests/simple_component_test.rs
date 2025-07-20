//! Simple test to validate the component system works

use engine::io::component_registry::ComponentRegistry;
use engine::core::entity::components::Transform;
use engine::component_system::Component;

#[test]
fn test_transform_component_registration() {
    let mut registry = ComponentRegistry::new();
    
    // Register Transform using the Component trait
    Transform::register(&mut registry);
    
    // Verify it's registered
    assert!(registry.is_registered("Transform"));
    
    // Verify we can create a default components registry
    let default_registry = ComponentRegistry::with_default_components();
    assert!(default_registry.is_registered("Transform"));
    assert!(default_registry.is_registered("Camera"));
    assert!(default_registry.is_registered("Material"));
    assert!(default_registry.is_registered("MeshId"));
    
    println!("Component system is working correctly!");
}