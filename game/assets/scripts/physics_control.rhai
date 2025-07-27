// Physics control script - demonstrates physics API usage

let time_accumulator = 0.0;
let impulse_applied = false;

fn on_start() {
    print("Physics control script started!");
    print("This cube will apply upward impulses periodically");
}

fn on_update(delta_time) {
    // Get the entity ID (passed by the scripting system)
    let entity_id = get_entity_id();
    
    // Accumulate time
    time_accumulator += delta_time;
    
    // Every 3 seconds, apply an upward impulse
    if time_accumulator >= 3.0 {
        if !impulse_applied {
            print("Applying upward impulse!");
            
            // Apply an upward impulse
            apply_impulse(entity_id, vec3(0.0, 10.0, 0.0));
            
            // Also apply a small torque for rotation
            apply_torque(entity_id, vec3(0.5, 1.0, 0.2));
            
            impulse_applied = true;
        }
    } else {
        impulse_applied = false;
        time_accumulator = time_accumulator % 3.0;
    }
    
    // Apply a small continuous force to the side based on sine wave
    let side_force = sin(time_accumulator * 2.0) * 2.0;
    apply_force(entity_id, vec3(side_force, 0.0, 0.0));
}

fn on_destroy() {
    print("Physics control script destroyed");
}