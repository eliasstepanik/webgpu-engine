//! Profiling module for Rhai scripts
//!
//! Provides Tracy profiling capabilities to Rhai scripts

use rhai::{Engine, Module};
use tracing::debug;

/// Register profiling API with Rhai engine
pub fn register_profiling_api(_engine: &mut Engine) {
    debug!("Registering profiling API for scripts");

    // Note: We'll register the module in create_profiling_module
}

/// Create a profiling module for scripts
pub fn create_profiling_module() -> Module {
    let mut module = Module::new();

    // Since Tracy zones require RAII and can't be easily managed from scripts,
    // we provide a simpler API that creates instant profiling markers

    // Helper for marking profiling points
    module.set_native_fn("mark", move |name: &str| {
        debug!(script_profile_mark = name, "Script profiling mark");

        // With Tracy, we create a very short-lived zone that appears as a marker
        #[cfg(feature = "tracy")]
        {
            if let Some(client) = tracy_client::Client::running() {
                // Create and immediately drop a zone - this shows up as an instant marker
                let _zone = client.span_alloc(Some(name), "", file!(), line!(), 0);
            }
        }
        Ok(())
    });

    // Helper for logging with a profiling context
    module.set_native_fn("log", move |context: &str, message: &str| {
        debug!(
            script_profile_context = context,
            script_profile_message = message,
            "Script profiling log"
        );

        // Create a brief Tracy zone with the context
        #[cfg(feature = "tracy")]
        {
            if let Some(client) = tracy_client::Client::running() {
                let zone_name = format!("Script::{context}");
                let _zone = client.span_alloc(Some(&zone_name), "", file!(), line!(), 0);
            }
        }
        Ok(())
    });

    module
}
