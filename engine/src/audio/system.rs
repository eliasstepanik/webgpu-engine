//! Audio update system

use crate::audio::{
    components::{AmbientSound, AudioSource},
    engine::AudioEngine,
    listener::{find_active_listener, VelocityTracker},
    propagation::calculate_occlusion,
    source::{apply_spatial_params, SpatialParams},
};
use crate::core::entity::{Transform, World};
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Audio system state
pub struct AudioSystemState {
    /// Velocity trackers for listeners
    listener_velocity_trackers: HashMap<u64, VelocityTracker>,
    /// Currently playing ambient sounds
    playing_ambient: HashMap<u64, crate::audio::AudioHandle>,
}

impl Default for AudioSystemState {
    fn default() -> Self {
        Self {
            listener_velocity_trackers: HashMap::new(),
            playing_ambient: HashMap::new(),
        }
    }
}

/// Update the audio system
pub fn audio_update_system(
    world: &mut World,
    audio_engine: &mut AudioEngine,
    state: &mut AudioSystemState,
    delta_time: f32,
) {
    // Process audio commands from scripts
    #[cfg(feature = "audio")]
    crate::scripting::modules::audio::process_audio_commands(world);

    // Find active listener
    let listener_state = match find_active_listener(world) {
        Some(mut listener) => {
            // Update velocity tracking
            let entity_id = listener.entity.to_bits().get();
            let tracker = state
                .listener_velocity_trackers
                .entry(entity_id)
                .or_insert_with(VelocityTracker::new);

            tracker.update(listener.position, delta_time);
            listener.velocity = tracker.velocity();

            listener
        }
        None => {
            trace!("No active audio listener found");
            return;
        }
    };

    // Process audio sources
    process_audio_sources(world, audio_engine, &listener_state);

    // Process ambient sounds
    process_ambient_sounds(world, audio_engine, state, &listener_state);
}

/// Process all audio sources in the world
fn process_audio_sources(
    world: &mut World,
    audio_engine: &mut AudioEngine,
    listener_state: &crate::audio::listener::ListenerState,
) {
    // Collect source updates to avoid borrow conflicts
    let mut source_updates = Vec::new();

    for (entity, (source, transform)) in world.query::<(&AudioSource, &Transform)>().iter() {
        source_updates.push((entity, source.clone(), *transform));
    }

    // Apply updates
    for (entity, mut source, transform) in source_updates {
        // Load sound if needed
        if source.sound.is_none() && !source.sound_path.is_empty() {
            match audio_engine.load_sound(&source.sound_path) {
                Ok(_) => {
                    debug!("Loaded audio source: {}", source.sound_path);
                }
                Err(e) => {
                    warn!("Failed to load audio source {}: {}", source.sound_path, e);
                    continue;
                }
            }
        }

        // Play sound if auto_play and not playing
        if source.auto_play && !source.is_playing && source.sound.is_none() {
            match audio_engine.play_with_settings(
                &source.sound_path,
                source.volume * listener_state.master_volume,
                source.pitch,
                source.looping,
            ) {
                Ok(handle) => {
                    source.sound = Some(handle);
                    source.is_playing = true;
                }
                Err(e) => {
                    warn!("Failed to play audio source: {}", e);
                }
            }
        }

        // Update spatial parameters if sound is playing
        if let Some(ref handle) = source.sound {
            if handle.is_playing() {
                if source.spatial {
                    // Calculate occlusion
                    let occlusion = calculate_occlusion(
                        listener_state.position,
                        transform.position,
                        world,
                        entity,
                    );

                    // Apply spatial parameters
                    let spatial_params = SpatialParams {
                        position: transform.position,
                        velocity: glam::Vec3::ZERO, // TODO: Track source velocity
                        max_distance: source.max_distance,
                        rolloff_factor: source.rolloff_factor,
                    };

                    apply_spatial_params(
                        handle,
                        &spatial_params,
                        listener_state.position,
                        listener_state.forward,
                        listener_state.right,
                        listener_state.velocity,
                        occlusion,
                    );
                } else {
                    // Non-spatial sound - just apply volume
                    handle.set_volume(source.volume * listener_state.master_volume, None);
                }
            } else {
                source.is_playing = false;
                source.sound = None;
            }
        }

        // Write back updated source
        let _ = world.insert_one(entity, source);
    }
}

/// Process ambient sounds
fn process_ambient_sounds(
    world: &mut World,
    audio_engine: &mut AudioEngine,
    state: &mut AudioSystemState,
    listener_state: &crate::audio::listener::ListenerState,
) {
    // Collect ambient sound updates
    let mut ambient_updates = Vec::new();

    for (entity, ambient) in world.query::<&AmbientSound>().iter() {
        ambient_updates.push((entity, ambient.clone()));
    }

    // Apply updates
    for (entity, mut ambient) in ambient_updates {
        let entity_id = entity.to_bits().get();

        // Load sound if needed
        if ambient.sound.is_none() && !ambient.sound_path.is_empty() {
            match audio_engine.load_sound(&ambient.sound_path) {
                Ok(_) => {
                    debug!("Loaded ambient sound: {}", ambient.sound_path);
                }
                Err(e) => {
                    warn!("Failed to load ambient sound {}: {}", ambient.sound_path, e);
                    continue;
                }
            }
        }

        // Check if we have a playing handle
        let is_playing = state
            .playing_ambient
            .get(&entity_id)
            .map(|h| h.is_playing())
            .unwrap_or(false);

        // Play sound if auto_play and not playing
        if ambient.auto_play && !is_playing {
            match audio_engine.play_with_settings(
                &ambient.sound_path,
                0.0, // Start at zero volume for fade in
                1.0,
                ambient.looping,
            ) {
                Ok(handle) => {
                    // Set volume (fade-in would need to be implemented manually if needed)
                    handle.set_volume(ambient.volume * listener_state.master_volume, None);

                    ambient.sound = Some(handle.clone());
                    state.playing_ambient.insert(entity_id, handle);
                }
                Err(e) => {
                    warn!("Failed to play ambient sound: {}", e);
                }
            }
        }

        // Update volume for playing ambient sounds
        if let Some(handle) = state.playing_ambient.get(&entity_id) {
            if !handle.is_playing() {
                state.playing_ambient.remove(&entity_id);
                ambient.sound = None;
            }
        }

        // Write back updated ambient
        let _ = world.insert_one(entity, ambient);
    }

    // Clean up stopped ambient sounds
    state
        .playing_ambient
        .retain(|_, handle| handle.is_playing());
}
