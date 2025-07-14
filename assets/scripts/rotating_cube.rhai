//! Rotating cube script with configurable parameters
//!
//! @property rotation_speed: float = 1.0 @range(-5, 5) @step(0.1)
//! @property rotation_axis: vec3 = [0.0, 1.0, 0.0]
//! @property bob_enabled: bool = true
//! @property bob_amplitude: float = 0.5 @range(0, 2) @step(0.05)
//! @property bob_frequency: float = 2.0 @range(0.1, 10) @step(0.1)

let mut time_elapsed = 0.0;

fn on_start(entity) {
    print("Rotating cube script started for entity: " + entity);
    time_elapsed = 0.0;
}

fn on_update(entity, delta_time) {
    // Access properties
    let speed = properties["rotation_speed"];
    let axis = properties["rotation_axis"];
    let bob_enabled = properties["bob_enabled"];
    let bob_amplitude = properties["bob_amplitude"];
    let bob_frequency = properties["bob_frequency"];
    
    // Update time
    time_elapsed += delta_time;
    
    // Get current transform
    let transform = world::get_component(entity, "Transform");
    if transform != () {
        // Apply rotation around the specified axis
        let rotation_amount = speed * delta_time;
        
        // Create rotation quaternion based on axis
        let quat = math::quat_from_axis_angle(
            axis["x"], 
            axis["y"], 
            axis["z"], 
            rotation_amount
        );
        
        // Apply rotation
        let new_rotation = math::quat_multiply(transform.rotation, quat);
        transform.rotation = new_rotation;
        
        // Apply bobbing motion if enabled
        if bob_enabled {
            let bob_offset = sin(time_elapsed * bob_frequency) * bob_amplitude;
            transform.position[1] = transform.position[1] + bob_offset * delta_time;
        }
        
        // Update the transform
        world::set_component(entity, "Transform", transform);
    }
}

fn on_destroy(entity) {
    print("Rotating cube script destroyed for entity: " + entity);
}