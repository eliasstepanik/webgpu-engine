pub mod tracy;

#[cfg(feature = "tracy")]
pub mod gpu;

// Re-export the profile_zone macro
pub use crate::profile_zone;
