//! Audio update system

use crate::audio::{
    components::{AmbientSound, AudioSource},
    engine::AudioEngine,
    listener::{find_active_listener, VelocityTracker},
    physical_occlusion::{apply_occlusion_to_audio, calculate_physical_occlusion, OcclusionConfig},
    source::{apply_spatial_params, calculate_panning, SpatialParams},
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
    // TODO: Implement audio module for scripting
    // #[cfg(feature = "audio")]
    // crate::scripting::modules::audio::process_audio_commands(world);

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

    // Update listener in audio engine
    debug!(
        "Audio listener at position: {:?}, master volume: {}",
        listener_state.position, listener_state.master_volume
    );

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
    // First pass: collect entities that need processing
    let entities_to_process: Vec<hecs::Entity> = world
        .query::<(&AudioSource, &Transform)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();

    debug!(
        "Found {} entities with AudioSource components",
        entities_to_process.len()
    );

    // Second pass: process each entity
    for entity in entities_to_process {
        // Get audio source data first
        let source_data = {
            if let Ok(mut query) = world.query_one::<&AudioSource>(entity) {
                if let Some(source) = query.get() {
                    Some((
                        source.sound_path.clone(),
                        source.volume,
                        source.pitch,
                        source.looping,
                        source.auto_play,
                        source.is_playing,
                        source.sound.is_some(),
                        source.spatial,
                        source.max_distance,
                        source.rolloff_factor,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        };

        let Some((
            sound_path,
            volume,
            pitch,
            looping,
            auto_play,
            is_playing,
            has_sound,
            spatial,
            max_distance,
            rolloff_factor,
        )) = source_data
        else {
            continue;
        };

        // Debug why sounds might not be playing
        if !sound_path.is_empty() {
            debug!(
                entity = ?entity,
                sound_path = %sound_path,
                auto_play = auto_play,
                is_playing = is_playing,
                has_sound = has_sound,
                looping = looping,
                spatial = spatial,
                "Audio source state check"
            );
        }

        // Handle auto-play sounds
        if auto_play && !is_playing && !has_sound && !sound_path.is_empty() {
            debug!(
                entity = ?entity,
                sound_path = %sound_path,
                "Attempting to auto-play sound"
            );

            // Calculate initial pan for spatial sounds
            let initial_pan = if spatial {
                if let Ok(mut query) = world.query_one::<&Transform>(entity) {
                    if let Some(transform) = query.get() {
                        calculate_panning(
                            transform.position,
                            listener_state.position,
                            listener_state.forward,
                            listener_state.right,
                        )
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0 // Center pan for non-spatial sounds
            };

            match audio_engine.play_with_settings(
                &sound_path,
                volume * listener_state.master_volume,
                pitch,
                looping,
                initial_pan,
            ) {
                Ok(handle) => {
                    // Update the audio source with the handle
                    if let Ok(mut query) = world.query_one::<&mut AudioSource>(entity) {
                        if let Some(source) = query.get() {
                            source.sound = Some(handle);
                            source.is_playing = true;
                            debug!("Started playing audio source: {}", sound_path);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to play audio source: {}", e);
                }
            }
        }

        // Update spatial parameters if sound is playing
        if let Ok(mut query) = world.query_one::<(&AudioSource, &Transform)>(entity) {
            if let Some((source, transform)) = query.get() {
                if let Some(ref handle) = source.sound {
                    let is_handle_playing = handle.is_playing();
                    debug!(
                        entity = ?entity,
                        handle_playing = is_handle_playing,
                        "Audio handle status check"
                    );

                    // Always update spatial parameters for active handles
                    if spatial {
                        // Use physical occlusion for more realistic sound
                        let occlusion_config = OcclusionConfig::default();
                        let occlusion_result = calculate_physical_occlusion(
                            listener_state.position,
                            transform.position,
                            world,
                            entity,
                            &occlusion_config,
                        );

                        // Apply spatial parameters
                        let spatial_params = SpatialParams {
                            position: transform.position,
                            velocity: glam::Vec3::ZERO, // TODO: Track source velocity
                            max_distance,
                            rolloff_factor,
                        };

                        apply_spatial_params(
                            handle,
                            &spatial_params,
                            listener_state.position,
                            listener_state.forward,
                            listener_state.right,
                            listener_state.velocity,
                            occlusion_result.occlusion,
                        );

                        // Apply frequency-dependent occlusion if available
                        if occlusion_result.occlusion > 0.01 {
                            apply_occlusion_to_audio(
                                handle,
                                &occlusion_result,
                                volume * listener_state.master_volume,
                            );
                        }
                    } else {
                        // Non-spatial sound - just apply volume
                        handle.set_volume(volume * listener_state.master_volume, None);
                    }

                    // Only clear the handle if looping is false and sound has stopped
                    if !looping && !is_handle_playing {
                        // Sound finished playing, update the source
                        debug!(entity = ?entity, "Non-looping sound finished, clearing handle");
                        if let Ok(mut query) = world.query_one::<&mut AudioSource>(entity) {
                            if let Some(source) = query.get() {
                                source.is_playing = false;
                                source.sound = None;
                            }
                        }
                    }
                }
            }
        }
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
                0.0, // Center pan for ambient sounds
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
