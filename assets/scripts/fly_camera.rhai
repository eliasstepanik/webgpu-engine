//! Fly camera controller script
//!
//! @property move_speed: float = 5.0 @range(0.1, 20) @step(0.5) @tooltip("Movement speed in units per second")
//! @property look_sensitivity: float = 0.5 @range(0.1, 2) @step(0.1) @tooltip("Mouse look sensitivity")
//! @property sprint_multiplier: float = 2.0 @range(1, 5) @step(0.5) @tooltip("Speed multiplier when sprinting")
//! @property invert_y: bool = false @tooltip("Invert Y axis for mouse look")
//! @property smoothing: float = 0.1 @range(0, 1) @step(0.05) @tooltip("Movement smoothing factor")

let mut pitch = 0.0;
let mut yaw = 0.0;
let mut velocity = #{x: 0.0, y: 0.0, z: 0.0};

fn on_start(entity) {
    print("Fly camera controller started for entity: " + entity);
    
    // Initialize rotation from current transform
    let transform = world::get_component(entity, "Transform");
    if transform != () {
        // Extract initial euler angles from quaternion
        let euler = math::quat_to_euler(transform.rotation);
        pitch = euler[0];
        yaw = euler[1];
    }
}

fn on_update(entity, delta_time) {
    // Get properties
    let speed = properties["move_speed"];
    let sensitivity = properties["look_sensitivity"];
    let sprint_mult = properties["sprint_multiplier"];
    let invert_y = properties["invert_y"];
    let smoothing = properties["smoothing"];
    
    // Get current transform
    let transform = world::get_component(entity, "Transform");
    if transform == () {
        return;
    }
    
    // Mouse look
    let mouse_delta = input::mouse_delta();
    if mouse_delta[0] != 0.0 || mouse_delta[1] != 0.0 {
        yaw -= mouse_delta[0] * sensitivity * delta_time;
        
        if invert_y {
            pitch += mouse_delta[1] * sensitivity * delta_time;
        } else {
            pitch -= mouse_delta[1] * sensitivity * delta_time;
        }
        
        // Clamp pitch to prevent flipping
        if pitch > 1.5 {
            pitch = 1.5;
        } else if pitch < -1.5 {
            pitch = -1.5;
        }
    }
    
    // Calculate movement direction based on input
    let move_input = #{x: 0.0, y: 0.0, z: 0.0};
    
    // Forward/backward
    if input::is_key_pressed("W") {
        move_input.z -= 1.0;
    }
    if input::is_key_pressed("S") {
        move_input.z += 1.0;
    }
    
    // Left/right
    if input::is_key_pressed("A") {
        move_input.x -= 1.0;
    }
    if input::is_key_pressed("D") {
        move_input.x += 1.0;
    }
    
    // Up/down
    if input::is_key_pressed("Space") {
        move_input.y += 1.0;
    }
    if input::is_key_pressed("LeftControl") || input::is_key_pressed("C") {
        move_input.y -= 1.0;
    }
    
    // Apply sprint multiplier
    let current_speed = speed;
    if input::is_key_pressed("LeftShift") {
        current_speed *= sprint_mult;
    }
    
    // Transform movement to world space based on camera rotation
    let forward_dir = #{
        x: -sin(yaw) * cos(pitch),
        y: sin(pitch),
        z: -cos(yaw) * cos(pitch)
    };
    
    let right_dir = #{
        x: cos(yaw),
        y: 0.0,
        z: -sin(yaw)
    };
    
    let up_dir = #{x: 0.0, y: 1.0, z: 0.0};
    
    // Calculate target velocity
    let target_vel = #{
        x: (right_dir.x * move_input.x + forward_dir.x * move_input.z) * current_speed,
        y: move_input.y * current_speed,
        z: (right_dir.z * move_input.x + forward_dir.z * move_input.z) * current_speed
    };
    
    // Smooth velocity
    velocity.x = velocity.x + (target_vel.x - velocity.x) * (1.0 - smoothing);
    velocity.y = velocity.y + (target_vel.y - velocity.y) * (1.0 - smoothing);
    velocity.z = velocity.z + (target_vel.z - velocity.z) * (1.0 - smoothing);
    
    // Apply movement
    transform.position[0] += velocity.x * delta_time;
    transform.position[1] += velocity.y * delta_time;
    transform.position[2] += velocity.z * delta_time;
    
    // Update rotation
    transform.rotation = math::quat_from_euler(pitch, yaw, 0.0);
    
    // Update the transform
    world::set_component(entity, "Transform", transform);
}

fn on_destroy(entity) {
    print("Fly camera controller destroyed for entity: " + entity);
}