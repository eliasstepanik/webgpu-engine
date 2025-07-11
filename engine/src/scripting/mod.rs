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
pub mod modules;
pub mod script;
pub mod system;

pub use components::ScriptRef;
pub use engine::ScriptEngine;
pub use modules::input::ScriptInputState;
pub use script::Script;
pub use system::script_execution_system;

// Re-export commonly used types
pub use rhai::{Dynamic, EvalAltResult};

// Command system types
pub use commands::{CommandQueue, ComponentCache, ScriptCommand, SharedComponentCache};
