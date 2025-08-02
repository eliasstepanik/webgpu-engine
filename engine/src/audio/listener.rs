//! Audio listener management

use crate::core::entity::{Entity, Transform, World};
use glam::Vec3;

/// Audio listener state
#[derive(Debug, Clone)]
pub struct ListenerState {
    /// Listener entity
    pub entity: Entity,
    /// World position
    pub position: Vec3,
    /// Forward direction
    pub forward: Vec3,
    /// Right direction
    pub right: Vec3,
    /// Up direction
    pub up: Vec3,
    /// Velocity (for Doppler effect)
    pub velocity: Vec3,
    /// Master volume
    pub master_volume: f32,
}

impl ListenerState {
    /// Create listener state from transform
    pub fn from_transform(
        entity: Entity,
        transform: &Transform,
        velocity: Vec3,
        master_volume: f32,
    ) -> Self {
        // Calculate orientation vectors from rotation
        let forward = transform.rotation * Vec3::NEG_Z; // Default forward is -Z
        let right = transform.rotation * Vec3::X;
        let up = transform.rotation * Vec3::Y;

        Self {
            entity,
            position: transform.position,
            forward: forward.normalize(),
            right: right.normalize(),
            up: up.normalize(),
            velocity,
            master_volume,
        }
    }

    /// Update listener state from transform
    pub fn update_from_transform(&mut self, transform: &Transform) {
        self.position = transform.position;
        self.forward = (transform.rotation * Vec3::NEG_Z).normalize();
        self.right = (transform.rotation * Vec3::X).normalize();
        self.up = (transform.rotation * Vec3::Y).normalize();
    }
}

/// Find the active audio listener in the world
pub fn find_active_listener(world: &World) -> Option<ListenerState> {
    use crate::audio::components::AudioListener;

    // Find first active listener
    for (entity, (listener, transform)) in world.query::<(&AudioListener, &Transform)>().iter() {
        if listener.active {
            // TODO: Calculate velocity from previous position
            let velocity = Vec3::ZERO;

            return Some(ListenerState::from_transform(
                entity,
                transform,
                velocity,
                listener.master_volume,
            ));
        }
    }

    None
}

/// Calculate listener velocity from position history
pub struct VelocityTracker {
    previous_position: Option<Vec3>,
    velocity: Vec3,
}

impl VelocityTracker {
    pub fn new() -> Self {
        Self {
            previous_position: None,
            velocity: Vec3::ZERO,
        }
    }

    /// Update velocity based on new position
    pub fn update(&mut self, position: Vec3, delta_time: f32) {
        if let Some(prev_pos) = self.previous_position {
            if delta_time > 0.0 {
                self.velocity = (position - prev_pos) / delta_time;
            }
        }
        self.previous_position = Some(position);
    }

    /// Get current velocity
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }
}

/// Convert world space position to listener space
pub fn world_to_listener_space(world_pos: Vec3, listener: &ListenerState) -> Vec3 {
    let relative = world_pos - listener.position;

    // Create basis matrix from listener orientation
    let x = relative.dot(listener.right);
    let y = relative.dot(listener.up);
    let z = relative.dot(listener.forward);

    Vec3::new(x, y, z)
}

/// Check if a position is within the listener's hearing range
pub fn is_in_hearing_range(source_pos: Vec3, listener_pos: Vec3, max_distance: f32) -> bool {
    let distance_squared = (source_pos - listener_pos).length_squared();
    distance_squared <= max_distance * max_distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_state_from_transform() {
        let mut transform = Transform::default();
        transform.position = Vec3::new(10.0, 5.0, 0.0);

        let state = ListenerState::from_transform(
            Entity::from_bits(1).unwrap(),
            &transform,
            Vec3::ZERO,
            1.0,
        );

        assert_eq!(state.position, Vec3::new(10.0, 5.0, 0.0));
        assert!((state.forward - Vec3::NEG_Z).length() < 0.001);
        assert!((state.right - Vec3::X).length() < 0.001);
        assert!((state.up - Vec3::Y).length() < 0.001);
    }

    #[test]
    fn test_velocity_tracker() {
        let mut tracker = VelocityTracker::new();

        // First update - no velocity yet
        tracker.update(Vec3::ZERO, 0.1);
        assert_eq!(tracker.velocity(), Vec3::ZERO);

        // Second update - calculate velocity
        tracker.update(Vec3::new(1.0, 0.0, 0.0), 0.1);
        let expected_velocity = Vec3::new(10.0, 0.0, 0.0); // 1.0 unit in 0.1 seconds
        assert!((tracker.velocity() - expected_velocity).length() < 0.001);
    }
}
