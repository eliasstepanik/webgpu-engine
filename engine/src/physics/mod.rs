//! Physics system for the WebGPU engine
//!
//! This module implements rigidbody physics using the Augmented Vertex Block Descent (AVBD) algorithm,
//! providing stable dynamics, collision detection, and constraint solving for real-time applications.

pub mod avbd_solver;
pub mod collision;
pub mod components;
pub mod constraints;
pub mod debug_visualization;
pub mod scene_validator;
pub mod simple_physics;
pub mod systems;

use glam::Vec3;
use serde::{Deserialize, Serialize};

// Re-export main types
pub use components::{Collider, CollisionShape, PhysicsMaterial, Rigidbody};
pub use systems::update_physics_system;

// Re-export solver for advanced usage
pub use avbd_solver::{AVBDConfig, AVBDSolver};

// Re-export scene validation
pub use scene_validator::{validate_physics_scene, SceneValidationResult};

/// Global physics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Gravity acceleration vector
    pub gravity: Vec3,
    /// Fixed timestep for physics simulation (Hz)
    pub fixed_timestep: f32,
    /// Number of position correction iterations
    pub position_iterations: u32,
    /// Number of velocity constraint iterations
    pub velocity_iterations: u32,
    /// Default linear damping factor
    pub linear_damping: f32,
    /// Default angular damping factor
    pub angular_damping: f32,
    /// Velocity threshold for restitution
    pub restitution_threshold: f32,
    /// Allowed penetration before correction
    pub contact_slop: f32,
    /// Maximum linear velocity
    pub max_linear_velocity: f32,
    /// Maximum angular velocity
    pub max_angular_velocity: f32,
    /// Velocity threshold for objects to be considered at rest
    pub rest_velocity_threshold: f32,
    /// Position correction rate (0-1)
    pub position_correction_rate: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_timestep: 1.0 / 120.0, // 120Hz
            position_iterations: 4,
            velocity_iterations: 8,
            linear_damping: 0.01,
            angular_damping: 0.01,
            restitution_threshold: 1.0, // m/s
            contact_slop: 0.004,        // 4mm
            max_linear_velocity: 100.0,
            max_angular_velocity: 100.0,
            rest_velocity_threshold: 0.1,
            position_correction_rate: 0.8,
        }
    }
}
