// Re-export tracy macros when feature enabled
#[cfg(feature = "tracy")]
pub use tracy_client::{frame_mark, span, span_location, Client, ProfiledAllocator};

// No-op macros when tracy disabled
#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! profile_zone {
    ($name:expr) => {};
}

#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! profile_zone {
    ($name:expr) => {
        let _tracy_zone = tracy_client::span!($name);
    };
}

// Frame marking helper
#[cfg(feature = "tracy")]
pub fn mark_frame() {
    tracy_client::frame_mark();
}

#[cfg(not(feature = "tracy"))]
pub fn mark_frame() {
    // No-op
}

// Memory tracking setup
#[cfg(feature = "tracy")]
pub fn setup_memory_tracking() -> Option<&'static str> {
    // Only enable if explicitly requested via env var
    if std::env::var("TRACY_MEMORY").is_ok() {
        Some("Memory tracking enabled - expect 5-10% performance impact")
    } else {
        None
    }
}

#[cfg(not(feature = "tracy"))]
pub fn setup_memory_tracking() -> Option<&'static str> {
    None
}
