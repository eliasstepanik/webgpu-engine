// Example script demonstrating Tracy profiling integration
// This script shows how to add profiling markers to your Rhai scripts

fn on_start() {
    // Mark the start of initialization
    profiling::mark("profiling_example::init_start");
    
    // Do some initialization work
    let config = #{
        speed: 10.0,
        enabled: true
    };
    
    // Log with profiling context
    profiling::log("initialization", "Script configuration loaded");
    
    // Mark the end of initialization
    profiling::mark("profiling_example::init_complete");
}

fn on_update(delta_time) {
    // Profile the entire update function
    profiling::mark("profiling_example::update_start");
    
    // Simulate some work
    let mut sum = 0.0;
    for i in 0..100 {
        sum += i as f64 * delta_time;
    }
    
    // Mark specific operations
    profiling::mark("profiling_example::calculations_done");
    
    // Check input with profiling
    if input::is_key_pressed("W") {
        profiling::log("input_handling", "W key pressed - moving forward");
    }
    
    // Mark the end of update
    profiling::mark("profiling_example::update_complete");
}

fn on_destroy() {
    profiling::log("cleanup", "Script being destroyed");
}