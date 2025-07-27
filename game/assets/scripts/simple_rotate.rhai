// Simple rotation script for demonstration
// This shows basic script functionality

fn on_start() {
    print("Simple rotation script started for entity " + entity);
}

fn on_update(delta_time) {
    // For now, just print that we're updating
    // In a full implementation, this would access and modify the transform
    print("Updating entity " + entity + " with delta time " + delta_time);
}

fn on_destroy() {
    print("Simple rotation script stopped for entity " + entity);
}