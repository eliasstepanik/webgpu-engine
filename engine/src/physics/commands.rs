//! Physics command system for thread-safe operations
//!
//! This module provides a command queue pattern for physics operations,
//! allowing scripts and other systems to safely interact with the physics world.

use glam::Vec3;
use rhai::Dynamic;
// Arc and RwLock removed - using thread-local storage instead

/// Physics command to be executed in the physics system
pub enum PhysicsCommand {
    /// Apply a force to a rigid body
    ApplyForce {
        /// Entity ID to apply force to
        entity: u64,
        /// Force vector in world space
        force: Vec3,
    },

    /// Apply an impulse to a rigid body
    ApplyImpulse {
        /// Entity ID to apply impulse to
        entity: u64,
        /// Impulse vector in world space
        impulse: Vec3,
    },

    /// Apply a torque to a rigid body
    ApplyTorque {
        /// Entity ID to apply torque to
        entity: u64,
        /// Torque vector in world space
        torque: Vec3,
    },

    /// Set the velocity of a rigid body
    SetVelocity {
        /// Entity ID to set velocity for
        entity: u64,
        /// Linear velocity in world space
        linear: Vec3,
        /// Angular velocity in world space
        angular: Vec3,
    },

    /// Perform a raycast query
    Raycast {
        /// Ray origin in world space
        origin: Vec3,
        /// Ray direction (should be normalized)
        direction: Vec3,
        /// Maximum distance for the ray
        max_distance: f32,
        /// Callback to execute with the result
        callback: Box<dyn FnOnce(Option<RaycastHit>) -> Dynamic + Send>,
    },
}

/// Result of a raycast query
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// Entity that was hit
    pub entity: u64,
    /// Distance to the hit point
    pub distance: f32,
    /// Hit point in world space
    pub point: Vec3,
    /// Surface normal at the hit point
    pub normal: Vec3,
}

// Note: The actual physics command queue is thread-local in system.rs
// These types are removed because PhysicsCommand contains a closure that isn't Send
