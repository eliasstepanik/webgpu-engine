//! Game viewport panel
//!
//! Displays the rendered game view within the editor.

use crate::panel_state::{PanelId, PanelManager};
use crate::shared_state::EditorSharedState;
use imgui::*;
use std::collections::VecDeque;
use std::time::Instant;

/// Performance metrics tracker
pub struct PerformanceMetrics {
    /// Frame time history (in milliseconds)
    frame_times: VecDeque<f32>,
    /// Last frame timestamp
    last_frame: Instant,
    /// Maximum number of samples to keep
    max_samples: usize,
    /// Show performance overlay
    pub show_overlay: bool,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            last_frame: Instant::now(),
            max_samples: 120,
            show_overlay: true,
        }
    }
}

impl PerformanceMetrics {
    /// Update metrics with new frame
    pub fn update(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame).as_secs_f32() * 1000.0;

        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }

        self.last_frame = now;
    }

    /// Get average frame time in milliseconds
    pub fn average_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    /// Get FPS based on average frame time
    pub fn fps(&self) -> f32 {
        let avg = self.average_frame_time();
        if avg > 0.0 {
            1000.0 / avg
        } else {
            0.0
        }
    }

    /// Get min/max frame times
    pub fn min_max_frame_times(&self) -> (f32, f32) {
        if self.frame_times.is_empty() {
            return (0.0, 0.0);
        }
        let min = self
            .frame_times
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0);
        let max = self
            .frame_times
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(0.0);
        (min, max)
    }
}

/// Render the viewport panel with texture
/// Returns the desired viewport size if it has changed
pub fn render_viewport_panel(
    ui: &imgui::Ui,
    texture_id: imgui::TextureId,
    render_target: &engine::graphics::render_target::RenderTarget,
    _shared_state: &EditorSharedState,
    panel_manager: &mut PanelManager,
    _window_size: (f32, f32),
    performance_metrics: &mut PerformanceMetrics,
) -> Option<(u32, u32)> {
    let panel_id = PanelId("viewport".to_string());

    // Get panel info
    let (panel_title, is_visible) = {
        match panel_manager.get_panel(&panel_id) {
            Some(panel) => (panel.title.clone(), panel.is_visible),
            None => return None,
        }
    };

    if !is_visible {
        return None;
    }

    let window_name = format!("{}##{}", panel_title, panel_id.0);
    let mut resize_needed = None;

    ui.window(&window_name)
        .size([800.0, 600.0], Condition::FirstUseEver)
        .position([100.0, 100.0], Condition::FirstUseEver)
        .resizable(true)
        .build(|| {
        let available_size = ui.content_region_avail();
        tracing::debug!(
            "Rendering viewport panel: texture_id={:?}, available_size={:?}, render_target_size={:?}",
            texture_id, available_size, render_target.size
        );

        // Check if viewport needs resizing
        let new_size = (available_size[0] as u32, available_size[1] as u32);
        if new_size != render_target.size && new_size.0 > 0 && new_size.1 > 0 {
            tracing::debug!(
                "Viewport resize needed: {:?} -> {:?}",
                render_target.size,
                new_size
            );
            resize_needed = Some(new_size);
        }

        // Display the game render target with proper aspect ratio
        imgui::Image::new(texture_id, available_size).build(ui);
        // Render performance overlay if enabled
        if performance_metrics.show_overlay {
            // Update metrics
            performance_metrics.update();
            // Position overlay in top-right corner
            let overlay_pos = [
                ui.window_pos()[0] + ui.window_size()[0] - 200.0 - 10.0,
                ui.window_pos()[1] + 30.0
            ];
            ui.window("##PerformanceOverlay")
                .position(overlay_pos, Condition::Always)
                .position_pivot([1.0, 0.0])
                .bg_alpha(0.8)
                .size([200.0, 0.0], Condition::Always)
                .no_decoration()
                .no_inputs()
                .no_nav()
                .build(|| {
                    let fps = performance_metrics.fps();
                    let avg_frame_time = performance_metrics.average_frame_time();
                    let (min_time, max_time) = performance_metrics.min_max_frame_times();
                    // FPS with color coding
                    let fps_color = if fps >= 60.0 {
                        [0.0, 1.0, 0.0, 1.0] // Green
                    } else if fps >= 30.0 {
                        [1.0, 1.0, 0.0, 1.0] // Yellow
                    } else {
                        [1.0, 0.0, 0.0, 1.0] // Red
                    };
                    ui.text_colored(fps_color, format!("FPS: {fps:.1}"));
                    ui.text(format!("Frame: {avg_frame_time:.2} ms"));
                    ui.text(format!("Min: {min_time:.2} ms"));
                    ui.text(format!("Max: {max_time:.2} ms"));
                    // Simple frame time graph
                    if !performance_metrics.frame_times.is_empty() {
                        let values: Vec<f32> = performance_metrics.frame_times.iter().copied().collect();
                        ui.plot_lines("##FrameTime", &values)
                            .graph_size([180.0, 50.0])
                            .scale_min(0.0)
                            .scale_max(33.33) // 30 FPS threshold
                            .build();
                    }
                });
        }

        // Panel position and size are now managed by ImGui's docking system
    });

    resize_needed
}
