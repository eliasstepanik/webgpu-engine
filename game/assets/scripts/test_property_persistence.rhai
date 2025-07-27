// Test script for property persistence
// This script increments a counter and toggles a boolean each frame
//
//! @property counter: int = 0 @tooltip("Frame counter that should persist")
//! @property toggle: bool = false @tooltip("Boolean that toggles each frame")
//! @property message: string = "Initial" @tooltip("Message that changes after 10 frames")

fn on_start() {
    print("Property persistence test script started for entity " + entity);
    print("Initial counter: " + properties["counter"]);
    print("Initial toggle: " + properties["toggle"]);
}

fn on_update(delta_time) {
    // Increment counter
    let current_counter = properties["counter"];
    properties["counter"] = current_counter + 1;
    
    // Toggle boolean
    let current_toggle = properties["toggle"];
    properties["toggle"] = !current_toggle;
    
    // Change message after 10 frames
    if current_counter == 10 {
        properties["message"] = "Changed after 10 frames!";
        print("Message changed at frame 10");
    }
    
    // Print every 30 frames to verify persistence
    if current_counter % 30 == 0 {
        print("Frame " + current_counter + ": toggle = " + current_toggle + ", message = " + properties["message"]);
    }
}

fn on_destroy() {
    print("Property persistence test script stopped");
    print("Final counter: " + properties["counter"]);
}