//! Main editor state management
//!
//! This module contains the EditorState struct which manages the imgui context,
//! render target for viewport, and all editor UI state.

use engine::core::entity::World;
use engine::graphics::{context::RenderContext, render_target::RenderTarget};
use imgui::*;
use imgui_wgpu::{Renderer as ImGuiRenderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::path::PathBuf;
use tracing::{debug, info};
use winit::event::{Event, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Pending scene action to perform after confirmation
#[derive(Debug, Clone)]
pub enum PendingAction {
    NewScene,
    LoadScene,
    Exit,
}

/// Scene operation that needs to be performed by main loop
#[derive(Debug, Clone)]
pub enum SceneOperation {
    NewScene,
    LoadScene(PathBuf),
    SaveScene(PathBuf),
}

/// Main editor state that manages all editor functionality
pub struct EditorState {
    /// ImGui context for UI rendering
    pub imgui_context: imgui::Context,
    /// ImGui-winit platform integration
    pub imgui_platform: WinitPlatform,
    /// ImGui-wgpu renderer
    pub imgui_renderer: ImGuiRenderer,
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
    /// Pending resize to apply when safe
    pending_resize: Option<winit::dpi::PhysicalSize<u32>>,
    /// Whether we're currently in a frame
    in_frame: bool,

    // Scene management state
    /// Path to the currently loaded scene
    pub current_scene_path: Option<PathBuf>,
    /// Whether the scene has been modified since last save
    pub scene_modified: bool,
    /// Current keyboard modifiers state
    pub current_modifiers: winit::event::Modifiers,

    // Dialog state
    /// Whether to show the unsaved changes dialog
    pub show_unsaved_dialog: bool,
    /// Pending action after unsaved changes dialog
    pub pending_action: Option<PendingAction>,
    /// Error message to display in modal
    pub error_message: Option<String>,
    /// Pending scene operation to be performed by main loop
    pub pending_scene_operation: Option<SceneOperation>,
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

        let mut imgui_renderer = ImGuiRenderer::new(
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

        let mut editor = Self {
            imgui_context,
            imgui_platform,
            imgui_renderer,
            render_target,
            texture_id,
            selected_entity: None,
            ui_mode: true,
            _frame_count: 0,
            pending_resize: None,
            in_frame: false,

            // Scene management state
            current_scene_path: None,
            scene_modified: false,
            current_modifiers: winit::event::Modifiers::default(),

            // Dialog state
            show_unsaved_dialog: false,
            pending_action: None,
            error_message: None,
            pending_scene_operation: None,
        };

        // Force proper initialization by setting initial values
        // This ensures all imgui state is properly set up
        {
            let io = editor.imgui_context.io_mut();
            io.display_size = [surface_size.0 as f32, surface_size.1 as f32];
            io.display_framebuffer_scale = [scale_factor, scale_factor];

            // Don't create a dummy frame here as it could conflict with font atlas
        }

        editor
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

        // Track modifier changes
        if let Event::WindowEvent {
            event: WindowEvent::ModifiersChanged(new_modifiers),
            ..
        } = event
        {
            self.current_modifiers = *new_modifiers;
            debug!("Modifiers changed: {:?}", self.current_modifiers);
        }

        // Check for keyboard shortcuts
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput {
                event: key_event, ..
            },
            ..
        } = event
        {
            if key_event.state == winit::event::ElementState::Pressed {
                // Check for scene management shortcuts when in UI mode
                if self.ui_mode {
                    let ctrl = self.current_modifiers.lcontrol_state()
                        == winit::keyboard::ModifiersKeyState::Pressed
                        || self.current_modifiers.rcontrol_state()
                            == winit::keyboard::ModifiersKeyState::Pressed;
                    let shift = self.current_modifiers.lshift_state()
                        == winit::keyboard::ModifiersKeyState::Pressed
                        || self.current_modifiers.rshift_state()
                            == winit::keyboard::ModifiersKeyState::Pressed;

                    if ctrl {
                        match key_event.physical_key {
                            PhysicalKey::Code(KeyCode::KeyN) => {
                                info!("Ctrl+N pressed - New Scene");
                                self.new_scene_action();
                                return true;
                            }
                            PhysicalKey::Code(KeyCode::KeyO) => {
                                info!("Ctrl+O pressed - Open Scene");
                                self.load_scene_action();
                                return true;
                            }
                            PhysicalKey::Code(KeyCode::KeyS) => {
                                if shift {
                                    info!("Ctrl+Shift+S pressed - Save Scene As");
                                    self.save_scene_as_action();
                                } else {
                                    info!("Ctrl+S pressed - Save Scene");
                                    self.save_scene_action();
                                }
                                return true;
                            }
                            _ => {}
                        }
                    }
                }

                // Check for Tab key to toggle input mode
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

            // Check if imgui wants to capture this event
            let io = self.imgui_context.io();
            let wants_keyboard = io.want_capture_keyboard;
            let wants_mouse = io.want_capture_mouse;

            // For keyboard events, only consume if imgui wants keyboard
            if matches!(
                event,
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { .. },
                    ..
                }
            ) {
                return wants_keyboard;
            }

            // For mouse events, only consume if imgui wants mouse
            if matches!(
                event,
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { .. }
                        | WindowEvent::MouseInput { .. }
                        | WindowEvent::MouseWheel { .. },
                    ..
                }
            ) {
                return wants_mouse;
            }

            // For other events in UI mode, don't consume
            return false;
        }

        false
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self, window: &winit::window::Window, render_context: &RenderContext) {
        // Mark that we're in a frame
        self.in_frame = true;

        // Handle any pending resize before starting the frame
        if let Some(new_size) = self.pending_resize.take() {
            self.do_resize(render_context, new_size);
        }

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
        debug!(
            "Rendering game to viewport texture, render_target size: {:?}",
            self.render_target.size
        );
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

        // We'll mark frame as done at the end of this method

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

        // Store actions to take after UI rendering
        let mut action_new_scene = false;
        let mut action_load_scene = false;
        let mut action_save_scene = false;
        let mut action_save_scene_as = false;
        let mut action_exit = false;

        // Main-menu bar --------------------------------------------------------
        ui.main_menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("New Scene##Ctrl+N") {
                    action_new_scene = true;
                }
                if ui.menu_item("Load Scene...##Ctrl+O") {
                    action_load_scene = true;
                }
                if ui.menu_item("Save Scene##Ctrl+S") {
                    action_save_scene = true;
                }
                if ui.menu_item("Save Scene As...##Ctrl+Shift+S") {
                    action_save_scene_as = true;
                }
                ui.separator();
                if ui.menu_item("Exit") {
                    action_exit = true;
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
                // Scene name
                let scene_name = self
                    .current_scene_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("Untitled");
                ui.text(format!(
                    "Scene: {}{}",
                    scene_name,
                    if self.scene_modified { "*" } else { "" }
                ));
                ui.same_line();
                ui.separator();
                ui.same_line();

                // Mode
                ui.text(if self.ui_mode {
                    "Mode: Editor"
                } else {
                    "Mode: Game"
                });
                ui.same_line();
                ui.separator();
                ui.same_line();

                // Entity count
                ui.text(format!("Entities: {}", world.query::<()>().iter().count()));
                ui.same_line();
                ui.separator();
                ui.same_line();

                // Selection
                match self.selected_entity {
                    Some(e) => ui.text(format!("Selected: {e:?}")),
                    None => ui.text("No selection"),
                }
            });

        // Dialogs --------------------------------------------------------------

        // Store dialog actions
        let mut dialog_save_and_proceed = false;
        let mut dialog_dont_save_and_proceed = false;
        let mut dialog_cancel = false;
        let mut clear_error = false;

        // Unsaved changes dialog
        if self.show_unsaved_dialog {
            ui.open_popup("unsaved_changes");
        }

        ui.modal_popup("unsaved_changes", || {
            ui.text("Save changes to current scene?");
            ui.spacing();

            if ui.button("Save") {
                dialog_save_and_proceed = true;
                ui.close_current_popup();
            }

            ui.same_line();
            if ui.button("Don't Save") {
                dialog_dont_save_and_proceed = true;
                ui.close_current_popup();
            }

            ui.same_line();
            if ui.button("Cancel") {
                dialog_cancel = true;
                ui.close_current_popup();
            }
        });

        // Error dialog
        if self.error_message.is_some() {
            ui.open_popup("error_dialog");
        }

        ui.modal_popup("error_dialog", || {
            ui.text("Error");
            ui.separator();
            if let Some(ref error) = self.error_message {
                ui.text_wrapped(error);
            }
            if ui.button("OK") {
                clear_error = true;
                ui.close_current_popup();
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
        {
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
        } // Pass is dropped here

        // Mark that we're no longer in a frame
        self.in_frame = false;

        // Handle deferred actions after UI rendering
        if action_new_scene {
            self.new_scene_action();
        }
        if action_load_scene {
            self.load_scene_action();
        }
        if action_save_scene {
            self.save_scene_action();
        }
        if action_save_scene_as {
            self.save_scene_as_action();
        }
        if action_exit {
            if self.scene_modified {
                self.show_unsaved_dialog = true;
                self.pending_action = Some(PendingAction::Exit);
            } else {
                std::process::exit(0);
            }
        }

        // Handle dialog actions
        if dialog_save_and_proceed && self.save_scene_action() {
            self.show_unsaved_dialog = false;
            if let Some(action) = self.pending_action.take() {
                match action {
                    PendingAction::NewScene => self.perform_new_scene(),
                    PendingAction::LoadScene => self.perform_load_scene(),
                    PendingAction::Exit => std::process::exit(0),
                }
            }
        }
        if dialog_dont_save_and_proceed {
            self.show_unsaved_dialog = false;
            if let Some(action) = self.pending_action.take() {
                match action {
                    PendingAction::NewScene => self.perform_new_scene(),
                    PendingAction::LoadScene => self.perform_load_scene(),
                    PendingAction::Exit => std::process::exit(0),
                }
            }
        }
        if dialog_cancel {
            self.show_unsaved_dialog = false;
            self.pending_action = None;
        }
        if clear_error {
            self.error_message = None;
        }
    }

    /// Handle window resize
    pub fn resize(
        &mut self,
        render_context: &RenderContext,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        debug!(
            "Editor resize called with: {}x{}, in_frame: {}",
            new_size.width, new_size.height, self.in_frame
        );

        // If we're in a frame, defer the resize
        if self.in_frame {
            self.pending_resize = Some(new_size);
            debug!("Deferring resize until next frame");
            return;
        }

        self.do_resize(render_context, new_size);
    }

    /// Actually perform the resize (when safe to do so)
    fn do_resize(
        &mut self,
        render_context: &RenderContext,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        debug!("Performing actual resize");

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

        self.imgui_renderer = ImGuiRenderer::new(
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

    // Scene management methods

    /// Handle new scene action
    pub fn new_scene_action(&mut self) {
        if self.scene_modified {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::NewScene);
        } else {
            self.perform_new_scene();
        }
    }

    /// Perform actual new scene creation
    pub fn perform_new_scene(&mut self) {
        info!("Creating new scene");
        self.pending_scene_operation = Some(SceneOperation::NewScene);
        self.current_scene_path = None;
        self.scene_modified = false;
    }

    /// Handle load scene action
    pub fn load_scene_action(&mut self) {
        if self.scene_modified {
            self.show_unsaved_dialog = true;
            self.pending_action = Some(PendingAction::LoadScene);
        } else {
            self.perform_load_scene();
        }
    }

    /// Perform actual scene loading
    pub fn perform_load_scene(&mut self) {
        if let Some(path) = self.show_open_dialog() {
            info!("Loading scene from: {:?}", path);
            self.pending_scene_operation = Some(SceneOperation::LoadScene(path.clone()));
            self.current_scene_path = Some(path);
            self.scene_modified = false;
        }
    }

    /// Handle save scene action
    pub fn save_scene_action(&mut self) -> bool {
        if let Some(path) = self.current_scene_path.clone() {
            self.save_scene_to_path(&path)
        } else {
            self.save_scene_as_action()
        }
    }

    /// Handle save scene as action
    pub fn save_scene_as_action(&mut self) -> bool {
        if let Some(path) = self.show_save_dialog() {
            self.save_scene_to_path(&path)
        } else {
            false
        }
    }

    /// Save scene to specific path
    fn save_scene_to_path(&mut self, path: &PathBuf) -> bool {
        info!("Saving scene to: {:?}", path);
        self.pending_scene_operation = Some(SceneOperation::SaveScene(path.clone()));
        self.current_scene_path = Some(path.clone());
        self.scene_modified = false;
        true
    }

    /// Show save dialog
    fn show_save_dialog(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Save Scene")
            .add_filter("Scene files", &["json"])
            .add_filter("All files", &["*"])
            .set_file_name("untitled.json")
            .save_file()
    }

    /// Show open dialog
    fn show_open_dialog(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title("Open Scene")
            .add_filter("Scene files", &["json"])
            .add_filter("All files", &["*"])
            .pick_file()
    }

    /// Mark the scene as modified
    pub fn mark_scene_modified(&mut self) {
        self.scene_modified = true;
    }
}
