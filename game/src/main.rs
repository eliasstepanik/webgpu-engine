//! Game entry point with WebGPU rendering demonstration

use engine::prelude::*;
use std::path::PathBuf;
use tracing::{debug, info};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

#[cfg(feature = "editor")]
use editor::{EditorState, SceneOperation};

/// Game application that wraps EngineApp with game-specific functionality
struct GameApp {
    /// Core engine application
    engine: EngineApp,
    /// Editor state (when editor feature is enabled)
    #[cfg(feature = "editor")]
    editor_state: Option<EditorState>,
    /// Track if we need to initialize editor
    #[cfg(feature = "editor")]
    editor_initialized: bool,
    /// Time tracking
    last_time: std::time::Instant,
}

impl GameApp {
    fn new() -> Self {
        // Configure asset paths to point to game/assets
        let asset_config = AssetConfig::new(
            PathBuf::from("game/assets"),
            "scripts".to_string(),
            "scenes".to_string(),
        );

        // Create engine configuration
        let engine_config = EngineBuilder::new()
            .title("WebGPU Game Engine Demo")
            .asset_config(asset_config)
            .with_scripting(true)
            .build();

        Self {
            engine: engine_config,
            #[cfg(feature = "editor")]
            editor_state: None,
            #[cfg(feature = "editor")]
            editor_initialized: false,
            last_time: std::time::Instant::now(),
        }
    }

    /// Initialize editor after engine is ready
    #[cfg(feature = "editor")]
    fn init_editor(&mut self) {
        if self.editor_initialized || self.editor_state.is_some() {
            return;
        }

        let Some(window_manager) = &self.engine.window_manager else {
            return;
        };
        let Some(render_context) = &self.engine.render_context else {
            return;
        };

        let window_data = window_manager
            .get_window(window_manager.main_window_id())
            .expect("Main window should exist");

        let window_size = window_data.window.inner_size();

        // Move the world to the editor's shared state
        let world = std::mem::replace(&mut self.engine.world, World::new());

        // Get surface config from window data
        let surface_config = window_data.surface_config.clone();

        self.editor_state = Some(EditorState::new(
            render_context,
            &window_data.window,
            surface_config.format,
            (window_size.width, window_size.height),
            world,
        ));

        self.editor_initialized = true;
        info!("Editor initialized");
    }

    #[cfg(feature = "editor")]
    fn render_frame(&mut self, window_id: WindowId) {
        // Render with editor if enabled
        if self.editor_state.is_some() {
            self.render_with_editor(window_id);
            return;
        }

        // Otherwise use standard engine rendering
        self.engine.render_frame(window_id);
    }

    #[cfg(feature = "editor")]
    fn render_with_editor(&mut self, window_id: WindowId) {
        let Some(editor_state) = &mut self.editor_state else {
            return;
        };
        let Some(window_manager) = &self.engine.window_manager else {
            return;
        };
        let Some(window_data) = window_manager.get_window(window_id) else {
            return;
        };
        let Some(render_context) = &self.engine.render_context else {
            return;
        };
        let Some(renderer) = &mut self.engine.renderer else {
            return;
        };

        // Skip rendering if window is minimized
        if window_manager.is_window_minimized(window_id) {
            return;
        }

        // Only render main window
        if window_id != window_manager.main_window_id() {
            return;
        }

        // Handle pending scene operations
        if let Some(operation) = editor_state.pending_scene_operation.take() {
            debug!(operation = ?operation, "Processing scene operation");
            match operation {
                SceneOperation::NewScene => {
                    debug!("Creating new default scene");
                    editor_state.shared_state.with_world_write(|world| {
                        editor::scene_operations::create_default_scene(world, renderer);
                    });
                }
                SceneOperation::LoadScene(path) => {
                    let result = editor_state.shared_state.with_world_write(|world| {
                        editor::scene_operations::load_scene_from_file(world, renderer, &path)
                    });
                    match result.unwrap_or(Err("Failed to access world".into())) {
                        Ok(_) => info!("Scene loaded successfully"),
                        Err(e) => {
                            tracing::error!("Failed to load scene: {:?}", e);
                            editor_state.error_message = Some(format!("Failed to load scene: {e}"));
                        }
                    }
                }
                SceneOperation::SaveScene(path) => {
                    let result = editor_state.shared_state.with_world_read(|world| {
                        editor::scene_operations::save_scene_to_file(world, &path)
                    });
                    match result.unwrap_or(Err("Failed to access world".into())) {
                        Ok(_) => info!("Scene saved successfully"),
                        Err(e) => {
                            tracing::error!("Failed to save scene: {:?}", e);
                            editor_state.error_message = Some(format!("Failed to save scene: {e}"));
                        }
                    }
                }
            }
        }

        // Begin editor frame
        editor_state.begin_frame(&window_data.window, render_context);

        // Render game to viewport texture
        let shared_state = editor_state.shared_state.clone();
        shared_state.with_world_read(|world| {
            editor_state.render_viewport(renderer, world);
        });

        // Get surface texture for final rendering
        let surface_texture = match window_data.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Outdated) => {
                info!("Surface outdated, reconfiguring");
                return;
            }
            Err(e) => {
                tracing::error!("Failed to get surface texture: {:?}", e);
                return;
            }
        };

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder for ImGui rendering
        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ImGui Render Encoder"),
                });

        // Clear the surface first
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render editor UI and ImGui to screen
        editor_state.render_ui_and_draw(
            render_context,
            &mut encoder,
            &view,
            &window_data.window,
            window_manager,
        );

        // Submit commands
        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize engine first
        if !self.engine.is_initialized() {
            self.engine.init(event_loop);

            // Check for SCENE environment variable
            if let Ok(scene_name) = std::env::var("SCENE") {
                info!("Loading scene from environment variable: {}", scene_name);

                if let Some(renderer) = &mut self.engine.renderer {
                    // Build the scene path
                    let scene_path = if scene_name.ends_with(".json") {
                        PathBuf::from("game/assets/scenes").join(&scene_name)
                    } else {
                        PathBuf::from("game/assets/scenes").join(format!("{scene_name}.json"))
                    };

                    // Try to load the scene
                    match engine::io::Scene::load_from_file(&scene_path) {
                        Ok(scene) => match scene.instantiate(&mut self.engine.world) {
                            Ok(_) => {
                                info!("Successfully loaded scene: {}", scene_path.display());
                            }
                            Err(e) => {
                                tracing::error!("Failed to instantiate scene: {}", e);
                                info!("Falling back to demo scene");
                                create_demo_scene(&mut self.engine.world, renderer);
                            }
                        },
                        Err(e) => {
                            tracing::error!(
                                "Failed to load scene from {}: {}",
                                scene_path.display(),
                                e
                            );
                            info!("Falling back to demo scene");
                            create_demo_scene(&mut self.engine.world, renderer);
                        }
                    }
                } else {
                    tracing::error!("Renderer not available to load scene");
                }
            } else {
                // Create demo scene if no environment variable is set
                if let Some(renderer) = &mut self.engine.renderer {
                    create_demo_scene(&mut self.engine.world, renderer);
                }
            }
        }

        // Initialize editor after engine
        #[cfg(feature = "editor")]
        {
            self.init_editor();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Let editor handle events first if enabled
        #[cfg(feature = "editor")]
        {
            if let Some(editor_state) = &mut self.editor_state {
                if let Some(window_manager) = &self.engine.window_manager {
                    if let Some(window_data) = window_manager.get_window(window_id) {
                        let window_event = winit::event::Event::WindowEvent {
                            event: event.clone(),
                            window_id,
                        };

                        let should_consume =
                            editor_state.handle_event(&window_data.window, &window_event);

                        // Don't consume critical events
                        if should_consume && !matches!(event, WindowEvent::RedrawRequested) {
                            return;
                        }
                    }
                }
            }
        }

        // Handle close request specially for editor cleanup
        if matches!(event, WindowEvent::CloseRequested) {
            if let Some(window_manager) = &self.engine.window_manager {
                if window_id == window_manager.main_window_id() {
                    info!("Main window close requested");

                    // Clean up editor resources before exit
                    #[cfg(feature = "editor")]
                    {
                        if let (Some(editor_state), Some(window_manager)) =
                            (&mut self.editor_state, &mut self.engine.window_manager)
                        {
                            editor_state.shutdown(window_manager);
                        }
                    }

                    event_loop.exit();
                    return;
                }
            }
        }

        // Special handling for RedrawRequested when editor is active
        if matches!(event, WindowEvent::RedrawRequested) {
            // Advance to next frame for hierarchy update tracking
            engine::core::entity::hierarchy::advance_frame();

            // Update time and engine state
            let current_time = std::time::Instant::now();
            let delta_time = (current_time - self.last_time).as_secs_f32();
            self.last_time = current_time;

            // Update through editor's shared state if editor is active
            #[cfg(feature = "editor")]
            {
                if let (Some(editor_state), Some(script_engine)) =
                    (&self.editor_state, &mut self.engine.script_engine)
                {
                    let script_input_state = self.engine.input_state.to_script_input_state();

                    editor_state.shared_state.with_world_write(|world| {
                        // Initialize script properties for new scripts
                        engine::scripting::script_initialization_system(world, script_engine);

                        // Execute scripts
                        engine::scripting::script_execution_system(
                            world,
                            script_engine,
                            &script_input_state,
                            delta_time,
                        );

                        update_hierarchy_system(world);
                    });

                    // Render with editor
                    self.render_frame(window_id);
                    return;
                }
            }

            // Otherwise use standard engine update
            self.engine.update(delta_time);
        }

        // Pass other events to engine
        self.engine.window_event(event_loop, window_id, event);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.engine.new_events(event_loop, cause);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.engine.about_to_wait(event_loop);
    }
}

fn main() {
    // Create event loop
    let event_loop = EventLoop::builder()
        .build()
        .expect("Failed to create event loop");

    // Create game application
    let mut app = GameApp::new();

    info!("Starting WebGPU game");

    // Run event loop
    event_loop.run_app(&mut app).expect("Failed to run app");
}

/// Create a demo scene with a rotating cube
fn create_demo_scene(world: &mut World, renderer: &mut Renderer) {
    info!("Creating demo scene");

    // Create camera
    let camera_entity = world.spawn((
        Name::new("Main Camera"),
        Camera::perspective(60.0, 16.0 / 9.0, 0.1, 1_000_000_000.0), // 1 billion units far plane
        Transform::from_position(Vec3::new(0.0, 2.0, 5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));
    info!("Created camera entity: {:?}", camera_entity);

    // Create a cube
    let cube_mesh = Mesh::cube(1.0);
    let cube_mesh_id = renderer.upload_mesh(&cube_mesh, "cube");

    let cube_entity = world.spawn((
        Name::new("Center Cube"),
        cube_mesh_id.clone(),
        Material::red(),
        Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    ));
    info!("Created cube entity: {:?}", cube_entity);

    // Create a plane
    let plane_mesh = Mesh::plane(10.0, 10.0);
    let plane_mesh_id = renderer.upload_mesh(&plane_mesh, "plane");

    let plane_entity = world.spawn((
        Name::new("Ground Plane"),
        plane_mesh_id,
        Material::gray(0.5),
        Transform::from_position(Vec3::new(0.0, -1.0, 0.0)),
        GlobalTransform::default(),
    ));
    info!("Created plane entity: {:?}", plane_entity);

    // Create additional cubes in a circle
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
        let x = angle.cos() * 3.0;
        let z = angle.sin() * 3.0;

        let color = match i % 3 {
            0 => Material::blue(),
            1 => Material::green(),
            _ => Material::from_rgb(1.0, 1.0, 0.0), // Yellow
        };

        let color_name = match i % 3 {
            0 => "Blue",
            1 => "Green",
            _ => "Yellow",
        };

        world.spawn((
            Name::new(format!("Orbital Cube {} ({})", i + 1, color_name)),
            cube_mesh_id.clone(),
            color,
            Transform::from_position(Vec3::new(x, 0.0, z)).with_scale(Vec3::splat(0.5)),
            GlobalTransform::default(),
        ));
    }
}
