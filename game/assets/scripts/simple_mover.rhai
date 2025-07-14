// Simple movement script with configurable parameters
// Demonstrates the script property system
//
//! @property move_speed: float = 2.0 @range(0.0, 10.0) @step(0.1) @tooltip("Movement speed in units per second")
//! @property move_direction: vec3 = [1.0, 0.0, 0.0] @tooltip("Direction to move in")
//! @property oscillate: bool = true @tooltip("Enable oscillating movement")
//! @property oscillate_amplitude: float = 2.0 @range(0.0, 5.0) @step(0.1) @tooltip("Oscillation distance")
//! @property oscillate_frequency: float = 1.0 @range(0.1, 5.0) @step(0.1) @tooltip("Oscillation speed")

fn on_start(entity) {
    print("Simple mover script started for entity " + entity);
}

fn on_update(entity, delta_time) {
    
    // Get parameters from properties
    let speed = properties["move_speed"];
    let direction = properties["move_direction"];
    let oscillate = properties["oscillate"];
    let amplitude = properties["oscillate_amplitude"];
    let frequency = properties["oscillate_frequency"];
    
    // Get current transform
    let transform = world::get_component(entity, "Transform");
    if transform != () {
        if oscillate {
            // Note: Without persistent state, true oscillation is not possible
            // For now, we'll just move back and forth based on position
            print("Note: Oscillation requires persistent state - using linear movement instead");
        }
        
        // Linear movement
        transform.position.x += direction["x"] * speed * delta_time;
        transform.position.y += direction["y"] * speed * delta_time;
        transform.position.z += direction["z"] * speed * delta_time;
        
        // Update the transform
        world::set_component(entity, "Transform", transform);
    }
}

fn on_destroy(entity) {
    print("Simple mover script stopped for entity " + entity);
}