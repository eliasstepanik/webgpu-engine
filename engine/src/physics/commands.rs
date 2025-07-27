//! Physics command system for thread-safe operations
//!
//! This module provides a command queue pattern for physics operations,
//! allowing scripts and other systems to safely interact with the physics world.

use glam::Vec3;
use std::sync::{Arc, RwLock};

/// Physics command to be executed in the physics system
#[derive(Debug, Clone)]
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
}

/// Thread-safe physics command queue
pub type PhysicsCommandQueue = Arc<RwLock<Vec<PhysicsCommand>>>;

/// Create a new physics command queue
pub fn create_command_queue() -> PhysicsCommandQueue {
    Arc::new(RwLock::new(Vec::new()))
}
