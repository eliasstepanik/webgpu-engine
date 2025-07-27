// Simple rotating cube script with configurable parameters
// Demonstrates basic component access and transformation with properties
//
// NOTE: Component modifications are queued and applied after all scripts finish executing.
// This ensures thread safety and prevents race conditions between scripts.
//
//! @property rotation_speed: float = 1.0 @range(-5.0, 5.0) @step(0.1) @tooltip("Rotation speed in radians per second")
//! @property rotation_axis: vec3 = [0.0, 1.0, 0.0] @tooltip("Axis to rotate around")
//! @property scale_factor: float = 1.0 @range(0.1, 3.0) @step(0.1) @tooltip("Scale multiplier")
//! @property tint_color: color = [1.0, 1.0, 1.0, 1.0] @tooltip("Tint color for the material")

fn on_start() {
    print("Rotating cube script started for entity " + entity);
}

fn on_update(delta_time) {
    // Get animation parameters from properties
    let rotation_speed = properties["rotation_speed"];
    let rotation_axis = properties["rotation_axis"];
    let scale_factor = properties["scale_factor"];
    let tint_color = properties["tint_color"];
    
    // Get the transform
    let transform = world::get_component(entity, "Transform");
    
    if transform != () {
        // Apply rotation around the specified axis
        let axis_x = rotation_axis["x"];
        let axis_y = rotation_axis["y"];
        let axis_z = rotation_axis["z"];
        
        // Normalize the axis
        let length = math::sqrt(axis_x * axis_x + axis_y * axis_y + axis_z * axis_z);
        if length > 0.001 {
            let norm_x = axis_x / length;
            let norm_y = axis_y / length;
            let norm_z = axis_z / length;
            
            // Create rotation quaternion
            let angle = rotation_speed * delta_time;
            let half_angle = angle * 0.5;
            let sin_half = math::sin(half_angle);
            
            // Manually construct quaternion components
            let qx = norm_x * sin_half;
            let qy = norm_y * sin_half;
            let qz = norm_z * sin_half;
            let qw = math::cos(half_angle);
            
            // Apply rotation using quaternion multiplication
            // This is a simplified version - in practice you'd use the built-in functions
            if math::abs(norm_y - 1.0) < 0.001 {
                // Special case for Y-axis rotation
                transform.rotate_y(rotation_speed * delta_time);
            } else if math::abs(norm_x - 1.0) < 0.001 {
                // Special case for X-axis rotation
                transform.rotate_x(rotation_speed * delta_time);
            } else if math::abs(norm_z - 1.0) < 0.001 {
                // Special case for Z-axis rotation
                transform.rotate_z(rotation_speed * delta_time);
            }
        }
        
        // Apply scale
        transform.scale.x = scale_factor;
        transform.scale.y = scale_factor;
        transform.scale.z = scale_factor;
        
        // Update the transform
        world::set_component(entity, "Transform", transform);
    }
    
    // Apply tint color to material if it exists
    let material = world::get_component(entity, "Material");
    if material != () {
        material.color[0] = tint_color["r"];
        material.color[1] = tint_color["g"];
        material.color[2] = tint_color["b"];
        material.color[3] = tint_color["a"];
        
        // Update the material
        world::set_component(entity, "Material", material);
    }
}

fn on_destroy() {
    print("Rotating cube script stopped for entity " + entity);
}