//! Game entry point with WebGPU rendering demonstration

use engine::prelude::*;
use engine::windowing::WindowManager;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
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
    render_context: Option<Arc<RenderContext>>,
    /// Renderer for the game
    renderer: Option<Renderer>,
    /// ECS world
    world: World,
    /// Time tracking
    last_time: std::time::Instant,
    /// Editor state
    #[cfg(feature = "editor")]
    editor_state: Option<EditorState>,
    /// WebGPU instance
    instance: Option<Arc<wgpu::Instance>>,
    /// Focus tracking for window management
    focus_tracker: HashMap<WindowId, bool>,
    /// Last focused window ID
    last_focused_window: Option<WindowId>,
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
            focus_tracker: HashMap::new(),
            last_focused_window: None,
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

        // Initialize render context with the same instance to ensure adapter IDs match
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

        // Create demo scene
        create_demo_scene(&mut self.world, &mut renderer);

        // Create editor state if feature is enabled
        #[cfg(feature = "editor")]
        let editor_state = {
            let window_size = window.inner_size();

            // Move the world to the editor's shared state
            let world = std::mem::replace(&mut self.world, World::new());

            // Note: DetachedWindowManager will be initialized lazily when first needed
            // to avoid surface creation conflicts during startup

            EditorState::new(
                &render_context,
                &window,
                surface_config.format,
                (window_size.width, window_size.height),
                world,
            )
        };

        // Store initialized components
        self.instance = Some(instance);
        self.window_manager = Some(window_manager);
        self.render_context = Some(render_context.clone());
        self.renderer = Some(renderer);
        #[cfg(feature = "editor")]
        {
            let mut editor_state = editor_state;

            // Initialize viewport backend if viewport feature is enabled
            #[cfg(feature = "viewport")]
            {
                editor_state.init_viewport_backend(&window, &render_context);
            }

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
                let aspect_ratio = new_size.width as f32 / new_size.height as f32;
                #[cfg(feature = "editor")]
                {
                    if let Some(editor_state) = &self.editor_state {
                        editor_state.shared_state.with_world_write(|world| {
                            for (_, camera) in world.query_mut::<&mut Camera>() {
                                camera.set_aspect_ratio(aspect_ratio);
                            }
                        });
                    }
                }

                #[cfg(not(feature = "editor"))]
                {
                    for (_, camera) in self.world.query_mut::<&mut Camera>() {
                        camera.set_aspect_ratio(aspect_ratio);
                    }
                }
            }
        }
    }

    fn render_frame(&mut self, window_id: WindowId, event_loop: &ActiveEventLoop) {
        // Handle detach/attach requests first (only if there are pending requests)
        #[cfg(feature = "editor")]
        {
            if let (Some(editor_state), Some(window_manager)) =
                (&mut self.editor_state, &mut self.window_manager)
            {
                // Handle panel detachment based on viewport feature
                #[cfg(feature = "viewport")]
                {
                    // Use viewport system for detachment
                    editor_state.process_viewport_requests(window_manager, event_loop);
                }

                #[cfg(not(feature = "viewport"))]
                {
                    // Fall back to old detached window manager when viewport is disabled
                    if editor_state.panel_manager.has_pending_requests() {
                        if editor_state.detached_window_manager.is_none() {
                            if let Some(render_context) = &self.render_context {
                                editor_state.init_detached_window_manager(render_context.clone());
                            }
                        }

                        if let Some(detached_window_manager) =
                            &mut editor_state.detached_window_manager
                        {
                            detached_window_manager.process_detach_requests(
                                &mut editor_state.panel_manager,
                                window_manager,
                                event_loop,
                            );
                            detached_window_manager.process_attach_requests(
                                &mut editor_state.panel_manager,
                                window_manager,
                            );
                        }
                    }
                }
            }
        }

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

        // Update demo scene and hierarchy
        #[cfg(feature = "editor")]
        {
            if let Some(editor_state) = &self.editor_state {
                // Update through shared state when editor is enabled
                editor_state.shared_state.with_world_write(|world| {
                    update_demo_scene(world, delta_time);
                    update_hierarchy_system(world);
                });
            }
        }

        #[cfg(not(feature = "editor"))]
        {
            // Update directly when editor is disabled
            update_demo_scene(&mut self.world, delta_time);
            update_hierarchy_system(&mut self.world);
        }

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
                            editor_state.shared_state.with_world_write(|world| {
                                editor::scene_operations::create_default_scene(world, renderer);
                            });
                        }
                        SceneOperation::LoadScene(path) => {
                            let result = editor_state.shared_state.with_world_write(|world| {
                                editor::scene_operations::load_scene_from_file(
                                    world, renderer, &path,
                                )
                            });
                            match result.unwrap_or(Err("Failed to access world".into())) {
                                Ok(_) => info!("Scene loaded successfully"),
                                Err(e) => {
                                    tracing::error!("Failed to load scene: {:?}", e);
                                    editor_state.error_message =
                                        Some(format!("Failed to load scene: {e}"));
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
                let shared_state = editor_state.shared_state.clone();
                shared_state.with_world_read(|world| {
                    editor_state.render_viewport(renderer, world);
                });

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

        #[cfg(not(feature = "editor"))]
        {
            if let Some(renderer) = &mut self.renderer {
                // Render frame normally when editor is disabled
                match renderer.render(&self.world, &window_data.surface) {
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

                // Handle viewport-specific events when viewport feature is enabled
                #[cfg(feature = "viewport")]
                {
                    if let Some(window_manager) = &self.window_manager {
                        editor_state.handle_viewport_event(
                            &window_event,
                            window_id,
                            window_manager,
                        );
                    }
                }

                let should_consume = editor_state.handle_event(&window_data.window, &window_event);

                // Don't return early for critical events like RedrawRequested
                if should_consume && !matches!(event, WindowEvent::RedrawRequested) {
                    return; // Event consumed by editor
                }
            }
        }

        match event {
            WindowEvent::Focused(focused) => {
                self.focus_tracker.insert(window_id, focused);

                // Platform-specific focus handling
                #[cfg(target_os = "windows")]
                {
                    // Windows has focus event ordering issues
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

                // Pass focus event to editor for viewport handling
                #[cfg(feature = "editor")]
                {
                    if let Some(editor_state) = &mut self.editor_state {
                        // Create event for editor
                        let window_event = winit::event::Event::WindowEvent {
                            event: WindowEvent::Focused(focused),
                            window_id,
                        };
                        editor_state.handle_event(&window_data.window, &window_event);
                    }
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

                    // Clean up editor resources before exit
                    #[cfg(feature = "editor")]
                    {
                        if let (Some(editor_state), Some(window_manager)) =
                            (&mut self.editor_state, &mut self.window_manager)
                        {
                            editor_state.shutdown(window_manager);
                        }
                    }

                    event_loop.exit();
                } else {
                    // Handle closing detached panels
                    info!("Secondary window close requested: {:?}", window_id);

                    #[cfg(feature = "editor")]
                    {
                        if let (Some(editor_state), Some(window_manager)) =
                            (&mut self.editor_state, &mut self.window_manager)
                        {
                            if let Some(detached_window_manager) =
                                &mut editor_state.detached_window_manager
                            {
                                detached_window_manager.handle_window_close(
                                    window_id,
                                    &mut editor_state.panel_manager,
                                    window_manager,
                                );
                            }
                        }
                    }
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
                self.render_frame(window_id, event_loop);
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
