//! Scripting system using Rhai
//!
//! This module provides a flexible scripting system that allows entities to execute
//! scripts with access to components, input, and world queries. Scripts support
//! lifecycle functions (on_start, on_update, on_destroy) and integrate seamlessly
//! with the existing ECS architecture.

pub mod commands;
pub mod component_access;
pub mod components;
pub mod engine;
pub mod lifecycle_tracker;
pub mod mesh_registry;
pub mod mesh_upload_system;
pub mod modules;
pub mod property_parser;
pub mod property_types;
pub mod script;
pub mod script_init_system;
pub mod system;

pub use components::ScriptRef;
pub use engine::ScriptEngine;
pub use mesh_registry::ScriptMeshRegistry;
pub use mesh_upload_system::process_script_mesh_uploads;
pub use modules::input::ScriptInputState;
pub use property_types::ScriptProperties;
pub use script::Script;
pub use script_init_system::script_initialization_system;
pub use system::script_execution_system;

// Re-export commonly used types
pub use rhai::{Dynamic, EvalAltResult};

// Command system types
pub use commands::{CommandQueue, ComponentCache, ScriptCommand, SharedComponentCache};

#[cfg(test)]
mod tests;
