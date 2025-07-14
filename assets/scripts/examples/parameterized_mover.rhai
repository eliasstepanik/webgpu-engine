//! Advanced movement script demonstrating all property types
//!
//! This script showcases the full range of property types available
//! for Unity-style script parameters in the editor.
//!
//! @property movement_speed: float = 2.0 @range(0, 10) @step(0.1) @tooltip("How fast the object moves")
//! @property movement_pattern: int = 0 @range(0, 3) @tooltip("0=Linear, 1=Circular, 2=Figure8, 3=Random")
//! @property auto_move: bool = true @tooltip("Whether the object moves automatically")
//! @property target_name: string = "target" @tooltip("Name of the target entity to follow")
//! @property move_direction: vec3 = [1.0, 0.0, 0.0] @tooltip("Direction for linear movement")
//! @property trail_color: color = [1.0, 0.5, 0.0, 1.0] @tooltip("Color of the movement trail")
//! @property circle_radius: float = 3.0 @range(1, 10) @step(0.5)
//! @property oscillate_height: bool = false
//! @property height_amplitude: float = 1.0 @range(0, 5)

let mut time_elapsed = 0.0;
let mut initial_position = #{x: 0.0, y: 0.0, z: 0.0};

fn on_start(entity) {
    print("Parameterized mover started for entity: " + entity);
    
    // Store initial position
    let transform = world::get_component(entity, "Transform");
    if transform != () {
        initial_position.x = transform.position[0];
        initial_position.y = transform.position[1];
        initial_position.z = transform.position[2];
    }
    
    time_elapsed = 0.0;
}

fn on_update(entity, delta_time) {
    // Access all properties
    let speed = properties["movement_speed"];
    let pattern = properties["movement_pattern"];
    let auto_move = properties["auto_move"];
    let target_name = properties["target_name"];
    let direction = properties["move_direction"];
    let color = properties["trail_color"];
    let radius = properties["circle_radius"];
    let oscillate = properties["oscillate_height"];
    let height_amp = properties["height_amplitude"];
    
    if !auto_move {
        return;
    }
    
    time_elapsed += delta_time;
    
    // Get current transform
    let transform = world::get_component(entity, "Transform");
    if transform == () {
        return;
    }
    
    // Calculate new position based on movement pattern
    let new_pos = #{x: 0.0, y: 0.0, z: 0.0};
    
    if pattern == 0 {
        // Linear movement
        let move_delta = speed * delta_time;
        new_pos.x = transform.position[0] + direction["x"] * move_delta;
        new_pos.y = transform.position[1] + direction["y"] * move_delta;
        new_pos.z = transform.position[2] + direction["z"] * move_delta;
        
    } else if pattern == 1 {
        // Circular movement
        let angle = time_elapsed * speed;
        new_pos.x = initial_position.x + cos(angle) * radius;
        new_pos.y = transform.position[1];
        new_pos.z = initial_position.z + sin(angle) * radius;
        
    } else if pattern == 2 {
        // Figure-8 movement
        let t = time_elapsed * speed;
        new_pos.x = initial_position.x + sin(t) * radius;
        new_pos.y = transform.position[1];
        new_pos.z = initial_position.z + sin(t * 2.0) * radius * 0.5;
        
    } else if pattern == 3 {
        // Random walk
        let rand_x = (rand() - 0.5) * 2.0;
        let rand_z = (rand() - 0.5) * 2.0;
        new_pos.x = transform.position[0] + rand_x * speed * delta_time;
        new_pos.y = transform.position[1];
        new_pos.z = transform.position[2] + rand_z * speed * delta_time;
    }
    
    // Apply height oscillation if enabled
    if oscillate {
        let height_offset = sin(time_elapsed * 2.0) * height_amp;
        new_pos.y = initial_position.y + height_offset;
    } else {
        new_pos.y = transform.position[1];
    }
    
    // Update position
    transform.position[0] = new_pos.x;
    transform.position[1] = new_pos.y;
    transform.position[2] = new_pos.z;
    
    // Update material color to match trail color
    let material = world::get_component(entity, "Material");
    if material != () {
        material.base_color[0] = color["r"];
        material.base_color[1] = color["g"];
        material.base_color[2] = color["b"];
        material.base_color[3] = color["a"];
        world::set_component(entity, "Material", material);
    }
    
    // Update the transform
    world::set_component(entity, "Transform", transform);
}

fn on_destroy(entity) {
    print("Parameterized mover destroyed for entity: " + entity);
}

// Helper function for random numbers (0-1)
fn rand() {
    // Simple pseudo-random using time
    let seed = (time_elapsed * 12345.6789) % 1.0;
    return (sin(seed * 43758.5453) + 1.0) * 0.5;
}