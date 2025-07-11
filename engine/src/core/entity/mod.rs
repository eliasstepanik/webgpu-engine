//! Entity-Component System (ECS) functionality
//!
//! This module provides the core ECS functionality for the engine,
//! including transform components and hierarchy management.

pub mod components;
pub mod hierarchy;
pub mod world;

// Re-export commonly used types
pub use components::{GlobalTransform, Name, Parent, Transform};
pub use hierarchy::update_hierarchy_system;
pub use world::World;

// Re-export hecs types that users will need
pub use hecs::Entity;
