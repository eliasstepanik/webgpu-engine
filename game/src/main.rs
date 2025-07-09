//! Game entry point with WebGPU rendering demonstration

use engine::prelude::*;
use engine::windowing::WindowManager;
use std::sync::Arc;
use tracing::info;
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{WindowAttributes, WindowId},
};

#[cfg(feature = "editor")]
use editor::{EditorState, SceneOperation};

/// Application state implementing the new ApplicationHandler trait
struct App {
    /// Window manager for multi-window support
    window_manager: Option<WindowManager>,
    /// Render context (shared GPU resources)
    render_context: Option<Arc<RenderContext<'static>>>,
    /// Renderer for the game
    renderer: Option<Renderer<'static>>,
    /// ECS world
    world: World,
    /// Time tracking
    last_time: std::time::Instant,
    /// Editor state
    #[cfg(feature = "editor")]
    editor_state: Option<EditorState>,
    /// WebGPU instance
    instance: Option<Arc<wgpu::Instance>>,
}

impl App {
    fn new() -> Self {
        Self {
            window_manager: None,
            render_context: None,
            renderer: None,
            world: World::new(),
            last_time: std::time::Instant::now(),
            #[cfg(feature = "editor")]
            editor_state: None,
            instance: None,
        }
    }

    /// Initialize the application after the event loop is ready
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        info!("Initializing application");

        // Create main window
        let window_attributes = WindowAttributes::default()
            .with_title("WebGPU Game Engine Demo")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window"),
        );

        // Create WebGPU instance (needed for WindowManager)
        let instance = Arc::new(wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        }));

        // Initialize renderer
        // HACK: We leak the window Arc to get a 'static reference
        // This is not ideal but works around the lifetime issue with RenderContext
        // TODO: Refactor RenderContext to not own the surface
        let window_static: &'static winit::window::Window = Box::leak(Box::new(window.clone()));
        let render_context = pollster::block_on(RenderContext::new(window_static))
            .expect("Failed to create render context");
        let render_context = Arc::new(render_context);

        // Create window manager
        let window_manager = WindowManager::new(
            window.clone(),
            instance.clone(),
            render_context.device.clone(),
            render_context.surface_config.lock().unwrap().clone(),
        )
        .expect("Failed to create window manager");

        let mut renderer = Renderer::new(render_context.clone());

        // Create demo scene
        create_demo_scene(&mut self.world, &mut renderer);

        // Create editor state if feature is enabled
        #[cfg(feature = "editor")]
        let editor_state = EditorState::new(&render_context, &window);

        // Store initialized components
        self.instance = Some(instance);
        self.window_manager = Some(window_manager);
        self.render_context = Some(render_context);
        self.renderer = Some(renderer);
        #[cfg(feature = "editor")]
        {
            self.editor_state = Some(editor_state);
        }
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

                // Resize editor viewport if enabled
                #[cfg(feature = "editor")]
                if let (Some(editor_state), Some(render_context)) =
                    (&mut self.editor_state, &self.render_context)
                {
                    editor_state.resize(render_context, new_size);
                }

                // Update camera aspect ratio
                for (_, camera) in self.world.query_mut::<&mut Camera>() {
                    camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                }
            }
        }
    }

    fn render_frame(&mut self, window_id: WindowId) {
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

        // Update time
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - self.last_time).as_secs_f32();
        self.last_time = current_time;

        // Update demo scene
        update_demo_scene(&mut self.world, delta_time);

        // Update transform hierarchy
        update_hierarchy_system(&mut self.world);

        // Render based on editor mode
        #[cfg(feature = "editor")]
        {
            if let (Some(editor_state), Some(render_context), Some(renderer)) = (
                &mut self.editor_state,
                &self.render_context,
                &mut self.renderer,
            ) {
                // Handle pending scene operations
                if let Some(operation) = editor_state.pending_scene_operation.take() {
                    match operation {
                        SceneOperation::NewScene => {
                            editor::scene_operations::create_default_scene(
                                &mut self.world,
                                renderer,
                            );
                        }
                        SceneOperation::LoadScene(path) => {
                            match editor::scene_operations::load_scene_from_file(
                                &mut self.world,
                                renderer,
                                &path,
                            ) {
                                Ok(_) => info!("Scene loaded successfully"),
                                Err(e) => {
                                    tracing::error!("Failed to load scene: {:?}", e);
                                    editor_state.error_message =
                                        Some(format!("Failed to load scene: {e}"));
                                }
                            }
                        }
                        SceneOperation::SaveScene(path) => {
                            match editor::scene_operations::save_scene_to_file(&self.world, &path) {
                                Ok(_) => info!("Scene saved successfully"),
                                Err(e) => {
                                    tracing::error!("Failed to save scene: {:?}", e);
                                    editor_state.error_message =
                                        Some(format!("Failed to save scene: {e}"));
                                }
                            }
                        }
                    }
                }

                // Begin editor frame FIRST
                editor_state.begin_frame(&window_data.window, render_context);

                // Render game to viewport texture
                editor_state.render_viewport(renderer, &self.world);

                // Get surface texture for final rendering
                let surface_texture = match window_data.surface.get_current_texture() {
                    Ok(texture) => texture,
                    Err(wgpu::SurfaceError::Outdated) => {
                        // Surface is outdated, likely due to resize
                        info!("Surface outdated, reconfiguring");
                        return; // Will trigger resize on next frame
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
                    // Render pass automatically ends when dropped
                }

                // Render editor UI and ImGui to screen
                editor_state.render_ui_and_draw(
                    &mut self.world,
                    render_context,
                    &mut encoder,
                    &view,
                    &window_data.window,
                );

                // Submit commands
                render_context
                    .queue
                    .submit(std::iter::once(encoder.finish()));
                surface_texture.present();
            }
        }

        #[cfg(not(feature = "editor"))]
        {
            if let Some(renderer) = &mut self.renderer {
                // Render frame normally when editor is disabled
                match renderer.render(&self.world) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        info!("Surface lost or outdated, reconfiguring");
                        return; // Will trigger resize on next frame
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        std::process::exit(1);
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "Render error");
                    }
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize on first resume
        if self.window_manager.is_none() {
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

        // Let editor handle events first if enabled
        #[cfg(feature = "editor")]
        {
            if let Some(editor_state) = &mut self.editor_state {
                // Create a proper Event for the editor
                let window_event = winit::event::Event::WindowEvent {
                    event: event.clone(),
                    window_id,
                };

                let should_consume = editor_state.handle_event(&window_data.window, &window_event);

                // Don't return early for critical events like RedrawRequested
                if should_consume && !matches!(event, WindowEvent::RedrawRequested) {
                    return; // Event consumed by editor
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                if window_id == window_manager.main_window_id() {
                    info!("Main window close requested");
                    event_loop.exit();
                } else {
                    // TODO: Handle closing detached panels
                    info!("Secondary window close requested");
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
                self.render_frame(window_id);
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
}

fn main() {
    // Initialize logging
    engine::init_logging();
    info!("Starting WebGPU game");

    // Create event loop
    let event_loop = EventLoop::builder()
        .build()
        .expect("Failed to create event loop");

    // Create application
    let mut app = App::new();

    // Run event loop
    event_loop.run_app(&mut app).expect("Failed to run app");
}

/// Create a demo scene with a rotating cube
fn create_demo_scene(world: &mut World, renderer: &mut Renderer) {
    info!("Creating demo scene");

    // Create camera
    let camera_entity = world.spawn((
        Camera::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0),
        Transform::from_position(Vec3::new(0.0, 2.0, 5.0)).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));
    info!("Created camera entity: {:?}", camera_entity);

    // Create a cube
    let cube_mesh = Mesh::cube(1.0);
    let cube_mesh_id = renderer.upload_mesh(&cube_mesh, "cube");

    let cube_entity = world.spawn((
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
        plane_mesh_id,
        Material::gray(0.3),
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

        world.spawn((
            cube_mesh_id.clone(),
            color,
            Transform::from_position(Vec3::new(x, 0.0, z)).with_scale(Vec3::splat(0.5)),
            GlobalTransform::default(),
        ));
    }
}

/// Update the demo scene (rotate objects)
fn update_demo_scene(world: &mut World, delta_time: f32) {
    // Rotate the center cube
    for (_entity, transform) in world.query_mut::<&mut Transform>() {
        // Only rotate entities at origin (the main cube)
        if transform.position.length() < 0.1 {
            transform.rotation *= Quat::from_rotation_y(delta_time);
        }
    }

    // Orbit smaller cubes
    for (_entity, transform) in world.query_mut::<&mut Transform>() {
        // Only rotate entities away from origin (the orbital cubes)
        if transform.position.length() > 2.0 {
            let angle = delta_time * 0.5;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            let x = transform.position.x;
            let z = transform.position.z;

            transform.position.x = x * cos_a - z * sin_a;
            transform.position.z = x * sin_a + z * cos_a;
        }
    }
}
