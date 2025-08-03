// Audio Propagation Demo Controller
// Controls:
// - 1: Toggle small room sound
// - 2: Toggle large room sound  
// - 3: Toggle hallway sound
// - 4: Toggle all sounds
// - Q/A: Increase/Decrease volume
// - W/S: Increase/Decrease pitch
// - Arrow Keys: Move sounds

let sound_entities = [];
let all_sounds_playing = true;
let current_volume = 0.8;
let current_pitch = 1.0;

fn on_start() {
    // Find all entities with audio sources
    let entities = query_entities_with_components(["AudioSource"]);
    
    for entity in entities {
        sound_entities.push(entity);
        print(`Found audio source entity: ${entity}`);
    }
    
    print("Audio Propagation Demo Controls:");
    print("1/2/3 - Toggle small room/large room/hallway sounds");
    print("4 - Toggle all sounds");
    print("Q/A - Increase/Decrease volume");
    print("W/S - Increase/Decrease pitch");
    print("Arrow Keys - Move sound sources");
}

fn on_update(delta_time) {
    let input = get_input();
    
    // Toggle individual sounds
    if input.is_key_pressed("Digit1") && sound_entities.len() > 0 {
        toggle_sound(sound_entities[0]);
        print("Toggled small room sound");
    }
    
    if input.is_key_pressed("Digit2") && sound_entities.len() > 1 {
        toggle_sound(sound_entities[1]);
        print("Toggled large room sound");
    }
    
    if input.is_key_pressed("Digit3") && sound_entities.len() > 2 {
        toggle_sound(sound_entities[2]);
        print("Toggled hallway sound");
    }
    
    // Toggle all sounds
    if input.is_key_pressed("Digit4") {
        all_sounds_playing = !all_sounds_playing;
        for entity in sound_entities {
            set_audio_playing(entity, all_sounds_playing);
        }
        print(`All sounds: ${if all_sounds_playing { "ON" } else { "OFF" }}`);
    }
    
    // Volume control
    if input.is_key_pressed("Q") {
        current_volume = min(1.0, current_volume + 0.1);
        update_all_volumes();
        print(`Volume: ${current_volume}`);
    }
    
    if input.is_key_pressed("A") {
        current_volume = max(0.0, current_volume - 0.1);
        update_all_volumes();
        print(`Volume: ${current_volume}`);
    }
    
    // Pitch control
    if input.is_key_pressed("W") {
        current_pitch = min(2.0, current_pitch + 0.1);
        update_all_pitch();
        print(`Pitch: ${current_pitch}`);
    }
    
    if input.is_key_pressed("S") {
        current_pitch = max(0.5, current_pitch - 0.1);
        update_all_pitch();
        print(`Pitch: ${current_pitch}`);
    }
    
    // Move sound sources with arrow keys
    let move_speed = 5.0 * delta_time;
    if input.is_key_held("ArrowUp") {
        move_sounds(0.0, 0.0, -move_speed);
    }
    if input.is_key_held("ArrowDown") {
        move_sounds(0.0, 0.0, move_speed);
    }
    if input.is_key_held("ArrowLeft") {
        move_sounds(-move_speed, 0.0, 0.0);
    }
    if input.is_key_held("ArrowRight") {
        move_sounds(move_speed, 0.0, 0.0);
    }
}

fn toggle_sound(entity) {
    let audio_source = get_component(entity, "AudioSource");
    if audio_source != () {
        let is_playing = audio_source.is_playing;
        set_audio_playing(entity, !is_playing);
    }
}

fn set_audio_playing(entity, should_play) {
    let audio_source = get_component(entity, "AudioSource");
    if audio_source != () {
        audio_source.auto_play = should_play;
        set_component(entity, "AudioSource", audio_source);
    }
}

fn update_all_volumes() {
    for entity in sound_entities {
        let audio_source = get_component(entity, "AudioSource");
        if audio_source != () {
            audio_source.volume = current_volume;
            set_component(entity, "AudioSource", audio_source);
        }
    }
}

fn update_all_pitch() {
    for entity in sound_entities {
        let audio_source = get_component(entity, "AudioSource");
        if audio_source != () {
            audio_source.pitch = current_pitch;
            set_component(entity, "AudioSource", audio_source);
        }
    }
}

fn move_sounds(dx, dy, dz) {
    for entity in sound_entities {
        let transform = get_component(entity, "Transform");
        if transform != () {
            transform.position[0] += dx;
            transform.position[1] += dy;
            transform.position[2] += dz;
            set_component(entity, "Transform", transform);
        }
    }
}

fn min(a, b) {
    if a < b { a } else { b }
}

fn max(a, b) {
    if a > b { a } else { b }
}