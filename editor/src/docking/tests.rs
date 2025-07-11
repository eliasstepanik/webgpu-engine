//! Tests for the docking module

use super::*;

#[test]
fn test_dock_zone_detection() {
    let zone = DockZone {
        edge: DockEdge::Left,
        threshold: 20.0,
    };
    let docked = zone.check_snap((10.0, 100.0), (200.0, 300.0), (800.0, 600.0));
    assert!(docked.is_some());
    assert_eq!(docked.unwrap().edge, DockEdge::Left);
}

#[test]
fn test_docked_position_calculation() {
    let docked = DockedState {
        edge: DockEdge::Top,
        offset: 0.5,
        size: 300.0,
    };
    let pos = docked.calculate_position((200.0, 100.0), (800.0, 600.0));
    assert_eq!(pos, (300.0, 0.0)); // Centered on top edge
}

#[test]
fn test_layout_serialization() {
    use serde_json;

    let docked = DockedState {
        edge: DockEdge::Right,
        offset: 0.3,
        size: 250.0,
    };

    let json = serde_json::to_string(&docked).unwrap();
    let parsed: DockedState = serde_json::from_str(&json).unwrap();

    assert_eq!(docked.edge, parsed.edge);
    assert_eq!(docked.offset, parsed.offset);
    assert_eq!(docked.size, parsed.size);
}
