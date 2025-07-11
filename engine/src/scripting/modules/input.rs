//! Input API for Rhai scripts

use rhai::{Dynamic, Engine, Module};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tracing::debug;

/// Input state accessible from scripts
#[derive(Clone, Debug)]
pub struct ScriptInputState {
    pub keys_pressed: HashSet<String>,
    pub mouse_position: (f32, f32),
    pub mouse_delta: (f32, f32),
    pub mouse_buttons: HashSet<u8>,
}

impl ScriptInputState {
    /// Create a new empty input state
    pub fn new() -> Self {
        Self {
            keys_pressed: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            mouse_buttons: HashSet::new(),
        }
    }

    /// Clear the input state (typically at start of frame)
    pub fn clear_frame_data(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    /// Set a key as pressed
    pub fn set_key_pressed(&mut self, key: String, pressed: bool) {
        if pressed {
            self.keys_pressed.insert(key);
        } else {
            self.keys_pressed.remove(&key);
        }
    }

    /// Set mouse position
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    /// Add to mouse delta
    pub fn add_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    /// Set mouse button state
    pub fn set_mouse_button(&mut self, button: u8, pressed: bool) {
        if pressed {
            self.mouse_buttons.insert(button);
        } else {
            self.mouse_buttons.remove(&button);
        }
    }
}

impl Default for ScriptInputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Register input API with Rhai engine
pub fn register_input_api(engine: &mut Engine) {
    debug!("Registering input API");

    // We'll store a shared reference to the input state
    // This will be set by the script execution system
    let input_state: Arc<RwLock<ScriptInputState>> = Arc::new(RwLock::new(ScriptInputState::new()));

    // Create input module
    let mut input_module = Module::new();

    // Clone for each closure
    let state = input_state.clone();
    input_module.set_native_fn("is_key_pressed", move |key: &str| {
        Ok(state.read().unwrap().keys_pressed.contains(key))
    });

    let state = input_state.clone();
    input_module.set_native_fn("mouse_position", move || {
        let state = state.read().unwrap();
        Ok(vec![
            Dynamic::from(state.mouse_position.0 as f64),
            Dynamic::from(state.mouse_position.1 as f64),
        ])
    });

    let state = input_state.clone();
    input_module.set_native_fn("mouse_delta", move || {
        let state = state.read().unwrap();
        Ok(vec![
            Dynamic::from(state.mouse_delta.0 as f64),
            Dynamic::from(state.mouse_delta.1 as f64),
        ])
    });

    let state = input_state.clone();
    input_module.set_native_fn("is_mouse_button_pressed", move |button: i64| {
        Ok(state
            .read()
            .unwrap()
            .mouse_buttons
            .contains(&(button as u8)))
    });

    // Register the module
    engine.register_static_module("input", input_module.into());

    debug!("Input API registered");
}

/// Set the input state for scripts to access
/// This should be called before script execution
pub fn set_script_input_state(_engine: &Engine, _state: &ScriptInputState) {
    // This is a simplified approach - in a real implementation,
    // we'd need a more sophisticated way to share state between
    // the host and scripts. For now, scripts will access a global
    // input state that's updated each frame.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state() {
        let mut state = ScriptInputState::new();

        // Test key press
        state.set_key_pressed("W".to_string(), true);
        assert!(state.keys_pressed.contains("W"));

        state.set_key_pressed("W".to_string(), false);
        assert!(!state.keys_pressed.contains("W"));

        // Test mouse position
        state.set_mouse_position(100.0, 200.0);
        assert_eq!(state.mouse_position, (100.0, 200.0));

        // Test mouse delta
        state.add_mouse_delta(5.0, -3.0);
        assert_eq!(state.mouse_delta, (5.0, -3.0));

        state.clear_frame_data();
        assert_eq!(state.mouse_delta, (0.0, 0.0));

        // Test mouse buttons
        state.set_mouse_button(0, true);
        assert!(state.mouse_buttons.contains(&0));

        state.set_mouse_button(0, false);
        assert!(!state.mouse_buttons.contains(&0));
    }

    #[test]
    fn test_input_api_registration() {
        let mut engine = Engine::new();
        register_input_api(&mut engine);

        // Test that the input module is available
        let result: bool = engine
            .eval(
                r#"
            input::is_key_pressed("W")
        "#,
            )
            .unwrap();
        assert!(!result); // Should be false since no keys are pressed

        // mouse_position returns a Rhai array (Vec of Dynamic)
        let result: rhai::Array = engine.eval("input::mouse_position()").unwrap();
        assert_eq!(result.len(), 2);

        // Extract values from Dynamic array
        let x = result[0].as_float().unwrap();
        let y = result[1].as_float().unwrap();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }
}
