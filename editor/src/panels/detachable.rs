//! Detachable panel wrapper
//!
//! Provides a wrapper for panels that can be detached into separate windows

use crate::panel_state::{PanelId, PanelManager};
use imgui::*;
use tracing::info;

/// Render a detachable panel window
///
/// This wrapper adds a detach button to the window title bar
pub fn detachable_window<F>(
    ui: &Ui,
    panel_id: &PanelId,
    panel_manager: &mut PanelManager,
    content: F,
) where
    F: FnOnce(),
{
    // Get panel info first to avoid borrowing issues
    let (panel_title, panel_position, panel_size, is_visible, is_detached) = {
        match panel_manager.get_panel(panel_id) {
            Some(panel) => (
                panel.title.clone(),
                panel.position,
                panel.size,
                panel.is_visible,
                panel.is_detached,
            ),
            None => return,
        }
    };

    if !is_visible {
        return;
    }

    // Only render if this panel belongs to the current window
    // (This check would be done by the caller in practice)

    let window_name = format!("{}##{}", panel_title, panel_id.0);

    // Build window with appropriate settings
    let mut window = ui.window(&window_name);

    // Configure window based on detached state
    if is_detached {
        // For detached panels, use NO_DOCKING to prevent docking back
        // Don't force position - let ImGui handle viewport creation
        window = window
            .position(
                [panel_position.0, panel_position.1],
                Condition::FirstUseEver,
            )
            .size([panel_size.0, panel_size.1], Condition::FirstUseEver)
            .flags(WindowFlags::NO_DOCKING)
            .resizable(true);

        info!("Rendering detached panel: {:?}", panel_id);
    } else {
        // For attached panels, use normal window settings
        window = window
            .position(
                [panel_position.0, panel_position.1],
                Condition::FirstUseEver,
            )
            .size([panel_size.0, panel_size.1], Condition::FirstUseEver)
            .resizable(true);
    }

    window.build(|| {
        // Add detach button in the window's top-right corner
        let window_pos = ui.window_pos();
        let window_size = ui.window_size();
        let button_size = 20.0;
        let padding = 5.0;

        // Position button in top-right corner
        let _button_pos = [
            window_pos[0] + window_size[0] - button_size - padding,
            window_pos[1] + padding,
        ];

        // Save current cursor position
        let cursor_pos = ui.cursor_pos();

        // Draw detach button
        ui.set_cursor_pos([window_size[0] - button_size - padding, padding]);

        // Enable detach/attach functionality with viewport support
        let button_text = if is_detached { "⬊" } else { "⬈" };
        if ui.button_with_size(button_text, [button_size, button_size]) {
            if is_detached {
                // Request reattachment
                panel_manager.request_attach(panel_id.clone());
                info!("Requesting attachment for panel: {:?}", panel_id);
            } else {
                // Request detachment
                panel_manager.request_detach(panel_id.clone());
                info!("Requesting detachment for panel: {:?}", panel_id);
            }
        }

        if ui.is_item_hovered() {
            ui.tooltip(|| {
                if is_detached {
                    ui.text("Reattach panel to main window");
                } else {
                    ui.text("Detach panel to separate window");
                    ui.text("Using Dear ImGui viewport system");
                }
            });
        }

        // Restore cursor position
        ui.set_cursor_pos(cursor_pos);

        // Render the actual content
        content();

        // Update panel position and size if window was moved/resized
        if let Some(panel) = panel_manager.get_panel_mut(panel_id) {
            let new_pos = ui.window_pos();
            let new_size = ui.window_size();

            if !panel.is_detached {
                panel.position = (new_pos[0], new_pos[1]);
                panel.size = (new_size[0], new_size[1]);
            }
        }
    });
}
