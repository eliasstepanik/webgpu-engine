//! Main editor state management
//!
//! This module contains the EditorState struct which manages the imgui context,
//! render target for viewport, and all editor UI state.

use engine::core::entity::World;
use engine::graphics::{context::RenderContext, render_target::RenderTarget};
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use tracing::{debug, info};
use winit::event::{Event, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Main editor state that manages all editor functionality
pub struct EditorState {
    /// ImGui context for UI rendering
    pub imgui_context: imgui::Context,
    /// ImGui-winit platform integration
    pub imgui_platform: WinitPlatform,
    /// ImGui-wgpu renderer
    pub imgui_renderer: Renderer,
    /// Render target for viewport texture
    pub render_target: RenderTarget,
    /// ImGui texture ID for the render target
    pub texture_id: imgui::TextureId,
    /// Currently selected entity in the hierarchy
    pub selected_entity: Option<hecs::Entity>,
    /// Current input mode (true = editor UI, false = game input)
    pub ui_mode: bool,
    /// Frame counter to skip initial frames during window setup
    frame_count: u32,
}

impl EditorState {
    /// Create a new editor state
    pub fn new(render_context: &RenderContext, window: &winit::window::Window) -> Self {
        info!("Initializing editor state with ImGui");

        // Create ImGui context
        let mut imgui_context = imgui::Context::create();

        // Configure ImGui
        imgui_context.set_ini_filename(None); // Don't save settings to file

        // Note: Docking is not available in imgui-rs 0.12 by default
        // We'll use a simpler window-based layout instead

        // Set up some styling
        let style = imgui_context.style_mut();
        style.window_rounding = 0.0;
        style.scrollbar_rounding = 0.0;

        // Create platform integration
        let mut imgui_platform = WinitPlatform::new(&mut imgui_context);

        // Get the actual window size before attaching
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;

        // Set display size BEFORE attaching window to prevent default 1920x1080
        let io = imgui_context.io_mut();
        io.display_size = [size.width as f32, size.height as f32];
        io.display_framebuffer_scale = [scale_factor, scale_factor];

        debug!(
            "ImGui initial display size: {}x{}, scale: {}",
            size.width, size.height, scale_factor
        );

        // Now attach the window
        imgui_platform.attach_window(imgui_context.io_mut(), window, HiDpiMode::Default);

        // Get surface format from surface config
        let surface_format = render_context.surface_config.lock().unwrap().format;

        // Create renderer
        let renderer_config = RendererConfig {
            texture_format: surface_format,
            ..Default::default()
        };

        let mut imgui_renderer = Renderer::new(
            &mut imgui_context,
            &render_context.device,
            &render_context.queue,
            renderer_config,
        );

        // Create render target for viewport
        let size = window.inner_size();
        let render_target = RenderTarget::new(
            &render_context.device,
            size.width,
            size.height,
            surface_format,
        );

        // Register render target texture with ImGui
        // Create texture configuration for the render target
        let texture_config = imgui_wgpu::RawTextureConfig {
            label: Some("Editor Viewport Texture"),
            sampler_desc: wgpu::SamplerDescriptor {
                label: Some("Viewport Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        };
        
        let imgui_texture = imgui_wgpu::Texture::from_raw_parts(
            &render_context.device,
            &imgui_renderer,
            std::sync::Arc::new(render_target.texture.clone()),
            std::sync::Arc::new(render_target.view.clone()),
            None,
            Some(&texture_config),
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
        let texture_id = imgui_renderer.textures.insert(imgui_texture);

        info!("Editor initialized - Press Tab to toggle between Editor UI and Game Input modes");

        Self {
            imgui_context,
            imgui_platform,
            imgui_renderer,
            render_target,
            texture_id,
            selected_entity: None,
            ui_mode: true,
            frame_count: 0,
        }
    }

    /// Handle winit events
    /// Returns true if the event was consumed by the editor
    pub fn handle_event(&mut self, window: &winit::window::Window, event: &Event<()>) -> bool {
        // Add explicit DPI handling for scale factor changes
        if let Event::WindowEvent {
            event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
            ..
        } = event
        {
            let window_size = window.inner_size();
            let physical_size = (window_size.width, window_size.height);
            let logical_size = (
                (window_size.width as f64 / scale_factor) as f32,
                (window_size.height as f64 / scale_factor) as f32,
            );

            debug!(
                physical_size = ?physical_size,
                logical_size = ?logical_size,
                scale_factor = scale_factor,
                "Window scale factor changed"
            );

            // Update ImGui's scale factor
            self.imgui_context.io_mut().display_framebuffer_scale =
                [*scale_factor as f32, *scale_factor as f32];
        }

        // Check for Tab key to toggle input mode
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput {
                event: key_event, ..
            },
            ..
        } = event
        {
            if key_event.state == winit::event::ElementState::Pressed {
                if let PhysicalKey::Code(KeyCode::Tab) = key_event.physical_key {
                    self.ui_mode = !self.ui_mode;
                    debug!("Toggled input mode: UI mode = {}", self.ui_mode);
                    return true;
                }
            }
        }

        // If in UI mode, let imgui handle ALL events
        if self.ui_mode {
            self.imgui_platform
                .handle_event(self.imgui_context.io_mut(), window, event);
            return self.imgui_context.io().want_capture_mouse
                || self.imgui_context.io().want_capture_keyboard;
        }

        false
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        // Get the actual window size
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;

        // Update ImGui's display size BEFORE prepare_frame
        let io = self.imgui_context.io_mut();
        let old_size = io.display_size;
        io.display_size = [size.width as f32, size.height as f32];
        io.display_framebuffer_scale = [scale_factor, scale_factor];

        // Log if size changed
        if old_size[0] != io.display_size[0] || old_size[1] != io.display_size[1] {
            debug!(
                "ImGui display size updated: {:?} -> {:?}",
                old_size, io.display_size
            );
        }

        self.imgui_platform
            .prepare_frame(self.imgui_context.io_mut(), window)
            .expect("Failed to prepare ImGui frame");
    }

    /// Render the game to the viewport texture
    pub fn render_viewport(
        &mut self,
        renderer: &mut engine::graphics::renderer::Renderer,
        world: &World,
    ) {
        // Render the game to our render target texture
        if let Err(e) = renderer.render_to_target(world, &self.render_target) {
            tracing::error!("Failed to render to viewport: {e:?}");
        }
    }

    /// Render the editor UI and imgui to screen  
    pub fn render_ui_and_draw(
        &mut self,
        world: &mut World,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &winit::window::Window,
    ) {
        // Increment frame counter
        self.frame_count += 1;

        // Skip the first few frames to let the window settle on Windows
        // This helps avoid initialization timing issues with DPI scaling
        const SKIP_FRAMES: u32 = 5;
        if self.frame_count < SKIP_FRAMES {
            debug!(
                frame = self.frame_count,
                skip_total = SKIP_FRAMES,
                "Skipping ImGui render during initialization"
            );
            return;
        }

        // Get the current render target size from the surface config
        let render_target_size = {
            let surface_config = render_context.surface_config.lock().unwrap();
            (surface_config.width, surface_config.height)
        };

        // Double-check that ImGui's display size matches the render target
        let window_size = window.inner_size();
        let io = self.imgui_context.io_mut();

        // Check if window size matches render target size
        if window_size.width != render_target_size.0 || window_size.height != render_target_size.1 {
            tracing::warn!(
                "Window size doesn't match render target: window={}x{}, target={}x{}",
                window_size.width,
                window_size.height,
                render_target_size.0,
                render_target_size.1
            );
            // Skip rendering this frame to avoid scissor rect errors
            return;
        }

        if io.display_size[0] != window_size.width as f32
            || io.display_size[1] != window_size.height as f32
        {
            tracing::warn!(
                "ImGui display size mismatch: imgui=[{}, {}], window=[{}, {}]",
                io.display_size[0],
                io.display_size[1],
                window_size.width,
                window_size.height
            );
            // Update the size
            io.display_size = [window_size.width as f32, window_size.height as f32];
        }

        let ui = self.imgui_context.new_frame();

        // Force ImGui to use the correct display size for this frame
        let io = ui.io();
        let display_size = [render_target_size.0 as f32, render_target_size.1 as f32];
        if io.display_size[0] != display_size[0] || io.display_size[1] != display_size[1] {
            tracing::warn!(
                "Forcing ImGui display size from {:?} to {:?}",
                io.display_size,
                display_size
            );
        }

        // Since docking is not available, we'll use a simpler layout
        // with fixed windows

        // Top menu bar
        ui.main_menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("New Scene") {
                    info!("New scene requested");
                    // TODO: Implement new scene
                }
                if ui.menu_item("Load Scene...") {
                    info!("Load scene requested");
                    // TODO: Implement load scene dialog
                }
                if ui.menu_item("Save Scene...") {
                    info!("Save scene requested");
                    // TODO: Implement save scene dialog
                }
                ui.separator();
                if ui.menu_item("Exit") {
                    std::process::exit(0);
                }
            });

            ui.menu("View", || {
                if ui.menu_item("Reset Layout") {
                    info!("Reset layout requested");
                    // TODO: Reset layout
                }
            });

            ui.menu("Help", || {
                if ui.menu_item("About") {
                    info!("About requested");
                }
            });
        });

        // Render panels
        crate::panels::render_hierarchy_panel(ui, world, &mut self.selected_entity);
        crate::panels::render_inspector_panel(ui, world, self.selected_entity);
        crate::panels::render_assets_panel(ui, world);

        // Render viewport with the game texture
        crate::panels::render_viewport_panel(ui, self.texture_id, &self.render_target);

        // Status bar
        let viewport_height = ui.io().display_size[1];
        ui.window("Status Bar")
            .position([0.0, viewport_height - 25.0], Condition::Always)
            .size([ui.io().display_size[0], 25.0], Condition::Always)
            .no_decoration()
            .movable(false)
            .scroll_bar(false)
            .build(|| {
                let mode_text = if self.ui_mode {
                    "Mode: Editor"
                } else {
                    "Mode: Game"
                };
                ui.text(mode_text);
                ui.same_line();
                ui.separator();
                ui.same_line();

                let entity_count = world.query::<()>().iter().count();
                ui.text(format!("Entities: {entity_count}"));
                ui.same_line();
                ui.separator();
                ui.same_line();

                if let Some(entity) = self.selected_entity {
                    ui.text(format!("Selected: {entity:?}"));
                } else {
                    ui.text("No selection");
                }
            });

        // Prepare render
        self.imgui_platform.prepare_render(ui, window);
        let draw_data = self.imgui_context.render();

        // Validate draw data before rendering
        if draw_data.display_size[0] <= 0.0 || draw_data.display_size[1] <= 0.0 {
            tracing::warn!(
                "Invalid draw data display size: {:?}",
                draw_data.display_size
            );
            return;
        }

        // Skip scissor rect validation for now - imgui-wgpu should handle this internally
        // The issue seems to be with ImGui using default 1920x1080 in early frames

        // Create a render pass for ImGui
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ImGui Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Load existing content
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Render ImGui - imgui-wgpu should handle scissor rects based on draw_data
        if let Err(e) = self.imgui_renderer.render(
            draw_data,
            &render_context.queue,
            &render_context.device,
            &mut render_pass,
        ) {
            tracing::error!("Failed to render ImGui: {:?}", e);
            tracing::error!("Render target size: {:?}", render_target_size);
            tracing::error!(
                "ImGui display size: {:?}",
                self.imgui_context.io().display_size
            );
        }

        drop(render_pass);
    }

    /// Handle window resize
    pub fn resize(
        &mut self,
        render_context: &RenderContext,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        debug!("Editor resize: {}x{}", new_size.width, new_size.height);

        // Update ImGui display size
        let io = self.imgui_context.io_mut();
        io.display_size = [new_size.width as f32, new_size.height as f32];

        // Get surface format from surface config
        let surface_format = render_context.surface_config.lock().unwrap().format;

        // Recreate render target with new size
        self.render_target = RenderTarget::new(
            &render_context.device,
            new_size.width,
            new_size.height,
            surface_format,
        );

        // Remove old texture and register new one
        self.imgui_renderer.textures.remove(self.texture_id);
        
        // Create texture configuration for the render target
        let texture_config = imgui_wgpu::RawTextureConfig {
            label: Some("Editor Viewport Texture"),
            sampler_desc: wgpu::SamplerDescriptor {
                label: Some("Viewport Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            },
        };
        
        let imgui_texture = imgui_wgpu::Texture::from_raw_parts(
            &render_context.device,
            &self.imgui_renderer,
            std::sync::Arc::new(self.render_target.texture.clone()),
            std::sync::Arc::new(self.render_target.view.clone()),
            None,
            Some(&texture_config),
            wgpu::Extent3d {
                width: new_size.width,
                height: new_size.height,
                depth_or_array_layers: 1,
            },
        );
        self.texture_id = self.imgui_renderer.textures.insert(imgui_texture);
    }
}
