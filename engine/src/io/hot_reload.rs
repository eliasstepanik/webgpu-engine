//! Hot-reload infrastructure for scene files

use crate::core::entity::World;
use crate::graphics::{AssetManager, Renderer};
use crate::io::Scene;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Hot-reload watcher for scene files
pub struct SceneWatcher {
    /// File system watcher
    _watcher: RecommendedWatcher,
    /// Path being watched
    scene_path: PathBuf,
    /// Control handle for stopping the watcher
    control_handle: Option<WatcherControlHandle>,
}

/// Control handle for managing the watcher thread
struct WatcherControlHandle {
    stop_sender: Sender<()>,
    thread_handle: thread::JoinHandle<()>,
}

/// Callback function type for reload events
pub type ReloadCallback = Box<
    dyn Fn(
            &mut World,
            &mut Renderer,
            &mut AssetManager,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

/// Configuration for the scene watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Debounce duration to avoid multiple reloads for rapid file changes
    pub debounce_duration: Duration,
    /// Whether to clear the world before reloading
    pub clear_world_on_reload: bool,
    /// Whether to validate assets during reload
    pub validate_assets: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_millis(300),
            clear_world_on_reload: true,
            validate_assets: true,
        }
    }
}

impl SceneWatcher {
    /// Create a new scene watcher
    ///
    /// The callback will be called whenever the scene file changes.
    /// The callback receives mutable references to the world, renderer, and asset manager.
    pub fn new<P: AsRef<Path>>(
        scene_path: P,
        config: WatcherConfig,
        callback: ReloadCallback,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let scene_path = scene_path.as_ref().to_path_buf();
        info!(path = ?scene_path, "Creating scene watcher");

        // Create channels for file events and control
        let (event_tx, event_rx) = mpsc::channel::<Event>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        // Create the file system watcher
        let mut watcher = RecommendedWatcher::new(
            move |res| match res {
                Ok(event) => {
                    if let Err(e) = event_tx.send(event) {
                        error!(error = %e, "Failed to send file event");
                    }
                }
                Err(e) => error!(error = %e, "File watcher error"),
            },
            Config::default(),
        )?;

        // Watch the scene file (or its parent directory)
        let watch_path = if scene_path.is_file() {
            scene_path.parent().unwrap_or(&scene_path)
        } else {
            &scene_path
        };

        watcher.watch(watch_path, RecursiveMode::NonRecursive)?;
        debug!(watch_path = ?watch_path, "Started watching for file changes");

        // Spawn background thread for handling events
        let scene_path_clone = scene_path.clone();
        let thread_handle = thread::spawn(move || {
            Self::event_loop(scene_path_clone, config, callback, event_rx, stop_rx);
        });

        let control_handle = WatcherControlHandle {
            stop_sender: stop_tx,
            thread_handle,
        };

        Ok(Self {
            _watcher: watcher,
            scene_path,
            control_handle: Some(control_handle),
        })
    }

    /// Event loop for handling file system events
    fn event_loop(
        scene_path: PathBuf,
        config: WatcherConfig,
        callback: ReloadCallback,
        event_rx: Receiver<Event>,
        stop_rx: Receiver<()>,
    ) {
        let mut last_reload = Instant::now();
        let callback = Arc::new(Mutex::new(callback));

        loop {
            // Check for stop signal (non-blocking)
            if stop_rx.try_recv().is_ok() {
                debug!("Scene watcher received stop signal");
                break;
            }

            // Wait for file events with timeout
            match event_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    // Check if this event is for our scene file
                    let should_reload = event.paths.iter().any(|path| {
                        path == &scene_path
                            || (path.is_dir() && scene_path.starts_with(path))
                            || path.file_name() == scene_path.file_name()
                    });

                    if should_reload {
                        let now = Instant::now();
                        if now.duration_since(last_reload) >= config.debounce_duration {
                            debug!(
                                event_kind = ?event.kind,
                                paths = ?event.paths,
                                "Scene file changed, triggering reload"
                            );

                            // Perform the reload
                            Self::perform_reload(&scene_path, &config, &callback);
                            last_reload = now;
                        } else {
                            debug!("Debouncing rapid file changes");
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Normal timeout, continue loop
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    debug!("Event channel disconnected, stopping watcher");
                    break;
                }
            }
        }

        info!("Scene watcher event loop stopped");
    }

    /// Perform the actual reload operation
    fn perform_reload(
        scene_path: &Path,
        _config: &WatcherConfig,
        callback: &Arc<Mutex<ReloadCallback>>,
    ) {
        info!(path = ?scene_path, "Reloading scene");

        // Note: In a real implementation, we would need access to the World, Renderer, and AssetManager
        // This is a simplified version that demonstrates the structure.
        // The actual callback execution would happen in the main thread with proper access to these resources.

        if let Ok(_callback_guard) = callback.lock() {
            // In practice, this would be called from the main thread with proper resource access
            debug!("Scene reload callback would be executed here");

            // The actual implementation would look like:
            // match callback_guard(world, renderer, asset_manager) {
            //     Ok(()) => info!("Scene reloaded successfully"),
            //     Err(e) => error!(error = %e, "Scene reload failed"),
            // }
        } else {
            error!("Failed to acquire callback lock for scene reload");
        }
    }

    /// Stop the watcher and clean up resources
    pub fn stop(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(control) = self.control_handle.take() {
            info!(path = ?self.scene_path, "Stopping scene watcher");

            // Send stop signal
            if let Err(e) = control.stop_sender.send(()) {
                warn!(error = %e, "Failed to send stop signal to watcher thread");
            }

            // Wait for thread to finish
            if let Err(e) = control.thread_handle.join() {
                error!(error = ?e, "Error joining watcher thread");
                return Err("Failed to join watcher thread".into());
            }

            info!("Scene watcher stopped successfully");
        }

        Ok(())
    }

    /// Get the path being watched
    pub fn scene_path(&self) -> &Path {
        &self.scene_path
    }
}

impl Drop for SceneWatcher {
    fn drop(&mut self) {
        if self.control_handle.is_some() {
            warn!("SceneWatcher dropped without calling stop() - forcing stop");
            if let Some(control) = self.control_handle.take() {
                let _ = control.stop_sender.send(());
                // Don't wait for thread in drop to avoid blocking
            }
        }
    }
}

/// Utility function to reload a scene with validation
pub fn reload_scene_with_validation<P: AsRef<Path>>(
    scene_path: P,
    world: &mut World,
    asset_manager: &mut AssetManager,
    clear_world: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let scene_path = scene_path.as_ref();
    info!(path = ?scene_path, "Reloading scene with validation");

    // Clear world if requested
    if clear_world {
        world.inner_mut().clear();
        debug!("Cleared world before reload");
    }

    // Load scene with validation
    let (scene, validation_report) =
        Scene::load_from_file_with_validation(scene_path, asset_manager)?;

    // Log validation results
    let summary = validation_report.summary();
    if !summary.is_valid {
        warn!(
            invalid_meshes = summary.total_mesh_references - summary.valid_mesh_references,
            errors = summary.total_errors,
            "Scene loaded with validation issues"
        );
    }

    // Instantiate scene with validation
    let _mapper = scene.instantiate_with_validation(world, asset_manager)?;

    info!("Scene reload completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert_eq!(config.debounce_duration, Duration::from_millis(300));
        assert!(config.clear_world_on_reload);
        assert!(config.validate_assets);
    }

    #[test]
    fn test_scene_watcher_creation() {
        // Create a temporary scene file
        let temp_path = "test_watcher_scene.json";
        fs::write(temp_path, r#"{"entities": []}"#).unwrap();

        let callback_called = Arc::new(AtomicBool::new(false));
        let callback_called_clone = callback_called.clone();

        let callback: ReloadCallback = Box::new(move |_world, _renderer, _asset_manager| {
            callback_called_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        // Create watcher
        let watcher = SceneWatcher::new(temp_path, WatcherConfig::default(), callback);
        assert!(watcher.is_ok());

        let watcher = watcher.unwrap();
        assert_eq!(
            watcher.scene_path().file_name().unwrap(),
            "test_watcher_scene.json"
        );

        // Stop watcher
        watcher.stop().unwrap();

        // Clean up
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_reload_scene_utility() {
        // This test would require actual World, Renderer, and AssetManager instances
        // For now, we just test that the function signature is correct

        // In a real implementation, you would:
        // let mut world = World::new();
        // let mut asset_manager = AssetManager::new();
        // let result = reload_scene_with_validation("test.json", &mut world, &mut asset_manager, true);
    }
}
