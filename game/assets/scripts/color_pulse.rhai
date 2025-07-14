// Color pulsing script with customizable colors
// Demonstrates color properties and material manipulation
//
//! @property base_color: color = [1.0, 1.0, 1.0, 1.0] @tooltip("Base color of the object")
//! @property pulse_color: color = [1.0, 0.0, 0.0, 1.0] @tooltip("Color to pulse to")
//! @property pulse_speed: float = 2.0 @range(0.1, 10.0) @step(0.1) @tooltip("Pulsing speed")
//! @property pulse_intensity: float = 0.5 @range(0.0, 1.0) @step(0.05) @tooltip("How much to blend the pulse color")

fn on_start(entity) {
    print("Color pulse script started for entity " + entity);
}

fn on_update(entity, delta_time) {
    
    // Get properties
    let base_color = properties["base_color"];
    let pulse_color = properties["pulse_color"];
    let pulse_speed = properties["pulse_speed"];
    let pulse_intensity = properties["pulse_intensity"];
    
    // Get material component
    let material = world::get_component(entity, "Material");
    if material != () {
        // Without persistent state, we'll use a simple alternating pattern
        // based on the entity ID for variation
        let phase = (entity % 100) as f64 * 0.01;
        let pulse_factor = pulse_intensity * phase;
        
        // Lerp between base color and pulse color
        material.color[0] = base_color["r"] + (pulse_color["r"] - base_color["r"]) * pulse_factor;
        material.color[1] = base_color["g"] + (pulse_color["g"] - base_color["g"]) * pulse_factor;
        material.color[2] = base_color["b"] + (pulse_color["b"] - base_color["b"]) * pulse_factor;
        material.color[3] = base_color["a"]; // Keep alpha from base color
        
        // Update the material
        world::set_component(entity, "Material", material);
    }
}

fn on_destroy(entity) {
    print("Color pulse script stopped for entity " + entity);
}