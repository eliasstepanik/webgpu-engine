// Physical Audio Occlusion Demo Controller
// This script provides controls and information for the audio occlusion demo

// State variables
let info_displayed = false;
let help_text = [
    "=== Physical Audio Occlusion Demo ===",
    "",
    "This demo shows how different materials affect sound:",
    "- Red sphere: Behind thick concrete wall (blocks most sound)",
    "- Green sphere: Behind thin wooden wall (partial blocking)",  
    "- Blue sphere: Behind glass panel (mostly transparent to sound)",
    "- Yellow sphere: In open space (no occlusion)",
    "",
    "Controls:",
    "- WASD: Move around",
    "- Mouse: Look around", 
    "- H: Toggle this help",
    "- 1-4: Toggle individual sound sources",
    "- Space: Play test tone through all materials",
    "",
    "Notice how:",
    "- Thick walls block high frequencies more than low ones",
    "- Thin materials let more sound through",
    "- Glass is mostly transparent to sound",
    "- Sound can diffract around edges of obstacles",
    "",
    "Press H to hide this help"
];

// Sound source entities
let sound_sources = [
    10, // Behind thick wall
    11, // Behind thin wall  
    12, // Behind glass
    13  // Open space
];

let source_names = [
    "Thick Wall",
    "Thin Wall",
    "Glass Panel",
    "Open Space"
];

// Initialize
fn on_start() {
    print("Physical Audio Occlusion Demo initialized");
    print("Press H for help and controls");
}

// Update function
fn on_update(delta_time) {
    // Toggle help display
    if input::is_key_just_pressed("KeyH") {
        info_displayed = !info_displayed;
        
        if info_displayed {
            for line in help_text {
                print(line);
            }
        } else {
            print("Help hidden. Press H to show again.");
        }
    }
    
    // Toggle individual sound sources
    for i in 0..4 {
        let key = (i + 1).to_string();
        if input::is_key_just_pressed("Key" + key) {
            toggle_sound_source(i);
        }
    }
    
    // Play test tone
    if input::is_key_just_pressed("Space") {
        play_test_sequence();
    }
    
    // Update listener info periodically
    update_listener_info(delta_time);
}

// Toggle a specific sound source
fn toggle_sound_source(index) {
    let entity_id = sound_sources[index];
    let name = source_names[index];
    
    let source = world::get_component(entity_id, "AudioSource");
    if source != () {
        // Note: We can't modify components directly from scripts yet
        // This would need to be implemented through the command system
        print(`Toggle sound ${index + 1} (${name}) - Entity ${entity_id}`);
    }
}

// Play a test sequence through all materials
fn play_test_sequence() {
    print("Playing test sequence through all materials...");
    print("Listen to how each material affects the sound differently");
    
    // This would trigger a sequence where each sound plays in turn
    // Implementation would require audio command support in scripts
}

// Track elapsed time for periodic updates
let update_timer = 0.0;

// Update and display listener information
fn update_listener_info(delta_time) {
    update_timer += delta_time;
    
    // Update every 2 seconds
    if update_timer > 2.0 {
        update_timer = 0.0;
        
        // Find camera/listener position
        // Note: In Rhai scripts, we don't have a way to query for entities with specific components
        // So we'll use the entity variable which is the current entity running this script
        // You may need to adjust this to match your camera entity ID
        let transform = world::get_component(entity, "Transform");
        if transform != () {
            let pos = transform.position;
            
            // Calculate distances to each sound source
            print("=== Audio Status ===");
            print(`Listener position: (${pos.x.to_int()}, ${pos.y.to_int()}, ${pos.z.to_int()})`);
            
            for i in 0..4 {
                let entity_id = sound_sources[i];
                let name = source_names[i];
                
                let source_transform = world::get_component(entity_id, "Transform");
                if source_transform != () {
                    let source_pos = source_transform.position;
                    // Calculate distance manually
                    let dx = source_pos.x - pos.x;
                    let dy = source_pos.y - pos.y;
                    let dz = source_pos.z - pos.z;
                    let distance = sqrt(dx * dx + dy * dy + dz * dz);
                    
                    // Estimate occlusion based on position
                    let occlusion_estimate = estimate_occlusion(i, pos, source_pos);
                    
                    print(`  ${name}: ${distance.to_int()}m away, ~${occlusion_estimate}% blocked`);
                }
            }
        }
    }
}

// Rough estimate of occlusion for display
fn estimate_occlusion(source_index, listener_pos, source_pos) {
    // Simple heuristic based on which obstacle is between listener and source
    switch source_index {
        0 => {
            // Behind thick wall - check if listener is in front of wall
            if listener_pos.z > 0.0 && source_pos.z < 0.0 {
                return 95; // Thick concrete blocks most sound
            }
        }
        1 => {
            // Behind thin wall
            if listener_pos.x > -10.0 && source_pos.x < -10.0 {
                return 60; // Thin wood partially blocks
            }
        }
        2 => {
            // Behind glass
            if listener_pos.x < 10.0 && source_pos.x > 10.0 {
                return 20; // Glass is mostly transparent
            }
        }
        3 => {
            // Open space
            return 0; // No occlusion
        }
    }
    
    return 0;
}

// Handle any cleanup
fn on_destroy() {
    print("Physical Audio Demo - cleanup");
}