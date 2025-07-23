// Test script for asset browser drag-and-drop validation
fn on_init(entity) {
    print("Test script initialized for entity " + entity);
}

fn on_update(entity, world, delta_time) {
    // Simple rotation behavior
    let rotation_speed = 1.0;
    
    if let transform = world.get_component(entity, "Transform") {
        let rotation = transform.rotation;
        let new_rotation = rotation * quat_from_axis_angle(vec3(0.0, 1.0, 0.0), rotation_speed * delta_time);
        transform.rotation = new_rotation;
        world.set_component(entity, "Transform", transform);
    }
}