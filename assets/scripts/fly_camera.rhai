// Fly camera controller script
// Allows WASD movement and mouse look
//
// NOTE: Component modifications are queued and applied after all scripts finish executing.
// This ensures thread safety and prevents race conditions between scripts.

let move_speed = 5.0;
let look_speed = 0.005;
let sprint_multiplier = 2.0;

fn on_start(entity) {
    print("Fly camera initialized for entity " + entity);
}

fn on_update(entity, delta_time) {
    // Get the transform component
    let transform = world::get_component(entity, "Transform");
    
    // Movement vector
    let mut move_delta = Vec3::zero();
    
    // Check movement keys
    if input::is_key_pressed("KeyW") {
        move_delta.z -= 1.0;
    }
    if input::is_key_pressed("KeyS") {
        move_delta.z += 1.0;
    }
    if input::is_key_pressed("KeyA") {
        move_delta.x -= 1.0;
    }
    if input::is_key_pressed("KeyD") {
        move_delta.x += 1.0;
    }
    if input::is_key_pressed("Space") {
        move_delta.y += 1.0;
    }
    if input::is_key_pressed("ShiftLeft") {
        move_delta.y -= 1.0;
    }
    
    // Apply sprint if shift is held
    let mut current_speed = move_speed;
    if input::is_key_pressed("ShiftLeft") {
        current_speed *= sprint_multiplier;
    }
    
    // Apply movement in local space
    if move_delta.length() > 0.0 {
        move_delta = move_delta.normalize() * current_speed * delta_time;
        
        // Transform to local space and apply
        let forward = transform.rotate_vector(Vec3::new(0.0, 0.0, -1.0));
        let right = transform.rotate_vector(Vec3::new(1.0, 0.0, 0.0));
        let up = Vec3::new(0.0, 1.0, 0.0); // Keep world up
        
        transform.position += right * move_delta.x;
        transform.position += up * move_delta.y;
        transform.position += forward * move_delta.z;
    }
    
    // Mouse look - only if right mouse button is held
    if input::is_mouse_button_pressed(1) {
        let mouse_delta = input::mouse_delta();
        
        if mouse_delta[0] != 0.0 || mouse_delta[1] != 0.0 {
            // Yaw (horizontal rotation)
            transform.rotate_y(-mouse_delta[0] * look_speed);
            
            // Pitch (vertical rotation) - limited to avoid gimbal lock
            let pitch_delta = -mouse_delta[1] * look_speed;
            
            // Calculate current pitch to limit it
            let forward = transform.rotate_vector(Vec3::new(0.0, 0.0, -1.0));
            let current_pitch = math::asin(forward.y);
            let new_pitch = math::clamp(current_pitch + pitch_delta, -1.4, 1.4); // ~80 degrees limit
            
            if (new_pitch - current_pitch).abs() > 0.001 {
                let pitch_rotation = Quat::from_rotation_x(new_pitch - current_pitch);
                transform.rotation = transform.rotation * pitch_rotation;
            }
        }
    }
    
    // Update the transform
    // Note: This change is queued and will be applied after all scripts have finished their on_update calls
    world::set_component(entity, "Transform", transform);
}

fn on_destroy(entity) {
    print("Fly camera destroyed for entity " + entity);
}