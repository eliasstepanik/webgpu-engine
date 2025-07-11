//! Window docking system for editor panels
//!
//! This module provides functionality to dock editor panels to the main window borders,
//! allowing for organized layouts similar to modern IDEs. Panels can snap to edges
//! when dragged near window borders and maintain their docked positions during resize.

mod dock_zone;
mod docked_state;

pub use dock_zone::{check_dock_zones, DockZone};
pub use docked_state::{DockEdge, DockedState};

#[cfg(test)]
mod tests;
