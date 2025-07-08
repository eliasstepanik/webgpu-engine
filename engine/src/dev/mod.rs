//! Development utilities and debugging tools
//! 
//! This module provides tools for debugging and development workflow improvements.
//! It includes scene statistics, debug overlays, and hot-reload integration.

pub mod debug_overlay;

pub use debug_overlay::{DebugOverlay, SceneDebugInfo};

/// Feature flag to enable development tools only in debug builds
#[cfg(debug_assertions)]
pub const DEV_TOOLS_ENABLED: bool = true;

#[cfg(not(debug_assertions))]
pub const DEV_TOOLS_ENABLED: bool = false;

/// Development utilities for scene debugging and workflow
pub struct DevTools;

impl DevTools {
    /// Check if development tools are enabled
    pub fn is_enabled() -> bool {
        DEV_TOOLS_ENABLED
    }

    /// Log development information if tools are enabled
    pub fn log_dev_info(message: &str) {
        if Self::is_enabled() {
            tracing::debug!(target: "dev_tools", "{}", message);
        }
    }

    /// Log development warning if tools are enabled
    pub fn log_dev_warning(message: &str) {
        if Self::is_enabled() {
            tracing::warn!(target: "dev_tools", "{}", message);
        }
    }

    /// Log development error if tools are enabled
    pub fn log_dev_error(message: &str) {
        if Self::is_enabled() {
            tracing::error!(target: "dev_tools", "{}", message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_tools_enabled() {
        // Should match debug_assertions
        #[cfg(debug_assertions)]
        assert!(DevTools::is_enabled());
        
        #[cfg(not(debug_assertions))]
        assert!(!DevTools::is_enabled());
    }
}