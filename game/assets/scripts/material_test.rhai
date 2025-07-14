// Simple material color test script
// Changes material color over time to verify updates work
//
//! @property time_accumulator: float = 0.0 @hidden @tooltip("Time accumulator")
//! @property last_print_time: float = 0.0 @hidden @tooltip("Last debug print time")

fn on_start(entity) {
    print("Material test script started for entity " + entity);
}

fn on_update(entity, delta_time) {
    // Update time accumulator
    let time = properties["time_accumulator"];
    time += delta_time;
    properties["time_accumulator"] = time;
    
    // Get the material
    let material = world::get_component(entity, "Material");
    
    if material != () {
        // Create a simple color cycle
        let r = (math::sin(time) + 1.0) * 0.5;
        let g = (math::sin(time + 2.0) + 1.0) * 0.5;
        let b = (math::sin(time + 4.0) + 1.0) * 0.5;
        
        // Method 1: Using set_color method
        material.set_color(r, g, b, 1.0);
        
        // Debug output every second
        let last_print = properties["last_print_time"];
        if time - last_print > 1.0 {
            print("Setting color to: R=" + r + " G=" + g + " B=" + b);
            
            // Also print current color to verify getter works
            let current_color = material.color;
            print("Current color: [" + current_color[0] + ", " + current_color[1] + ", " + current_color[2] + ", " + current_color[3] + "]");
            
            properties["last_print_time"] = time;
        }
        
        // Update the material in the world
        world::set_component(entity, "Material", material);
    } else {
        print("Entity " + entity + " has no Material component!");
    }
}

fn on_destroy(entity) {
    print("Material test script stopped for entity " + entity);
}