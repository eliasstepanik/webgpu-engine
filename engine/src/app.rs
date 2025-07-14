//! Application management for the engine

use crate::config::AssetConfig;
use crate::core::entity::{update_hierarchy_system, World};
use crate::graphics::{RenderContext, Renderer};
use crate::input::InputState;
use crate::scripting::ScriptEngine;
use crate::windowing::WindowManager;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

/// Configuration for engine initialization
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Window title
    pub window_title: String,
    /// Window size (None = use primary monitor size)
    pub window_size: Option<(u32, u32)>,
    /// Asset configuration
    pub asset_config: AssetConfig,
    /// Enable scripting
    pub enable_scripting: bool,
    /// Custom logging filter (None = default)
    pub log_filter: Option<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_title: "WebGPU Engine".to_string(),
            window_size: None,
            asset_config: AssetConfig::default(),
            enable_scripting: true,
            log_filter: None,
        }
    }
}

/// Main engine application struct that implements ApplicationHandler
pub struct EngineApp {
    /// Window manager for multi-window support
    pub window_manager: Option<WindowManager>,
    /// Render context (shared GPU resources)
    pub render_context: Option<Arc<RenderContext>>,
    /// Renderer for the game
    pub renderer: Option<Renderer>,
    /// ECS world
    pub world: World,
    /// Script engine
    pub script_engine: Option<ScriptEngine>,
    /// Input state
    pub input_state: InputState,

    // Private fields
    config: EngineConfig,
    instance: Option<Arc<wgpu::Instance>>,
    last_time: std::time::Instant,
    focus_tracker: HashMap<WindowId, bool>,
    last_focused_window: Option<WindowId>,
    initialized: bool,
}

impl EngineApp {
    /// Create a new engine app with default configuration
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }

    /// Create a new engine app with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        // Initialize logging if log filter is provided
        if let Some(filter) = &config.log_filter {
            std::env::set_var("RUST_LOG", filter);
        }
        crate::init_logging();

        info!("Creating EngineApp with config: {:?}", config);

        Self {
            window_manager: None,
            render_context: None,
            renderer: None,
            world: World::new(),
            script_engine: None,
            input_state: InputState::new(),
            config,
            instance: None,
            last_time: std::time::Instant::now(),
            focus_tracker: HashMap::new(),
            last_focused_window: None,
            initialized: false,
        }
    }

    /// Check if the engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Initialize the engine (called when event loop is ready)
    pub fn init(&mut self, event_loop: &ActiveEventLoop) {
        if self.initialized {
            return;
        }

        info!("Initializing EngineApp");

        // Determine window size
        let (width, height) = self.config.window_size.unwrap_or_else(|| {
            event_loop
                .primary_monitor()
                .map(|monitor| {
                    let size = monitor.size();
                    info!(
                        "Using primary monitor resolution: {}x{}",
                        size.width, size.height
                    );
                    (size.width, size.height)
                })
                .unwrap_or_else(|| {
                    info!("No primary monitor found, using default size 1280x720");
                    (1280, 720)
                })
        });

        // Create main window
        let window_attributes = WindowAttributes::default()
            .with_title(self.config.window_title.clone())
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        // Create WebGPU instance
        let instance = Arc::new(wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        }));

        // Initialize render context
        let render_context = pollster::block_on(RenderContext::new((*instance).clone(), None))
            .expect("Failed to create render context");
        let render_context = Arc::new(render_context);

        // Create surface for the main window
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        // Create surface configuration
        let surface_config = render_context.create_surface_configuration(
            &surface,
            window.inner_size().width,
            window.inner_size().height,
        );

        // Create window manager
        let mut window_manager = WindowManager::new(
            window.clone(),
            instance.clone(),
            render_context.device.clone(),
            surface_config.clone(),
        )
        .expect("Failed to create window manager");

        // Set up the main window with its surface
        window_manager
            .set_main_window(window.clone(), surface, surface_config.clone())
            .expect("Failed to set up main window");

        let mut renderer = Renderer::new(render_context.clone());

        // Resize renderer to match window size
        renderer.resize(window.inner_size());

        // Initialize script engine if enabled
        let script_engine = if self.config.enable_scripting {
            let mut engine = ScriptEngine::with_config(self.config.asset_config.clone());
            crate::scripting::system::initialize_script_engine(&mut engine);
            Some(engine)
        } else {
            None
        };

        // Store initialized components
        self.instance = Some(instance);
        self.window_manager = Some(window_manager);
        self.render_context = Some(render_context);
        self.renderer = Some(renderer);
        self.script_engine = script_engine;
        self.initialized = true;
    }

    /// Update the engine state
    pub fn update(&mut self, delta_time: f32) {
        // Clear per-frame input data
        self.input_state.clear_frame_data();

        // Execute scripts
        if let Some(script_engine) = &mut self.script_engine {
            let script_input_state = self.input_state.to_script_input_state();

            // Initialize script properties for new scripts
            crate::scripting::script_initialization_system(&mut self.world, script_engine);

            // Execute scripts
            crate::scripting::script_execution_system(
                &mut self.world,
                script_engine,
                &script_input_state,
                delta_time,
            );
        }

        update_hierarchy_system(&mut self.world);
    }

    fn handle_resize(&mut self, window_id: WindowId, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        info!("Window resized to {:?}", new_size);

        // Resize in window manager
        if let Some(window_manager) = &mut self.window_manager {
            window_manager.resize_window(window_id, new_size);
        }

        // Only resize renderer if this is the main window
        if let Some(window_manager) = &self.window_manager {
            if window_id == window_manager.main_window_id() {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }

                // Update camera aspect ratio
                let aspect_ratio = new_size.width as f32 / new_size.height as f32;
                for (_, camera) in self.world.query_mut::<&mut crate::core::camera::Camera>() {
                    camera.set_aspect_ratio(aspect_ratio);
                }
            }
        }
    }

    /// Render a frame
    pub fn render_frame(&mut self, window_id: WindowId) {
        let Some(window_manager) = &self.window_manager else {
            return;
        };
        let Some(window_data) = window_manager.get_window(window_id) else {
            return;
        };

        // Skip rendering if window is minimized
        if window_manager.is_window_minimized(window_id) {
            return;
        }

        // Only render main window for now
        if window_id != window_manager.main_window_id() {
            return;
        }

        if let Some(renderer) = &mut self.renderer {
            // Render frame
            match renderer.render(&self.world, &window_data.surface) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    info!("Surface lost or outdated, reconfiguring");
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    std::process::exit(1);
                }
                Err(e) => {
                    error!(error = ?e, "Render error");
                }
            }
        }
    }
}

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize on first resume
        if !self.initialized {
            self.init(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window_manager) = &self.window_manager else {
            return;
        };
        let Some(window_data) = window_manager.get_window(window_id) else {
            return;
        };

        match event {
            WindowEvent::Focused(focused) => {
                self.focus_tracker.insert(window_id, focused);

                // Platform-specific focus handling
                #[cfg(target_os = "windows")]
                {
                    if focused && self.last_focused_window != Some(window_id) {
                        // Force update all other windows as unfocused
                        for (wid, focus) in self.focus_tracker.iter_mut() {
                            if *wid != window_id {
                                *focus = false;
                            }
                        }
                    }
                }

                self.last_focused_window = if focused { Some(window_id) } else { None };

                // Ensure main window continues processing
                if window_id == window_manager.main_window_id() && !focused {
                    window_data.window.request_redraw();
                }

                debug!(
                    window_id = ?window_id,
                    focused = focused,
                    main_window = window_id == window_manager.main_window_id(),
                    "Window focus changed"
                );
            }
            WindowEvent::CloseRequested => {
                if window_id == window_manager.main_window_id() {
                    info!("Main window close requested");
                    event_loop.exit();
                } else {
                    // Handle closing secondary windows
                    info!("Secondary window close requested: {:?}", window_id);
                }
            }
            WindowEvent::Resized(physical_size) => {
                self.handle_resize(window_id, physical_size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                info!("Scale factor changed to {}", scale_factor);
                let new_size = window_data.window.inner_size();
                self.handle_resize(window_id, new_size);
            }
            WindowEvent::RedrawRequested => {
                // Update time
                let current_time = std::time::Instant::now();
                let delta_time = (current_time - self.last_time).as_secs_f32();
                self.last_time = current_time;

                // Update engine state
                self.update(delta_time);

                // Render frame
                self.render_frame(window_id);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_state.handle_keyboard_event(&event);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = (position.x as f32, position.y as f32);
                let old_pos = self.input_state.mouse_position;
                self.input_state.set_mouse_position(new_pos.0, new_pos.1);
                self.input_state
                    .add_mouse_delta(new_pos.0 - old_pos.0, new_pos.1 - old_pos.1);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.input_state.handle_mouse_button(button, state);
            }
            WindowEvent::MouseWheel { .. } => {
                // Mouse wheel events can be handled here if needed
            }
            _ => {}
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        // Request redraw for all windows
        if let Some(window_manager) = &self.window_manager {
            for window_id in window_manager.window_ids() {
                if let Some(window_data) = window_manager.get_window(*window_id) {
                    window_data.window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw for continuous rendering
        if let Some(window_manager) = &self.window_manager {
            for window_id in window_manager.window_ids() {
                if let Some(window_data) = window_manager.get_window(*window_id) {
                    window_data.window.request_redraw();
                }
            }
        }
    }
}

/// Builder pattern for EngineApp configuration
pub struct EngineBuilder {
    config: EngineConfig,
}

impl EngineBuilder {
    /// Create a new engine builder
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    /// Set the window title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.window_title = title.into();
        self
    }

    /// Set the window size
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.config.window_size = Some((width, height));
        self
    }

    /// Set the asset configuration
    pub fn asset_config(mut self, config: AssetConfig) -> Self {
        self.config.asset_config = config;
        self
    }

    /// Enable or disable scripting
    pub fn with_scripting(mut self, enable: bool) -> Self {
        self.config.enable_scripting = enable;
        self
    }

    /// Set a custom log filter
    pub fn log_filter(mut self, filter: impl Into<String>) -> Self {
        self.config.log_filter = Some(filter.into());
        self
    }

    /// Build the EngineApp
    pub fn build(self) -> EngineApp {
        EngineApp::with_config(self.config)
    }
}

impl Default for EngineApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
