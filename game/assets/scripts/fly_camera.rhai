// Fly camera controller script
// Allows WASD movement and mouse look
//
// NOTE: Component modifications are queued and applied after all scripts finish executing.
// This ensures thread safety and prevents race conditions between scripts.

fn on_start(entity) {
    print("Fly camera initialized for entity " + entity);
}

fn on_update(entity, delta_time) {
    // Movement parameters
    let move_speed = 5.0;
    let look_speed = 0.005;
    let sprint_multiplier = 2.0;
    
    // Get the transform component
    let transform = world::get_component(entity, "Transform");
    
    // Calculate movement vector based on input
    let move_x = if input::is_key_pressed("KeyA") { -1.0 } else if input::is_key_pressed("KeyD") { 1.0 } else { 0.0 };
    let move_y = if input::is_key_pressed("Space") { 1.0 } else if input::is_key_pressed("ShiftLeft") { -1.0 } else { 0.0 };
    let move_z = if input::is_key_pressed("KeyW") { -1.0 } else if input::is_key_pressed("KeyS") { 0.0 } else { 1.0 };
    let move_delta = Vec3::create(move_x, move_y, move_z);
    
    // Apply sprint if shift is held
    let current_speed = if input::is_key_pressed("ShiftLeft") {
        move_speed * sprint_multiplier
    } else {
        move_speed
    };
    
    // Apply movement in local space
    if move_delta.length() > 0.0 {
        let normalized_delta = move_delta.normalize() * current_speed * delta_time;
        
        // Transform to local space and apply
        let forward = transform.rotate_vector(Vec3::create(0.0, 0.0, -1.0));
        let right = transform.rotate_vector(Vec3::create(1.0, 0.0, 0.0));
        let up = Vec3::create(0.0, 1.0, 0.0); // Keep world up
        
        transform.position = transform.position + right * normalized_delta.x;
        transform.position = transform.position + up * normalized_delta.y;
        transform.position = transform.position + forward * normalized_delta.z;
    }
    
    // Mouse look - only if right mouse button is held
    if input::is_mouse_button_pressed(1) {
        let mouse_delta = input::mouse_delta();
        
        if mouse_delta[0] != 0.0 || mouse_delta[1] != 0.0 {
            // Yaw (horizontal rotation)
            transform.rotate_y(-mouse_delta[0] * look_speed);
            
            // Pitch (vertical rotation) - limited to avoid gimbal lock
            let pitch_delta = -mouse_delta[1] * look_speed;
            
            // Simple pitch rotation with clamping
            // We'll apply the rotation and rely on the game logic to handle limits
            let pitch_rotation = Quat::from_rotation_x(pitch_delta);
            transform.rotation = transform.rotation * pitch_rotation;
        }
    }
    
    // Update the transform
    // Note: This change is queued and will be applied after all scripts have finished their on_update calls
    world::set_component(entity, "Transform", transform);
}

fn on_destroy(entity) {
    print("Fly camera destroyed for entity " + entity);
}