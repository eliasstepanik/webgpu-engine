//! Audio scripting module

use crate::audio::{AudioEngine, AudioHandle, AudioMaterial};
use crate::core::entity::Entity;
use glam::Vec3;
use rhai::{Dynamic, EvalAltResult, Module};
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Thread-safe audio engine wrapper for scripting
#[derive(Clone)]
pub struct ScriptAudioEngine {
    engine: Arc<Mutex<AudioEngine>>,
}

impl ScriptAudioEngine {
    pub fn new(engine: Arc<Mutex<AudioEngine>>) -> Self {
        Self { engine }
    }

    /// Play a sound
    pub fn play_sound(&self, path: &str) -> Result<AudioHandle, Box<EvalAltResult>> {
        let mut engine = self.engine.lock().map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("Failed to lock audio engine: {}", e).into(),
                rhai::Position::NONE,
            ))
        })?;

        engine.play(path).map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("Failed to play sound: {}", e).into(),
                rhai::Position::NONE,
            ))
        })
    }

    /// Play a sound with custom settings
    pub fn play_sound_with_settings(
        &self,
        path: &str,
        volume: f32,
        pitch: f32,
        looping: bool,
    ) -> Result<AudioHandle, Box<EvalAltResult>> {
        let mut engine = self.engine.lock().map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("Failed to lock audio engine: {}", e).into(),
                rhai::Position::NONE,
            ))
        })?;

        engine
            .play_with_settings(path, volume, pitch, looping)
            .map_err(|e| {
                Box::new(EvalAltResult::ErrorRuntime(
                    format!("Failed to play sound: {}", e).into(),
                    rhai::Position::NONE,
                ))
            })
    }

    /// Play a one-shot sound
    pub fn play_one_shot(&self, path: &str, volume: f32) -> Result<(), Box<EvalAltResult>> {
        let mut engine = self.engine.lock().map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("Failed to lock audio engine: {}", e).into(),
                rhai::Position::NONE,
            ))
        })?;

        engine.play_one_shot(path, volume, 1.0).map_err(|e| {
            Box::new(EvalAltResult::ErrorRuntime(
                format!("Failed to play one-shot sound: {}", e).into(),
                rhai::Position::NONE,
            ))
        })
    }
}

/// Audio command for deferred execution
pub enum AudioCommand {
    /// Set audio material properties for an entity
    SetAudioMaterial {
        entity: u64,
        absorption: f32,
        transmission: f32,
    },
    /// Play sound at position (creates temporary audio source)
    PlaySoundAt {
        path: String,
        position: Vec3,
        volume: f32,
    },
}

thread_local! {
    /// Thread-local audio command queue
    static AUDIO_COMMAND_QUEUE: std::cell::RefCell<Vec<AudioCommand>> = std::cell::RefCell::new(Vec::new());
}

/// Queue an audio command for execution in the next update
pub fn queue_audio_command(command: AudioCommand) {
    AUDIO_COMMAND_QUEUE.with(|queue| {
        queue.borrow_mut().push(command);
    });
}

/// Process queued audio commands
pub fn process_audio_commands(world: &mut crate::core::entity::World) {
    AUDIO_COMMAND_QUEUE.with(|queue| {
        let commands = queue.borrow_mut().drain(..).collect::<Vec<_>>();

        for command in commands {
            match command {
                AudioCommand::SetAudioMaterial {
                    entity,
                    absorption,
                    transmission,
                } => {
                    if let Some(entity) = Entity::from_bits(entity) {
                        let material = AudioMaterial {
                            absorption: absorption.clamp(0.0, 1.0),
                            scattering: 0.5, // Default scattering
                            transmission: transmission.clamp(0.0, 1.0),
                        };
                        let _ = world.insert_one(entity, material);
                        debug!("Set audio material for entity {:?}", entity);
                    }
                }
                AudioCommand::PlaySoundAt {
                    path,
                    position,
                    volume,
                } => {
                    // Create temporary audio source entity
                    use crate::audio::AudioSource;
                    use crate::core::entity::Transform;

                    let _entity = world.spawn((
                        Transform::from_position(position),
                        AudioSource {
                            sound_path: path,
                            volume,
                            auto_play: true,
                            looping: false,
                            spatial: true,
                            ..Default::default()
                        },
                    ));
                    debug!("Created temporary audio source at {:?}", position);
                }
            }
        }
    });
}

/// Create the audio scripting module
pub fn create_audio_module(audio_engine: Arc<Mutex<AudioEngine>>) -> Module {
    let mut module = Module::new();
    let script_engine = ScriptAudioEngine::new(audio_engine);

    // Basic sound playback
    {
        let engine = script_engine.clone();
        module.set_native_fn(
            "play_sound",
            move |path: &str| -> Result<Dynamic, Box<EvalAltResult>> {
                engine
                    .play_sound(path)
                    .map(|handle| Dynamic::from(handle as AudioHandle))
            },
        );
    }

    // Play sound with settings
    {
        let engine = script_engine.clone();
        module.set_native_fn(
            "play_sound_ex",
            move |path: &str,
                  volume: f32,
                  pitch: f32,
                  looping: bool|
                  -> Result<Dynamic, Box<EvalAltResult>> {
                engine
                    .play_sound_with_settings(path, volume, pitch, looping)
                    .map(|handle| Dynamic::from(handle as AudioHandle))
            },
        );
    }

    // Play one-shot sound
    {
        let engine = script_engine.clone();
        module.set_native_fn(
            "play_one_shot",
            move |path: &str, volume: f32| -> Result<(), Box<EvalAltResult>> {
                engine.play_one_shot(path, volume)
            },
        );
    }

    // Play sound at position
    // TODO: Fix Rhai trait bounds
    // module.set_native_fn(
    //     "play_sound_at",
    //     move |path: &str, x: f32, y: f32, z: f32, volume: f32| {
    //         queue_audio_command(AudioCommand::PlaySoundAt {
    //             path: path.to_string(),
    //             position: Vec3::new(x, y, z),
    //             volume,
    //         });
    //         debug!(
    //             "Queued sound at position: {} at ({}, {}, {})",
    //             path, x, y, z
    //         );
    //     },
    // );

    // Sound handle control
    // TODO: Fix Rhai trait bounds for AudioHandle methods
    // module.set_native_fn("stop_sound", move |handle: &mut AudioHandle| {
    //     handle.stop(None);
    // });

    // module.set_native_fn(
    //     "stop_sound_fade",
    //     move |handle: &mut AudioHandle, fade_time: f32| {
    //         handle.stop(Some(std::time::Duration::from_secs_f32(fade_time)));
    //     },
    // );

    // module.set_native_fn(
    //     "set_volume",
    //     move |handle: &mut AudioHandle, volume: f32| {
    //         handle.set_volume(volume, None);
    //     },
    // );

    // module.set_native_fn("set_pitch", move |handle: &mut AudioHandle, pitch: f32| {
    //     handle.set_playback_rate(pitch, None);
    // });

    // module.set_native_fn("is_playing", move |handle: &AudioHandle| -> bool {
    //     handle.is_playing()
    // });

    // Audio material control
    // TODO: Fix Rhai trait bounds
    // module.set_native_fn(
    //     "set_audio_material",
    //     move |entity: u64, absorption: f32, transmission: f32| {
    //         queue_audio_command(AudioCommand::SetAudioMaterial {
    //             entity,
    //             absorption,
    //             transmission,
    //         });
    //     },
    // );

    // Material presets
    // TODO: Fix Rhai trait bounds for material functions
    // module.set_native_fn("audio_material_concrete", move || {
    //     let mat = AudioMaterial::concrete();
    //     (mat.absorption, mat.transmission)
    // });

    // module.set_native_fn("audio_material_wood", move || {
    //     let mat = AudioMaterial::wood();
    //     (mat.absorption, mat.transmission)
    // });

    // module.set_native_fn("audio_material_glass", move || {
    //     let mat = AudioMaterial::glass();
    //     (mat.absorption, mat.transmission)
    // });

    // module.set_native_fn("audio_material_fabric", move || {
    //     let mat = AudioMaterial::fabric();
    //     (mat.absorption, mat.transmission)
    // });

    // module.set_native_fn("audio_material_metal", move || {
    //     let mat = AudioMaterial::metal();
    //     (mat.absorption, mat.transmission)
    // });

    module
}
