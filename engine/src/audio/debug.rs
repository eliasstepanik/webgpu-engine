//! Audio debug visualization
//!
//! This module provides debug rendering for audio sources, listeners, and spatial audio properties,
//! converting audio data to debug lines in camera-relative space.

use crate::audio::panning::calculate_panning;
use crate::audio::{AudioListener, AudioSource};
use crate::core::entity::components::Transform;
use crate::core::entity::{Entity, World};
use crate::dev::debug_overlay::DebugLineData;
use glam::{DVec3, Vec3, Vec4};
use tracing::trace;

/// Settings for audio debug visualization
#[derive(Debug, Clone)]
pub struct AudioDebugSettings {
    /// Show audio source positions
    pub show_sources: bool,
    /// Show attenuation ranges
    pub show_ranges: bool,
    /// Show directional lines to listener
    pub show_directions: bool,
    /// Show listener position/orientation
    pub show_listener: bool,
    /// Color for playing spatial sounds
    pub playing_spatial_color: Vec4,
    /// Color for stopped sounds
    pub stopped_color: Vec4,
    /// Color for playing non-spatial sounds
    pub non_spatial_color: Vec4,
    /// Color for out-of-range sounds
    pub out_of_range_color: Vec4,
}

impl Default for AudioDebugSettings {
    fn default() -> Self {
        Self {
            show_sources: false,
            show_ranges: false,
            show_directions: false,
            show_listener: false,
            playing_spatial_color: Vec4::new(0.0, 1.0, 0.0, 1.0), // Green
            stopped_color: Vec4::new(1.0, 0.0, 0.0, 1.0),         // Red
            non_spatial_color: Vec4::new(0.0, 0.0, 1.0, 1.0),     // Blue
            out_of_range_color: Vec4::new(1.0, 1.0, 0.0, 0.5),    // Yellow (semi-transparent)
        }
    }
}

/// Draw audio debug visualization
pub fn draw_audio_debug(
    world: &World,
    debug_lines: &mut Vec<DebugLineData>,
    settings: &AudioDebugSettings,
    camera_world_position: DVec3,
) {
    if !settings.show_sources
        && !settings.show_ranges
        && !settings.show_directions
        && !settings.show_listener
    {
        return;
    }

    trace!("Drawing audio debug visualization");

    // Get listener info
    let listener_info = get_listener_info(world);

    // Draw audio sources
    for (entity, (source, transform)) in world.query::<(&AudioSource, &Transform)>().iter() {
        // Calculate camera-relative position
        let world_pos = DVec3::from(transform.position);
        let relative_pos = world_pos - camera_world_position;
        let relative_pos_f32 = Vec3::new(
            relative_pos.x as f32,
            relative_pos.y as f32,
            relative_pos.z as f32,
        );

        // Determine playback state and color
        let (is_playing, color) =
            determine_source_state(source, &listener_info, transform.position, settings);

        // Draw source sphere
        if settings.show_sources {
            draw_sphere(debug_lines, relative_pos_f32, 0.5, color, 8); // Small sphere
        }

        // Draw range sphere
        if settings.show_ranges && source.spatial {
            draw_sphere(
                debug_lines,
                relative_pos_f32,
                source.max_distance,
                color * Vec4::new(1.0, 1.0, 1.0, 0.3), // Semi-transparent
                16,
            );
        }

        // Draw directional line
        if settings.show_directions && is_playing && source.spatial {
            if let Some((listener_pos, forward, right)) = &listener_info {
                let listener_relative = *listener_pos - camera_world_position;
                let listener_relative_f32 = Vec3::new(
                    listener_relative.x as f32,
                    listener_relative.y as f32,
                    listener_relative.z as f32,
                );

                // Calculate pan value for color
                let pan = calculate_panning(
                    transform.position,
                    Vec3::from(*listener_pos),
                    forward,
                    right,
                );

                // Color based on pan: red=left, blue=right, white=center
                let line_color = Vec4::new(
                    1.0 - (pan + 1.0) * 0.5, // More red on left
                    1.0,
                    (pan + 1.0) * 0.5, // More blue on right
                    0.8,
                );

                debug_lines.push(DebugLineData {
                    start: listener_relative_f32,
                    end: relative_pos_f32,
                    color: line_color,
                });
            }
        }
    }

    // Draw listener
    if settings.show_listener {
        if let Some((pos, forward, right)) = listener_info {
            draw_listener_gizmo(debug_lines, pos - camera_world_position, forward, right);
        }
    }
}

/// Get active listener information from the world
fn get_listener_info(world: &World) -> Option<(DVec3, Vec3, Vec3)> {
    for (_, (listener, transform)) in world.query::<(&AudioListener, &Transform)>().iter() {
        if listener.active {
            let forward = transform.forward();
            let right = transform.right();
            return Some((DVec3::from(transform.position), forward, right));
        }
    }
    None
}

/// Determine the state and color of an audio source
fn determine_source_state(
    source: &AudioSource,
    listener_info: &Option<(DVec3, Vec3, Vec3)>,
    source_pos: Vec3,
    settings: &AudioDebugSettings,
) -> (bool, Vec4) {
    // Check if handle exists and is playing
    let is_playing = source
        .sound
        .as_ref()
        .map(|handle| handle.is_playing())
        .unwrap_or(false);

    if !is_playing {
        return (false, settings.stopped_color);
    }

    if !source.spatial {
        return (true, settings.non_spatial_color);
    }

    // Check if in range
    if let Some((listener_pos, _, _)) = listener_info {
        let distance = (Vec3::from(*listener_pos) - source_pos).length();
        if distance > source.max_distance {
            return (true, settings.out_of_range_color);
        }
    }

    (true, settings.playing_spatial_color)
}

/// Draw a sphere wireframe
fn draw_sphere(
    debug_lines: &mut Vec<DebugLineData>,
    position: Vec3,
    radius: f32,
    color: Vec4,
    segments: usize,
) {
    // Three circles for the sphere
    for axis in 0..3 {
        for i in 0..segments {
            let angle1 = (i as f32) * 2.0 * std::f32::consts::PI / segments as f32;
            let angle2 = ((i + 1) % segments) as f32 * 2.0 * std::f32::consts::PI / segments as f32;

            let (sin1, cos1) = angle1.sin_cos();
            let (sin2, cos2) = angle2.sin_cos();

            let p1 = match axis {
                0 => Vec3::new(0.0, sin1 * radius, cos1 * radius),
                1 => Vec3::new(sin1 * radius, 0.0, cos1 * radius),
                _ => Vec3::new(sin1 * radius, cos1 * radius, 0.0),
            };

            let p2 = match axis {
                0 => Vec3::new(0.0, sin2 * radius, cos2 * radius),
                1 => Vec3::new(sin2 * radius, 0.0, cos2 * radius),
                _ => Vec3::new(sin2 * radius, cos2 * radius, 0.0),
            };

            debug_lines.push(DebugLineData {
                start: position + p1,
                end: position + p2,
                color,
            });
        }
    }
}

/// Draw a listener gizmo showing position and orientation
fn draw_listener_gizmo(
    debug_lines: &mut Vec<DebugLineData>,
    position: DVec3,
    forward: Vec3,
    right: Vec3,
) {
    let pos = Vec3::new(position.x as f32, position.y as f32, position.z as f32);
    let up = forward.cross(right);

    // Draw coordinate axes
    let axis_length = 2.0;

    // Forward (Z) - Blue
    debug_lines.push(DebugLineData {
        start: pos,
        end: pos + forward * axis_length,
        color: Vec4::new(0.0, 0.0, 1.0, 1.0),
    });

    // Right (X) - Red
    debug_lines.push(DebugLineData {
        start: pos,
        end: pos + right * axis_length,
        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
    });

    // Up (Y) - Green
    debug_lines.push(DebugLineData {
        start: pos,
        end: pos + up * axis_length,
        color: Vec4::new(0.0, 1.0, 0.0, 1.0),
    });

    // Draw cone for forward direction
    let cone_distance = 1.5;
    let cone_radius = 0.5;
    let cone_tip = pos + forward * cone_distance;

    for i in 0..8 {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 8.0;
        let (sin_a, cos_a) = angle.sin_cos();

        let offset = right * sin_a * cone_radius + up * cos_a * cone_radius;
        let base_point = pos + forward * 0.5 + offset;

        // Lines from base to tip
        debug_lines.push(DebugLineData {
            start: base_point,
            end: cone_tip,
            color: Vec4::new(0.5, 0.5, 1.0, 1.0),
        });
    }
}
