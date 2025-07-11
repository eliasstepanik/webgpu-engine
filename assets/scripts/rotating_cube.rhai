// Simple rotating cube script
// Demonstrates basic component access and transformation
//
// NOTE: Component modifications are queued and applied after all scripts finish executing.
// This ensures thread safety and prevents race conditions between scripts.

let rotation_speed = 1.0; // Radians per second
let bob_speed = 2.0;      // Oscillation frequency
let bob_amplitude = 0.5;  // Oscillation amplitude
let start_y = 0.0;
let time_elapsed = 0.0;

fn on_start(entity) {
    print("Rotating cube script started for entity " + entity);
    
    // Store the initial Y position
    let transform = world::get_component(entity, "Transform");
    start_y = transform.position.y;
}

fn on_update(entity, delta_time) {
    // Update time
    time_elapsed += delta_time;
    
    // Get the transform
    let transform = world::get_component(entity, "Transform");
    
    // Rotate around Y axis
    transform.rotate_y(rotation_speed * delta_time);
    
    // Add bobbing motion
    let bob_offset = math::sin(time_elapsed * bob_speed) * bob_amplitude;
    transform.position.y = start_y + bob_offset;
    
    // Optional: Change color based on rotation
    if world::get_component(entity, "Material") != () {
        let hue = (time_elapsed * 0.5) % 1.0;
        let color = hsv_to_rgb(hue, 0.8, 1.0);
        let material = Material::from_rgb(color[0], color[1], color[2]);
        world::set_component(entity, "Material", material);
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