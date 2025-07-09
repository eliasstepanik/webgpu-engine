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
    _frame_count: u32,
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

        // Get the actual surface size from render context
        let surface_size = {
            let surface_config = render_context.surface_config.lock().unwrap();
            (surface_config.width, surface_config.height)
        };
        let scale_factor = window.scale_factor() as f32;

        // Configure platform before attaching window
        imgui_platform.attach_window(imgui_context.io_mut(), window, HiDpiMode::Default);

        // CRITICAL: Force correct display size after attaching window
        // This overrides any default size imgui-winit might have set
        let io = imgui_context.io_mut();
        io.display_size = [surface_size.0 as f32, surface_size.1 as f32];
        io.display_framebuffer_scale = [scale_factor, scale_factor];

        debug!(
            "ImGui initial display size forced to: {}x{}, scale: {} (window reports {}x{})",
            surface_size.0,
            surface_size.1,
            scale_factor,
            window.inner_size().width,
            window.inner_size().height
        );

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

        // Create render target for viewport using surface size
        let render_target = RenderTarget::new(
            &render_context.device,
            surface_size.0,
            surface_size.1,
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
                width: surface_size.0,
                height: surface_size.1,
                depth_or_array_layers: 1,
            },
        );
        let texture_id = imgui_renderer.textures.insert(imgui_texture);

        info!("Editor initialized - Press Tab to toggle between Editor UI and Game Input modes");

        // Force initial event processing to ensure imgui is properly initialized
        imgui_platform
            .prepare_frame(imgui_context.io_mut(), window)
            .expect("Initial frame preparation failed");

        Self {
            imgui_context,
            imgui_platform,
            imgui_renderer,
            render_target,
            texture_id,
            selected_entity: None,
            ui_mode: true,
            _frame_count: 0,
        }
    }

    /// Handle winit events
    /// Returns true if the event was consumed by the editor
    pub fn handle_event(&mut self, window: &winit::window::Window, event: &Event<()>) -> bool {
        debug!(
            "Editor handle_event: ui_mode={}, event={:?}",
            self.ui_mode, event
        );
        // Add explicit DPI handling for scale factor changes
        if let Event::WindowEvent {
            event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
            ..
        } = event
        {
            // Note: We don't have access to surface size here, but we need to handle the scale factor
            // The actual size will be corrected in begin_frame when we have access to render_context
            let window_size = window.inner_size();

            // Calculate logical size with proper rounding to avoid fractional pixels
            let logical_width = (window_size.width as f64 / scale_factor).round();
            let logical_height = (window_size.height as f64 / scale_factor).round();

            // Calculate the exact physical size that corresponds to the rounded logical size
            let exact_physical_width = (logical_width * scale_factor) as u32;
            let exact_physical_height = (logical_height * scale_factor) as u32;

            debug!(
                "Scale factor changed to {}: window={}x{}, logical={}x{}, exact_physical={}x{}",
                scale_factor,
                window_size.width,
                window_size.height,
                logical_width,
                logical_height,
                exact_physical_width,
                exact_physical_height
            );

            // Update ImGui's scale factor
            let io = self.imgui_context.io_mut();
            io.display_framebuffer_scale = [*scale_factor as f32, *scale_factor as f32];

            // Note: We're not updating display_size here as we should use surface size
            // which will be set correctly in begin_frame
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
            // Process the event
            self.imgui_platform
                .handle_event(self.imgui_context.io_mut(), window, event);

            // Always consume events in UI mode
            // Don't rely on want_capture flags as they may not be set correctly yet
            return true;
        }

        false
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self, window: &winit::window::Window, render_context: &RenderContext) {
        // --- 1. Query current surface size (physical pixels) -------------------
        let surface_size = {
            let cfg = render_context.surface_config.lock().unwrap();
            (cfg.width, cfg.height)
        };

        // --- 2. Convert to logical units and write once ------------------------
        let dpi = window.scale_factor() as f32;
        let logical_size = [surface_size.0 as f32 / dpi, surface_size.1 as f32 / dpi];
        debug!(
            "Begin frame: surface_size={:?}, display_size={:?}",
            surface_size, logical_size
        );

        // --- 3. Prepare the frame first (this syncs with winit) ----------------
        self.imgui_platform
            .prepare_frame(self.imgui_context.io_mut(), window)
            .expect("imgui prepare_frame failed");

        // --- 4. Force correct display size AFTER prepare_frame ------------------
        // prepare_frame might have set incorrect values, so we override them
        {
            let io = self.imgui_context.io_mut();
            let old_size = io.display_size;
            io.display_size = logical_size;
            io.display_framebuffer_scale = [dpi, dpi];

            if old_size != logical_size {
                debug!(
                    "Corrected ImGui display size from {:?} to {:?} (surface: {:?})",
                    old_size, logical_size, surface_size
                );
            }
        }

        // --- 4. Sanity-check the mapping (debug only) --------------------------
        debug_assert_eq!(
            (self.imgui_context.io().display_size[0]
                * self.imgui_context.io().display_framebuffer_scale[0])
                .round() as u32,
            surface_size.0,
            "width mismatch"
        );
        debug_assert_eq!(
            (self.imgui_context.io().display_size[1]
                * self.imgui_context.io().display_framebuffer_scale[1])
                .round() as u32,
            surface_size.1,
            "height mismatch"
        );
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

    /// Render editor UI, then draw ImGui to the surface ---------------------------------
    pub fn render_ui_and_draw(
        &mut self,
        world: &mut World,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &winit::window::Window,
    ) {
        // Trace for debugging
        tracing::trace!("render_ui_and_draw called");

        // -------------------------------------------------------------------- sizes
        let surface_size = {
            let cfg = render_context.surface_config.lock().unwrap();
            (cfg.width, cfg.height) // physical pixels
        };
        let _dpi = window.scale_factor() as f32;

        // ImGui logical → physical
        let io = self.imgui_context.io();
        let imgui_phys = (
            (io.display_size[0] * io.display_framebuffer_scale[0]).round() as u32,
            (io.display_size[1] * io.display_framebuffer_scale[1]).round() as u32,
        );

        // Abort the frame when sizes disagree
        if imgui_phys != surface_size {
            tracing::error!(
                "Size mismatch: surface={}x{}, imgui(logical)={:?}, imgui(physical)={}x{}",
                surface_size.0,
                surface_size.1,
                io.display_size,
                imgui_phys.0,
                imgui_phys.1
            );
            return;
        }

        // -------------------------------------------------------------------- build UI
        let ui = self.imgui_context.new_frame();

        // Main-menu bar --------------------------------------------------------
        ui.main_menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("New Scene") {
                    info!("New scene requested");
                }
                if ui.menu_item("Load Scene…") {
                    info!("Load scene requested");
                }
                if ui.menu_item("Save Scene…") {
                    info!("Save scene requested");
                }
                ui.separator();
                if ui.menu_item("Exit") {
                    std::process::exit(0);
                }
            });
            ui.menu("View", || {
                if ui.menu_item("Reset Layout") {
                    info!("Reset layout requested");
                }
            });
            ui.menu("Help", || {
                if ui.menu_item("About") {
                    info!("About requested");
                }
            });
        });

        // Panels ---------------------------------------------------------------
        crate::panels::render_hierarchy_panel(ui, world, &mut self.selected_entity);
        crate::panels::render_inspector_panel(ui, world, self.selected_entity);
        crate::panels::render_assets_panel(ui, world);
        crate::panels::render_viewport_panel(ui, self.texture_id, &self.render_target);

        // Status bar -----------------------------------------------------------
        let viewport_h = ui.io().display_size[1];
        ui.window("Status Bar")
            .position([0.0, viewport_h - 25.0], Condition::Always)
            .size([ui.io().display_size[0], 25.0], Condition::Always)
            .no_decoration()
            .movable(false)
            .scroll_bar(false)
            .build(|| {
                ui.text(if self.ui_mode {
                    "Mode: Editor"
                } else {
                    "Mode: Game"
                });
                ui.same_line();
                ui.separator();
                ui.same_line();
                ui.text(format!("Entities: {}", world.query::<()>().iter().count()));
                ui.same_line();
                ui.separator();
                ui.same_line();
                match self.selected_entity {
                    Some(e) => ui.text(format!("Selected: {e:?}")),
                    None => ui.text("No selection"),
                }
            });

        // -------------------------------------------------------------------- render
        self.imgui_platform.prepare_render(ui, window);
        let draw_data = self.imgui_context.render();

        // Final sanity check (physical) ---------------------------------------
        let draw_phys = (
            (draw_data.display_size[0] * draw_data.framebuffer_scale[0]).round() as u32,
            (draw_data.display_size[1] * draw_data.framebuffer_scale[1]).round() as u32,
        );
        if draw_phys != surface_size {
            tracing::error!(
                "Draw-data size mismatch: draw={}x{}, surface={}x{}",
                draw_phys.0,
                draw_phys.1,
                surface_size.0,
                surface_size.1
            );
            return;
        }

        // Render pass ----------------------------------------------------------
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ImGui Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Err(e) = self.imgui_renderer.render(
            draw_data,
            &render_context.queue,
            &render_context.device,
            &mut pass,
        ) {
            tracing::error!("ImGui render failed: {e:?}");
        }
    }

    /// Handle window resize
    pub fn resize(
        &mut self,
        render_context: &RenderContext,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        debug!(
            "Editor resize called with: {}x{}",
            new_size.width, new_size.height
        );

        // Get the actual surface size from render context
        // This is critical - we must use the surface config size, not the window size
        let (actual_width, actual_height) = {
            let surface_config = render_context.surface_config.lock().unwrap();
            (surface_config.width, surface_config.height)
        };

        debug!(
            "Surface config size: {}x{} (requested: {}x{})",
            actual_width, actual_height, new_size.width, new_size.height
        );

        // Update ImGui display size to match surface configuration
        let io = self.imgui_context.io_mut();
        io.display_size = [actual_width as f32, actual_height as f32];

        // Get surface format from surface config
        let surface_format = render_context.surface_config.lock().unwrap().format;

        // Store the old texture existence status before recreating renderer
        let old_texture_exists = self.imgui_renderer.textures.get(self.texture_id).is_some();
        debug!(
            "Editor resize: old_texture_exists={}, new_size={:?}",
            old_texture_exists, new_size
        );

        // Only remove if it exists in current renderer
        if old_texture_exists {
            self.imgui_renderer.textures.remove(self.texture_id);
        }

        // CRITICAL: Recreate the imgui renderer to ensure it uses the new viewport size
        // This prevents scissor rect validation errors from cached viewport dimensions
        let renderer_config = RendererConfig {
            texture_format: surface_format,
            ..Default::default()
        };

        self.imgui_renderer = Renderer::new(
            &mut self.imgui_context,
            &render_context.device,
            &render_context.queue,
            renderer_config,
        );

        // Recreate render target with actual surface size
        self.render_target = RenderTarget::new(
            &render_context.device,
            actual_width,
            actual_height,
            surface_format,
        );

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
                width: actual_width,
                height: actual_height,
                depth_or_array_layers: 1,
            },
        );
        self.texture_id = self.imgui_renderer.textures.insert(imgui_texture);
    }
}
