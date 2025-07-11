//! Input state tracking

use std::collections::HashSet;
use tracing::trace;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Tracks the current state of input devices
#[derive(Debug, Clone)]
pub struct InputState {
    /// Currently pressed keys
    pub keys_pressed: HashSet<KeyCode>,
    /// Mouse position in window coordinates
    pub mouse_position: (f32, f32),
    /// Mouse movement delta since last frame
    pub mouse_delta: (f32, f32),
    /// Currently pressed mouse buttons
    pub mouse_buttons_pressed: HashSet<MouseButton>,
}

impl InputState {
    /// Create a new empty input state
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            mouse_buttons_pressed: HashSet::new(),
        }
    }

    /// Clear per-frame data (like mouse delta)
    pub fn clear_frame_data(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    /// Handle a keyboard event
    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    self.keys_pressed.insert(key_code);
                    trace!("Key pressed: {:?}", key_code);
                }
                ElementState::Released => {
                    self.keys_pressed.remove(&key_code);
                    trace!("Key released: {:?}", key_code);
                }
            }
        }
    }

    /// Update mouse position
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    /// Add mouse movement delta
    pub fn add_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
        trace!("Mouse delta: ({}, {})", dx, dy);
    }

    /// Handle a mouse button event
    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_buttons_pressed.insert(button);
                trace!("Mouse button pressed: {:?}", button);
            }
            ElementState::Released => {
                self.mouse_buttons_pressed.remove(&button);
                trace!("Mouse button released: {:?}", button);
            }
        }
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.keys_pressed.contains(&key_code)
    }

    /// Check if a mouse button is currently pressed
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    /// Convert to script input state
    pub fn to_script_input_state(&self) -> crate::scripting::modules::input::ScriptInputState {
        let mut script_state = crate::scripting::modules::input::ScriptInputState::new();

        // Convert key codes to string names
        for key in &self.keys_pressed {
            let key_name = format!("{key:?}");
            script_state.keys_pressed.insert(key_name);
        }

        script_state.mouse_position = self.mouse_position;
        script_state.mouse_delta = self.mouse_delta;

        // Convert mouse buttons
        for button in &self.mouse_buttons_pressed {
            let button_id = match button {
                MouseButton::Left => 0,
                MouseButton::Right => 1,
                MouseButton::Middle => 2,
                MouseButton::Back => 3,
                MouseButton::Forward => 4,
                MouseButton::Other(id) => *id as u8,
            };
            script_state.mouse_buttons.insert(button_id);
        }

        script_state
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::MouseButton;
    use winit::keyboard::KeyCode;

    #[test]
    fn test_input_state_keyboard() {
        let mut state = InputState::new();

        // Simulate key press
        // Create a default KeyEvent using the builder pattern
        // This avoids dealing with platform-specific internals

        // Simulate key press by directly modifying state
        state.keys_pressed.insert(KeyCode::KeyW);

        assert!(state.is_key_pressed(KeyCode::KeyW));

        // Simulate key release
        state.keys_pressed.remove(&KeyCode::KeyW);
        assert!(!state.is_key_pressed(KeyCode::KeyW));
    }

    #[test]
    fn test_input_state_mouse() {
        let mut state = InputState::new();

        // Test mouse position
        state.set_mouse_position(100.0, 200.0);
        assert_eq!(state.mouse_position, (100.0, 200.0));

        // Test mouse delta
        state.add_mouse_delta(5.0, -3.0);
        assert_eq!(state.mouse_delta, (5.0, -3.0));

        state.clear_frame_data();
        assert_eq!(state.mouse_delta, (0.0, 0.0));

        // Test mouse buttons
        state.handle_mouse_button(MouseButton::Left, ElementState::Pressed);
        assert!(state.is_mouse_button_pressed(MouseButton::Left));

        state.handle_mouse_button(MouseButton::Left, ElementState::Released);
        assert!(!state.is_mouse_button_pressed(MouseButton::Left));
    }

    #[test]
    fn test_to_script_input_state() {
        let mut state = InputState::new();

        // Set up some test data
        state.keys_pressed.insert(KeyCode::KeyW);
        state.keys_pressed.insert(KeyCode::Space);
        state.mouse_position = (150.0, 250.0);
        state.mouse_delta = (10.0, -5.0);
        state.mouse_buttons_pressed.insert(MouseButton::Left);

        let script_state = state.to_script_input_state();

        assert!(script_state.keys_pressed.contains("KeyW"));
        assert!(script_state.keys_pressed.contains("Space"));
        assert_eq!(script_state.mouse_position, (150.0, 250.0));
        assert_eq!(script_state.mouse_delta, (10.0, -5.0));
        assert!(script_state.mouse_buttons.contains(&0)); // Left button = 0
    }
}
