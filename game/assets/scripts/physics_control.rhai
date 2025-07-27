// Physics control script - demonstrates physics API usage

fn on_start() {
    print("Physics control script started!");
    print("This cube will apply upward impulses periodically");
}

fn on_update(delta_time) {
    // Entity ID is passed by the scripting system
    let entity_id = entity;
    
    // Get properties for persistent state
    let time_accumulator = if "time_accumulator" in properties {
        properties.time_accumulator
    } else {
        0.0
    };
    
    let impulse_applied = if "impulse_applied" in properties {
        properties.impulse_applied
    } else {
        false
    };
    
    // Update time
    let new_time_accumulator = time_accumulator + delta_time;
    properties.time_accumulator = new_time_accumulator;
    
    // Every 3 seconds, apply an upward impulse
    if new_time_accumulator >= 3.0 {
        if !impulse_applied {
            print("Applying upward impulse!");
            
            // Apply an upward impulse
            physics::apply_impulse(entity_id, vec3(0.0, 10.0, 0.0));
            
            // Also apply a small torque for rotation
            physics::apply_torque(entity_id, vec3(0.5, 1.0, 0.2));
            
            properties.impulse_applied = true;
        }
    } else {
        properties.impulse_applied = false;
        properties.time_accumulator = new_time_accumulator % 3.0;
    }
    
    // Apply a small continuous force to the side based on sine wave
    let side_force = sin(properties.time_accumulator * 2.0) * 2.0;
    physics::apply_force(entity_id, vec3(side_force, 0.0, 0.0));
}

fn on_destroy() {
    print("Physics control script destroyed");
}