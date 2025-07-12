// Simple rotating cube script
// Demonstrates basic component access and transformation
//
// NOTE: Component modifications are queued and applied after all scripts finish executing.
// This ensures thread safety and prevents race conditions between scripts.

fn on_start(entity) {
    print("Rotating cube script started for entity " + entity);
}

fn on_update(entity, delta_time) {
    // Animation parameters
    let rotation_speed = 1.0; // Radians per second
    let bob_speed = 2.0;      // Oscillation frequency
    let bob_amplitude = 0.5;  // Oscillation amplitude
    
    // Get the transform
    let transform = world::get_component(entity, "Transform");
    
    // Rotate around Y axis
    transform.rotate_y(rotation_speed * delta_time);
    
    // For now, we'll skip the bobbing motion since we can't maintain state
    // In a future update, we could store time elapsed in a custom component
    
    // Optional: Change color based on rotation
    // We'll use a simple counter approach
    if world::get_component(entity, "Material") != () {
        // Get current material color and modify it slightly
        let material = world::get_component(entity, "Material");
        // For now, just keep the material as is
        // In a future update, we could implement color cycling with state storage
    }
    
    // Update the transform
    // Note: This change is queued and will be applied after all scripts have finished their on_update calls
    world::set_component(entity, "Transform", transform);
}

fn on_destroy(entity) {
    print("Rotating cube script stopped for entity " + entity);
}

// Helper function to convert HSV to RGB
fn hsv_to_rgb(h, s, v) {
    let c = v * s;
    let x = c * (1.0 - math::abs(((h * 6.0) % 2.0) - 1.0));
    let m = v - c;
    
    let rgb = if h < 1.0/6.0 {
        [c, x, 0.0]
    } else if h < 2.0/6.0 {
        [x, c, 0.0]
    } else if h < 3.0/6.0 {
        [0.0, c, x]
    } else if h < 4.0/6.0 {
        [0.0, x, c]
    } else if h < 5.0/6.0 {
        [x, 0.0, c]
    } else {
        [c, 0.0, x]
    };
    
    [rgb[0] + m, rgb[1] + m, rgb[2] + m]
}