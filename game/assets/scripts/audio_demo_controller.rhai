// Audio Demo Controller Script
// 
// Controls:
// - Space: Play a one-shot sound at camera position
// - 1: Toggle sound source 1
// - 2: Toggle sound source 2
// - Up/Down: Adjust master volume
// - Left/Right: Move camera to test spatial audio

let camera_speed = 10.0;
let volume_change_speed = 0.5;

// State
let sound1_entity = 0;
let sound2_entity = 0;

fn on_start() {
    print("Audio Demo Controller started!");
    print("Controls:");
    print("  Space: Play one-shot sound");
    print("  1/2: Toggle sound sources");
    print("  Up/Down: Adjust volume");
    print("  Left/Right/W/S: Move camera");
    
    // Find sound source entities
    // Note: find_entities_with_name is not implemented yet
    // For now, we'll use hardcoded entity IDs or skip this functionality
    sound1_entity = 0;
    sound2_entity = 0;
}

fn on_update(delta_time) {
    // For now, assume camera is entity 0 (first entity in scene)
    let camera_entity = 0;
    let pos = world::get_position(camera_entity);
    
    // Camera movement
    if input::is_key_held("A") || input::is_key_held("Left") {
        pos.x -= camera_speed * delta_time;
    }
    if input::is_key_held("D") || input::is_key_held("Right") {
        pos.x += camera_speed * delta_time;
    }
    if input::is_key_held("W") {
        pos.z -= camera_speed * delta_time;
    }
    if input::is_key_held("S") {
        pos.z += camera_speed * delta_time;
    }
    
    world::set_position(camera_entity, pos);
    
    // Play one-shot sound at camera position
    if input::is_key_pressed("Space") {
        print("Playing one-shot sound at camera position");
        play_sound_at("sounds/impact.wav", pos.x, pos.y, pos.z, 0.8);
    }
    
    // Toggle sound sources
    if input::is_key_pressed("Key1") && sound1_entity != 0 {
        // In a real implementation, we would toggle the AudioSource component
        print("Toggling sound source 1");
    }
    
    if input::is_key_pressed("Key2") && sound2_entity != 0 {
        // In a real implementation, we would toggle the AudioSource component
        print("Toggling sound source 2");
    }
    
    // Volume control
    if input::is_key_held("Up") {
        // In a real implementation, we would adjust the listener's master volume
        print("Increasing volume");
    }
    if input::is_key_held("Down") {
        // In a real implementation, we would adjust the listener's master volume
        print("Decreasing volume");
    }
    
    // Test sound occlusion by creating a temporary wall
    if input::is_key_pressed("O") {
        print("Creating occlusion test wall");
        let wall = world::spawn_entity();
        world::set_position(wall, vec3(0.0, 5.0, 0.0));
        world::set_scale(wall, vec3(5.0, 10.0, 0.5));
        
        // Set audio material for occlusion
        set_audio_material(wall, 0.05, 0.0); // Low absorption, no transmission
    }
}