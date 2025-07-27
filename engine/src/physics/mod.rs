//! Physics system using Rapier3D with f64 precision
//!
//! This module provides physics simulation for the engine using the Rapier physics engine.
//! It supports both standard Transform (f32) and WorldTransform (f64) components for
//! large world scenarios, and integrates with the scripting system.

pub mod commands;
pub mod components;
pub mod debug;
pub mod system;
pub mod world;

#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use commands::{PhysicsCommand, PhysicsCommandQueue};
pub use components::{
    Collider, ColliderShape, PhysicsMass, PhysicsVelocity, RigidBody, RigidBodyType,
};
pub use debug::PhysicsDebugSettings;
pub use system::physics_update_system;
pub use world::PhysicsWorld;

// Re-export commonly used Rapier types
pub use rapier3d_f64::prelude::{
    ColliderBuilder, ColliderHandle, RigidBodyBuilder, RigidBodyHandle,
};
