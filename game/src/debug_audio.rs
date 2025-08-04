//! Example integration of audio debug visualization
//!
//! This module shows how to integrate the audio debug visualization into your game.

use engine::audio::{draw_audio_debug, AudioDebugSettings};
use engine::core::entity::World;
use engine::dev::debug_overlay::DebugLineData;
use engine::graphics::Renderer;
use glam::DVec3;

/// Example state for managing debug visualization
pub struct DebugState {
    /// Audio debug settings
    pub audio_settings: AudioDebugSettings,
    /// Collected debug lines
    debug_lines: Vec<DebugLineData>,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            audio_settings: AudioDebugSettings {
                show_sources: true,
                show_ranges: true,
                show_directions: true,
                show_listener: true,
                ..Default::default()
            },
            debug_lines: Vec::new(),
        }
    }
}

impl DebugState {
    /// Update debug visualization
    pub fn update(&mut self, world: &World, camera_position: DVec3) {
        // Clear previous frame's debug lines
        self.debug_lines.clear();

        // Draw audio debug
        draw_audio_debug(
            world,
            &mut self.debug_lines,
            &self.audio_settings,
            camera_position,
        );

        // You can also draw physics debug here if needed:
        // if let Some(physics_world) = &physics_world {
        //     engine::physics::debug::draw_physics_debug(
        //         world,
        //         physics_world,
        //         &mut self.debug_lines,
        //         &physics_settings,
        //         camera_position,
        //     );
        // }
    }

    /// Send debug lines to renderer
    pub fn render(&self, renderer: &mut Renderer) {
        // Convert DebugLineData to flat array format expected by renderer
        // Format: [x, y, z, r, g, b, a] for each vertex
        let mut line_data = Vec::new();

        for line in &self.debug_lines {
            // Start vertex
            line_data.push(line.start.x);
            line_data.push(line.start.y);
            line_data.push(line.start.z);
            line_data.push(line.color.x);
            line_data.push(line.color.y);
            line_data.push(line.color.z);
            line_data.push(line.color.w);

            // End vertex
            line_data.push(line.end.x);
            line_data.push(line.end.y);
            line_data.push(line.end.z);
            line_data.push(line.color.x);
            line_data.push(line.color.y);
            line_data.push(line.color.z);
            line_data.push(line.color.w);
        }

        // Update renderer with debug lines
        renderer.update_debug_lines(&line_data);
    }
}

// Example of how to integrate into your game loop:
// ```rust
// // In your game state
// struct GameState {
//     debug_state: DebugState,
//     // ... other fields
// }
//
// // In your update method
// fn update(&mut self, world: &World, camera_position: DVec3) {
//     // Update debug visualization
//     self.debug_state.update(world, camera_position);
// }
//
// // In your render method
// fn render(&mut self, renderer: &mut Renderer) {
//     // Send debug lines to renderer
//     self.debug_state.render(renderer);
//
//     // Render the scene
//     renderer.render(&world, &surface);
// }
//
// // Toggle debug visualization with a key press
// fn handle_input(&mut self, key: KeyCode) {
//     match key {
//         KeyCode::F3 => {
//             self.debug_state.audio_settings.show_sources =
//                 !self.debug_state.audio_settings.show_sources;
//         }
//         KeyCode::F4 => {
//             self.debug_state.audio_settings.show_ranges =
//                 !self.debug_state.audio_settings.show_ranges;
//         }
//         // ... etc
//     }
// }
// ```
