## FEATURE:

Implement window docking to main window borders for proper panel layouts

## EXAMPLES:

examples/scene_demo.rs – Example of scene structure that could benefit from docked panels
examples/developer_layout.json – Shows fixed panel positions that should dock to window edges
examples/artist_layout.json – Viewport-focused layout requiring proper docking alignment
examples/compact_layout.json – Small panel arrangement that needs edge snapping

## DOCUMENTATION:

https://github.com/ocornut/imgui/issues/1591 – ImGui window snapping implementation discussion
https://github.com/ocornut/imgui/issues/2583 – Permanent window docking approaches
https://github.com/ocornut/imgui/issues/7078 – Programmatic dock/undock window control
https://github.com/ocornut/imgui/issues/3537 – Border rendering issues with docked windows

## OTHER CONSIDERATIONS:

- Current implementation uses absolute positioning (panel_state.rs:19-22) without edge attachment
- ImGui has docking feature enabled in Cargo.toml but not utilized for edge snapping
- Panel positions stored in editor_layout.json are fixed coordinates not relative to window edges
- Need to handle window resize events to maintain docked panel positions
- Consider snap zones near window borders (e.g., 10-20 pixel threshold)
- Must preserve ability to float panels when dragged away from edges
- Performance: Each detached window uses ~50-100MB GPU memory (layout_config.json:82-83)
- Security: Validate panel positions stay within safe screen bounds (panel_state.rs:57-66)