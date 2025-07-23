// Color pulsing script with customizable colors
// Demonstrates color properties and material manipulation
//
//! @property base_color: color = [1.0, 1.0, 1.0, 1.0] @tooltip("Base color of the object")
//! @property pulse_color: color = [1.0, 0.0, 0.0, 1.0] @tooltip("Color to pulse to")
//! @property pulse_speed: float = 2.0 @range(0.1, 10.0) @step(0.1) @tooltip("Pulsing speed")
//! @property pulse_intensity: float = 0.5 @range(0.0, 1.0) @step(0.05) @tooltip("How much to blend the pulse color (0=no pulse, 1=full pulse)")
//! @property time_accumulator: float = 0.0 @hidden @tooltip("Internal time accumulator")

fn on_start(entity) {
    print("Color pulse script started for entity " + entity);
}

fn on_update(entity, delta_time) {
    // Get properties
    let base_color = properties["base_color"];
    let pulse_color = properties["pulse_color"];
    let pulse_speed = properties["pulse_speed"];
    let pulse_intensity = properties["pulse_intensity"];
    let time_accumulator = properties["time_accumulator"];
    
    // Update time accumulator
    time_accumulator += delta_time * pulse_speed;
    properties["time_accumulator"] = time_accumulator;
    
    // Get material component
    let material = world::get_component(entity, "Material");
    if material != () {
        // Clamp pulse_intensity to valid range
        let clamped_intensity = if pulse_intensity > 1.0 { 1.0 } else if pulse_intensity < 0.0 { 0.0 } else { pulse_intensity };
        
        // Use a sine wave for smooth pulsing
        let pulse_factor = (math::sin(time_accumulator) + 1.0) * 0.5 * clamped_intensity;
        
        // Debug print every 60 frames or so
        if (time_accumulator * 10.0) % 1.0 < 0.1 {
            print("Entity " + entity + " pulse_factor: " + pulse_factor + " intensity: " + clamped_intensity + " time: " + time_accumulator);
        }
        
        // Lerp between base color and pulse color
        material.color[0] = base_color["r"] + (pulse_color["r"] - base_color["r"]) * pulse_factor;
        material.color[1] = base_color["g"] + (pulse_color["g"] - base_color["g"]) * pulse_factor;
        material.color[2] = base_color["b"] + (pulse_color["b"] - base_color["b"]) * pulse_factor;
        material.color[3] = base_color["a"]; // Keep alpha from base color
        
        // Update the material
        world::set_component(entity, "Material", material);
    } else {
        print("Entity " + entity + " has no Material component!");
    }
}

fn on_destroy(entity) {
    print("Color pulse script stopped for entity " + entity);
}