//! Game entry point with WebGPU rendering demonstration

use engine::prelude::*;
use std::sync::Arc;
use tracing::info;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowAttributes,
};

#[cfg(feature = "editor")]
use editor::EditorState;

fn main() {
    // Initialize logging
    engine::init_logging();
    info!("Starting WebGPU game");

    // Create event loop and window
    let event_loop = EventLoop::builder()
        .build()
        .expect("Failed to create event loop");
    let window_attributes = WindowAttributes::default()
        .with_title("WebGPU Game Engine Demo")
        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));

    #[allow(deprecated)] // Using create_window on EventLoop for simplicity
    let window = Arc::new(
        event_loop
            .create_window(window_attributes)
            .expect("Failed to create window"),
    );

    // Initialize renderer
    let render_context =
        pollster::block_on(RenderContext::new(&window)).expect("Failed to create render context");
    let render_context = Arc::new(render_context);
    let mut renderer = Renderer::new(render_context.clone());

    // Create ECS world
    let mut world = World::new();

    // Create demo scene
    create_demo_scene(&mut world, &mut renderer);

    // Game state
    let mut last_time = std::time::Instant::now();

    // Create editor state if feature is enabled
    #[cfg(feature = "editor")]
    let mut editor_state = EditorState::new(&render_context, &window);

    // Clone window Arc for the event loop closure
    let window = window.clone();

    // Run event loop
    #[allow(deprecated)] // Using the simpler closure-based API for now
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => {
                // Let editor handle events first if enabled
                #[cfg(feature = "editor")]
                {
                    if editor_state.handle_event(
                        &window,
                        &Event::WindowEvent {
                            event: event.clone(),
                            window_id: window.id(),
                        },
                    ) {
                        return; // Event consumed by editor
                    }
                }

                match event {
                    WindowEvent::CloseRequested => {
                        info!("Window close requested");
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        info!("Window resized to {:?}", physical_size);
                        // Renderer.resize() now handles both RenderContext and depth texture resize
                        renderer.resize(physical_size);

                        // Resize editor viewport if enabled
                        #[cfg(feature = "editor")]
                        editor_state.resize(&render_context, physical_size);

                        // Update camera aspect ratio
                        for (_, camera) in world.query_mut::<&mut Camera>() {
                            camera.set_aspect_ratio(
                                physical_size.width as f32 / physical_size.height as f32,
                            );
                        }
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        info!("Scale factor changed to {}", scale_factor);
                        // In winit 0.30, we need to handle scale factor changes differently
                        // The window size is automatically updated, so we just need to get the new size
                        let new_size = window.inner_size();
                        // Renderer.resize() now handles both RenderContext and depth texture resize
                        renderer.resize(new_size);

                        // Update camera aspect ratio
                        for (_, camera) in world.query_mut::<&mut Camera>() {
                            camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        // Skip rendering if window is minimized
                        let size = window.inner_size();
                        if size.width == 0 || size.height == 0 {
                            return;
                        }

                        // Update time
                        let current_time = std::time::Instant::now();
                        let delta_time = (current_time - last_time).as_secs_f32();
                        last_time = current_time;

                        // Update demo scene
                        update_demo_scene(&mut world, delta_time);

                        // Update transform hierarchy
                        update_hierarchy_system(&mut world);

                        // Render based on editor mode
                        // Render based on editor mode
                        #[cfg(feature = "editor")]
                        {
                            // Begin editor frame
                            editor_state.begin_frame(&window);

                            // Render game to viewport texture
                            editor_state.render_viewport(&mut renderer, &world);

                            // Get surface texture for final rendering
                            let surface_texture = match render_context
                                .surface
                                .lock()
                                .unwrap()
                                .get_current_texture()
                            {
                                Ok(texture) => texture,
                                Err(e) => {
                                    tracing::error!("Failed to get surface texture: {:?}", e);
                                    return;
                                }
                            };

                            let view = surface_texture
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());

                            // Create command encoder for ImGui rendering
                            let mut encoder = render_context.device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor {
                                    label: Some("ImGui Render Encoder"),
                                },
                            );

                            // Clear the surface first
                            {
                                let _render_pass =
                                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: Some("Clear Pass"),
                                        color_attachments: &[Some(
                                            wgpu::RenderPassColorAttachment {
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
                                            },
                                        )],
                                        depth_stencil_attachment: None,
                                        timestamp_writes: None,
                                        occlusion_query_set: None,
                                    });
                                // Render pass automatically ends when dropped
                            }

                            // Render editor UI and ImGui to screen
                            editor_state.render_ui_and_draw(
                                &mut world,
                                &render_context,
                                &mut encoder,
                                &view,
                                &window,
                            );

                            // Submit commands
                            render_context
                                .queue
                                .submit(std::iter::once(encoder.finish()));
                            surface_texture.present();
                        }
                        #[cfg(not(feature = "editor"))]
                        {
                            // Render frame normally when editor is disabled
                            match renderer.render(&world) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => {
                                    info!("Surface lost, reconfiguring");
                                    let size = window.inner_size();
                                    // Renderer.resize() now handles both RenderContext and depth texture resize
                                    renderer.resize(size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    elwt.exit();
                                }
                                Err(e) => {
                                    tracing::error!(error = ?e, "Render error");
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // Request redraw
                window.request_redraw();
            }
            _ => {}
        }
    });
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
