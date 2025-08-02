//! WebGPU Engine for 3D rendering
//!
//! This crate provides core functionality for WebGPU-based 3D rendering,
//! including primitive generation, camera controls, and shader management.

pub mod app;
pub mod component_system;
pub mod config;
pub mod core;
pub mod dev;
pub mod graphics;
pub mod input;
pub mod io;
pub mod physics;
pub mod profiling;
pub mod scripting;
pub mod shaders;
pub mod utils;
pub mod windowing;

#[cfg(feature = "audio")]
pub mod audio;

#[cfg(not(feature = "audio"))]
#[path = "audio_stub.rs"]
pub mod audio;

#[cfg(test)]
mod test_ui_component;

// Re-export commonly used types
pub mod prelude {
    // Entity system types
    pub use crate::core::entity::{
        update_hierarchy_system, Entity, GlobalTransform, Name, Parent, Transform, World,
    };

    // Camera types
    pub use crate::core::camera::{Camera, ProjectionMode};

    // Math types
    pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

    // Graphics types
    pub use crate::graphics::{Material, Mesh, MeshId, RenderContext, Renderer, Vertex};

    // IO types
    pub use crate::io::{Scene, SceneError};

    // Config types
    pub use crate::config::AssetConfig;

    // App types
    pub use crate::app::{EngineApp, EngineBuilder, EngineConfig};

    // Scripting types
    pub use crate::scripting::{ScriptEngine, ScriptProperties, ScriptRef};

    // Physics types
    pub use crate::physics::{
        Collider, ColliderShape, PhysicsMass, PhysicsVelocity, PhysicsWorld, RigidBody,
        RigidBodyType,
    };

    // Input types
    pub use crate::input::InputState;

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
