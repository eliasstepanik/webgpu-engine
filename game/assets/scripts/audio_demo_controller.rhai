// Audio Demo Controller Script
// 
// Controls:
// - Space: Play a one-shot sound at current position
// - 1: Toggle sound source 1
// - 2: Toggle sound source 2
// - Up/Down: Adjust master volume
// - Left/Right: Move controller to test spatial audio

// @properties
// camera_speed: float = 10.0
// volume_change_speed: float = 0.5

fn on_start() {
    print("Audio Demo Controller started!");
    print("Controls:");
    print("  Space: Play one-shot sound");
    print("  1/2: Toggle sound sources");
    print("  Up/Down: Adjust volume");
    print("  Left/Right/W/S: Move position");
}

fn on_update(delta_time) {
    // Get this entity's position (the controller entity)
    let pos = world::get_position(entity);
    
    // Movement using properties
    let speed = properties.camera_speed;
    
    if input::is_key_pressed("A") || input::is_key_pressed("Left") {
        pos.x -= speed * delta_time;
    }
    if input::is_key_pressed("D") || input::is_key_pressed("Right") {
        pos.x += speed * delta_time;
    }
    if input::is_key_pressed("W") {
        pos.z -= speed * delta_time;
    }
    if input::is_key_pressed("S") {
        pos.z += speed * delta_time;
    }
    
    world::set_position(entity, pos);
    
    // Play one-shot sound at current position
    if input::is_key_pressed("Space") {
        print("Playing one-shot sound at position: " + pos.x + ", " + pos.y + ", " + pos.z);
        // Note: play_sound_at function would need to be implemented in the audio module
        // For now, just print the action
    }
    
    // Toggle sound sources
    if input::is_key_pressed("Key1") {
        print("Toggle sound source 1 - not implemented yet");
    }
    
    if input::is_key_pressed("Key2") {
        print("Toggle sound source 2 - not implemented yet");
    }
    
    // Volume control
    if input::is_key_pressed("Up") {
        // In a real implementation, we would adjust the listener's master volume
        print("Increasing volume");
    }
    if input::is_key_pressed("Down") {
        // In a real implementation, we would adjust the listener's master volume
        print("Decreasing volume");
    }
    
    // Test sound occlusion by creating a temporary wall
    if input::is_key_pressed("O") {
        print("Creating occlusion test wall - not implemented yet");
        // Note: world::spawn_entity and audio material functions would need to be implemented
    }
}