// Simple rotating cube script with configurable speed
// This is a minimal example that demonstrates the property system
//
//! @property rotation_speed: float = 1.0 @range(-5.0, 5.0) @step(0.1) @tooltip("Rotation speed in radians per second")

fn on_start(entity) {
    print("Simple rotating cube script started for entity " + entity);
}

fn on_update(entity, delta_time) {
    // Get rotation speed from properties
    let speed = properties["rotation_speed"];
    
    // Get the transform component
    let transform = world::get_component(entity, "Transform");
    
    if transform != () {
        // Rotate around Y axis
        transform.rotate_y(speed * delta_time);
        
        // Update the transform
        world::set_component(entity, "Transform", transform);
    }
}

fn on_destroy(entity) {
    print("Simple rotating cube script stopped for entity " + entity);
}