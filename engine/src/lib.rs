//! WebGPU Engine for 3D rendering
//!
//! This crate provides core functionality for WebGPU-based 3D rendering,
//! including primitive generation, camera controls, and shader management.

pub mod core;
pub mod graphics;
pub mod input;
pub mod shaders;

// Re-export commonly used types
pub mod prelude {
    // Entity system types
    pub use crate::core::entity::{
        update_hierarchy_system, Entity, GlobalTransform, Parent, Transform, World,
    };

    // Math types
    pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

    // Graphics types
    pub use wgpu;
    pub use winit;
}

/// Initialize logging for the engine
pub fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,wgpu_core=warn,wgpu_hal=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
