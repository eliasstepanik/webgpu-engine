//! Main editor state management
//!
//! This module contains the EditorState struct which manages the imgui context,
//! render target for viewport, and all editor UI state.

use crate::detached_window_manager::DetachedWindowManager;
use crate::panel_state::PanelManager;
use crate::safe_imgui_renderer::SafeImGuiRenderer;
use crate::shared_state::EditorSharedState;
#[cfg(feature = "viewport")]
use crate::viewport_backend::ViewportBackend;
use engine::core::entity::World;
use engine::graphics::{context::RenderContext, render_target::RenderTarget, RenderTargetInfo};
use engine::windowing::WindowManager;
use imgui::*;
use imgui_wgpu::RendererConfig;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};
use winit::event::{Event, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Pending scene action to perform after confirmation
#[derive(Debug, Clone)]
pub enum PendingAction {
    NewScene,
    LoadScene,
    Exit,
}

/// Menu actions that need to be handled after UI rendering
struct MenuActions {
    new_scene: bool,
    load_scene: bool,
    save_scene: bool,
    save_scene_as: bool,
    exit: bool,
    save_layout: bool,
    load_layout: bool,
    reset_layout: bool,
}

/// Dialog actions that need to be handled after UI rendering
struct DialogActions {
    save_and_proceed: bool,
    dont_save_and_proceed: bool,
    cancel: bool,
    clear_error: bool,
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
    /// ImGui-wgpu renderer with safety validation
    pub imgui_renderer: SafeImGuiRenderer,
    /// Render target for viewport texture
    pub render_target: RenderTarget,
    /// ImGui texture ID for the render target
    pub texture_id: imgui::TextureId,
    /// Shared state for multi-window synchronization
    pub shared_state: EditorSharedState,
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
    /// Surface format for rendering
    surface_format: wgpu::TextureFormat,
    /// Panel manager
    pub panel_manager: PanelManager,
    /// Detached window manager
    pub detached_window_manager: Option<DetachedWindowManager>,
    /// Pending menu actions to be handled after UI rendering
    pending_menu_actions: Option<MenuActions>,
    /// Pending dialog actions to be handled after UI rendering
    pending_dialog_actions: Option<DialogActions>,
    /// Viewport backend for multi-window support
    #[cfg(feature = "viewport")]
    pub viewport_backend: Option<ViewportBackend>,
    /// Enhanced viewport renderer
    #[cfg(feature = "viewport")]
    pub viewport_renderer: Option<crate::enhanced_viewport_renderer::EnhancedViewportRenderer>,
    /// Shared viewport renderer backend
    #[cfg(feature = "viewport")]
    pub viewport_renderer_backend:
        Option<Arc<Mutex<crate::viewport_renderer_backend::ViewportRendererBackend>>>,
}

impl EditorState {
    /// Create a new editor state
    pub fn new(
        render_context: &RenderContext,
        window: &winit::window::Window,
        surface_format: wgpu::TextureFormat,
        surface_size: (u32, u32),
        world: World,
    ) -> Self {
        info!("Initializing editor state with ImGui");

        // Create ImGui context
        let mut imgui_context = imgui::Context::create();

        // Configure ImGui
        imgui_context.set_ini_filename(None); // Don't save settings to file

        // Enable viewport support for multi-window functionality
        {
            let io = imgui_context.io_mut();
            io.config_flags |= ConfigFlags::VIEWPORTS_ENABLE;
            io.config_flags |= ConfigFlags::DOCKING_ENABLE;
        }
        info!("Enabled ImGui viewport and docking support");

        // Set up some styling
        let style = imgui_context.style_mut();
        style.window_rounding = 0.0;
        style.scrollbar_rounding = 0.0;

        // Create platform integration
        let mut imgui_platform = WinitPlatform::new(&mut imgui_context);

        // Use the provided surface size
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

        // Use the provided surface format

        // Create renderer
        let renderer_config = RendererConfig {
            texture_format: surface_format,
            ..Default::default()
        };

        let mut imgui_renderer = SafeImGuiRenderer::new(
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
            imgui_renderer.inner(),
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
        let texture_id = imgui_renderer.textures().insert(imgui_texture);

        info!("Editor initialized - Press Tab to toggle between Editor UI and Game Input modes");

        // Force initial event processing to ensure imgui is properly initialized
        imgui_platform
            .prepare_frame(imgui_context.io_mut(), window)
            .expect("Initial frame preparation failed");

        // Create shared state for multi-window synchronization
        let shared_state = EditorSharedState::new(world);

        let mut editor = Self {
            imgui_context,
            imgui_platform,
            imgui_renderer,
            render_target,
            texture_id,
            shared_state,
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
            surface_format,
            panel_manager: PanelManager::with_layout_file(PanelManager::default_layout_path()),
            detached_window_manager: None,
            pending_menu_actions: None,
            pending_dialog_actions: None,
            #[cfg(feature = "viewport")]
            viewport_backend: None,
            #[cfg(feature = "viewport")]
            viewport_renderer: None,
            #[cfg(feature = "viewport")]
            viewport_renderer_backend: None,
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

    /// Initialize the viewport backend for multi-window support
    #[cfg(feature = "viewport")]
    pub fn init_viewport_backend(
        &mut self,
        window: &winit::window::Window,
        render_context: &RenderContext,
    ) {
        if self.viewport_backend.is_none() {
            let mut viewport_backend = ViewportBackend::new();
            viewport_backend.init(&mut self.imgui_context, window);
            self.viewport_backend = Some(viewport_backend);
            info!("Initialized viewport backend for multi-window support");
        }

        if self.viewport_renderer.is_none() {
            use crate::enhanced_viewport_renderer::EnhancedViewportRenderer;

            let renderer_config = imgui_wgpu::RendererConfig {
                texture_format: self.surface_format,
                ..Default::default()
            };

            let viewport_renderer = EnhancedViewportRenderer::new(
                &mut self.imgui_context,
                render_context.device.clone(),
                render_context.queue.clone(),
                renderer_config,
            );

            self.viewport_renderer = Some(viewport_renderer);
            info!("Initialized enhanced viewport renderer");

            // Set up the renderer viewport backend
            use crate::viewport_renderer_backend::ViewportRendererBackend;
            let renderer_backend = ViewportRendererBackend::new(
                render_context.device.clone(),
                render_context.queue.clone(),
                self.surface_format,
                &mut self.imgui_context,
            );

            let shared_backend = Arc::new(Mutex::new(renderer_backend));
            self.viewport_renderer_backend = Some(shared_backend.clone());

            // Create a wrapper that delegates to the shared backend
            struct SharedBackendWrapper {
                backend: Arc<Mutex<ViewportRendererBackend>>,
            }

            impl imgui::RendererViewportBackend for SharedBackendWrapper {
                fn create_window(&mut self, viewport: &mut Viewport) {
                    self.backend.lock().unwrap().create_window(viewport);
                }

                fn destroy_window(&mut self, viewport: &mut Viewport) {
                    self.backend.lock().unwrap().destroy_window(viewport);
                }

                fn set_window_size(&mut self, viewport: &mut Viewport, size: [f32; 2]) {
                    self.backend.lock().unwrap().set_window_size(viewport, size);
                }

                fn render_window(&mut self, viewport: &mut Viewport) {
                    self.backend.lock().unwrap().render_window(viewport);
                }

                fn swap_buffers(&mut self, viewport: &mut Viewport) {
                    self.backend.lock().unwrap().swap_buffers(viewport);
                }
            }

            let wrapper = SharedBackendWrapper {
                backend: shared_backend,
            };

            self.imgui_context.set_renderer_backend(wrapper);
            info!("Set renderer viewport backend with shared access");
        }
    }

    /// Initialize the detached window manager
    pub fn init_detached_window_manager(&mut self, render_context: Arc<RenderContext>) {
        self.detached_window_manager = Some(DetachedWindowManager::new(
            render_context,
            self.surface_format,
        ));
        info!("Initialized detached window manager");
    }

    /// Render all viewports using the enhanced renderer
    #[cfg(feature = "viewport")]
    pub fn render_all_viewports(
        &mut self,
        window_manager: &engine::windowing::WindowManager,
        clear_color: wgpu::Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(renderer) = &mut self.viewport_renderer {
            renderer.render_all_viewports(&mut self.imgui_context, window_manager, clear_color)?;
        }
        Ok(())
    }

    /// Process pending viewport window creation requests
    #[cfg(feature = "viewport")]
    pub fn process_viewport_requests(
        &mut self,
        window_manager: &mut engine::windowing::WindowManager,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        // Process pending detach requests from panel manager
        let pending_detach = self.panel_manager.take_pending_detach();
        let pending_attach = self.panel_manager.take_pending_attach();

        // Mark panels for viewport detachment
        for panel_id in pending_detach {
            if let Some(panel) = self.panel_manager.get_panel_mut(&panel_id) {
                // For viewport system, we just mark the panel as wanting to be detached
                // The actual viewport/window will be created by ImGui when we render
                // the window without the NoViewport flag
                panel.is_detached = true;
                info!("Marked panel for viewport detachment: {:?}", panel_id);
            }
        }

        // Process reattachment requests
        for panel_id in pending_attach {
            if let Some(panel) = self.panel_manager.get_panel_mut(&panel_id) {
                panel.attach();
                info!("Reattached panel: {:?}", panel_id);
            }
        }

        if let Some(viewport_backend) = &mut self.viewport_backend {
            let requests = viewport_backend.take_pending_requests();

            if !requests.is_empty() {
                info!(
                    "Found {} viewport window creation requests to process",
                    requests.len()
                );
            }

            for request in requests {
                info!("Processing viewport window creation request: {:?}", request);

                // Get the DPI scale from the main window
                let dpi_scale = window_manager.get_main_window().window.scale_factor() as f32;

                // Create window attributes
                // The request size is in logical pixels, we need to convert to physical
                let window_attributes = winit::window::WindowAttributes::default()
                    .with_title(&request.title)
                    .with_inner_size(winit::dpi::PhysicalSize::new(
                        (request.size[0] * dpi_scale) as u32,
                        (request.size[1] * dpi_scale) as u32,
                    ))
                    .with_position(winit::dpi::PhysicalPosition::new(
                        (request.position[0] * dpi_scale) as i32,
                        (request.position[1] * dpi_scale) as i32,
                    ));

                // Create the window
                match event_loop.create_window(window_attributes) {
                    Ok(window) => {
                        let window = Arc::new(window);
                        let _window_id = window.id();

                        // Create surface configuration for the new window
                        // Use the actual physical size from the created window
                        let window_size = window.inner_size();
                        let surface_config = wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format: self.surface_format,
                            width: window_size.width,
                            height: window_size.height,
                            present_mode: wgpu::PresentMode::Fifo,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        };

                        // Add window to window manager
                        match window_manager.create_window(window, surface_config) {
                            Ok(created_window_id) => {
                                // Register the created window with viewport backend
                                viewport_backend.register_created_window(
                                    request.viewport_id,
                                    created_window_id,
                                );

                                // Register with viewport renderer if available
                                if let Some(renderer) = &mut self.viewport_renderer {
                                    renderer.on_viewport_created(
                                        &mut self.imgui_context,
                                        request.viewport_id,
                                        created_window_id,
                                    );
                                }

                                // Register with shared renderer backend
                                if let Some(backend) = &self.viewport_renderer_backend {
                                    backend
                                        .lock()
                                        .unwrap()
                                        .register_viewport(request.viewport_id, created_window_id);
                                }

                                info!(
                                    "Successfully created viewport window {:?} for viewport {:?}",
                                    created_window_id, request.viewport_id
                                );
                            }
                            Err(e) => {
                                warn!("Failed to add viewport window to manager: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to create viewport window: {}", e);
                    }
                }
            }
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

    /// Handle viewport-specific events
    #[cfg(feature = "viewport")]
    pub fn handle_viewport_event(
        &mut self,
        event: &Event<()>,
        window_id: winit::window::WindowId,
        window_manager: &WindowManager,
    ) {
        use tracing::trace;

        // Special handling for focus events
        if let Event::WindowEvent {
            event: WindowEvent::Focused(focused),
            window_id: event_window_id,
        } = event
        {
            if *event_window_id == window_id {
                trace!(
                    window_id = ?window_id,
                    focused = focused,
                    "Viewport focus event"
                );

                // If a viewport is focused, ensure main window continues processing
                if *focused && window_id != window_manager.main_window_id() {
                    let main_window_data = window_manager.get_main_window();
                    main_window_data.window.request_redraw();
                }
            }
        }

        // Pass events to viewport backend if available
        if let Some(viewport_backend) = &mut self.viewport_backend {
            if let Event::WindowEvent {
                event: window_event,
                ..
            } = event
            {
                viewport_backend.handle_window_event(window_id, window_event);
            }
        }
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
        // Get surface size from window
        let window_size = window.inner_size();
        let surface_size = (window_size.width, window_size.height);

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
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &winit::window::Window,
        window_manager: &WindowManager,
    ) {
        #[cfg(feature = "viewport")]
        {
            // If viewport renderer is available, use it for all rendering
            if let Some(_viewport_renderer) = &mut self.viewport_renderer {
                self.render_with_viewports(render_context, encoder, view, window, window_manager);
                return;
            }
        }

        // Fall back to single-window rendering
        self.render_single_window(render_context, encoder, view, window);
    }

    /// Render using the enhanced viewport system
    #[cfg(feature = "viewport")]
    fn render_with_viewports(
        &mut self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &winit::window::Window,
        window_manager: &WindowManager,
    ) {
        // NOTE: begin_frame must have been called before this
        // which calls prepare_frame and sets up the frame

        // Check size consistency like in single window renderer
        let window_size = window.inner_size();
        let surface_size = (window_size.width, window_size.height);
        let io = self.imgui_context.io();
        let imgui_phys = (
            (io.display_size[0] * io.display_framebuffer_scale[0]).round() as u32,
            (io.display_size[1] * io.display_framebuffer_scale[1]).round() as u32,
        );

        if imgui_phys != surface_size {
            tracing::error!(
                "Size mismatch in viewport renderer: surface={}x{}, imgui_physical={}x{}",
                surface_size.0,
                surface_size.1,
                imgui_phys.0,
                imgui_phys.1
            );
            return;
        }

        // Build the UI - this creates draw data
        let ui = self.imgui_context.new_frame();

        // Build the actual editor UI inline to avoid borrow issues
        {
            // Store actions to take after UI rendering
            let mut action_new_scene = false;
            let mut action_load_scene = false;
            let mut action_save_scene = false;
            let mut action_save_scene_as = false;
            let mut action_exit = false;
            let mut action_save_layout = false;
            let mut action_load_layout = false;
            let mut action_reset_layout = false;

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
                    if ui.menu_item("Save Layout") {
                        action_save_layout = true;
                    }
                    if ui.menu_item("Load Layout") {
                        action_load_layout = true;
                    }
                    ui.separator();
                    if ui.menu_item("Reset Layout") {
                        action_reset_layout = true;
                    }
                });
                ui.menu("Help", || {
                    if ui.menu_item("About") {
                        info!("About requested");
                    }
                });
            });

            // Panels ---------------------------------------------------------------
            crate::panels::render_hierarchy_panel(ui, &self.shared_state, &mut self.panel_manager);
            crate::panels::render_inspector_panel(ui, &self.shared_state, &mut self.panel_manager);
            crate::panels::render_assets_panel(ui, &self.shared_state, &mut self.panel_manager);

            // Central viewport that displays the 3D scene
            crate::panels::render_viewport_panel(
                ui,
                self.texture_id,
                &self.render_target,
                &self.shared_state,
                &mut self.panel_manager,
            );

            // Dialog handling
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

            // Store actions for deferred handling
            self.pending_menu_actions = Some(MenuActions {
                new_scene: action_new_scene,
                load_scene: action_load_scene,
                save_scene: action_save_scene,
                save_scene_as: action_save_scene_as,
                exit: action_exit,
                save_layout: action_save_layout,
                load_layout: action_load_layout,
                reset_layout: action_reset_layout,
            });

            self.pending_dialog_actions = Some(DialogActions {
                save_and_proceed: dialog_save_and_proceed,
                dont_save_and_proceed: dialog_dont_save_and_proceed,
                cancel: dialog_cancel,
                clear_error,
            });
        }

        // Render main viewport
        let draw_data = self.imgui_context.render();

        // Render the main viewport to the screen
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ImGui Main Viewport Pass"),
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

            self.imgui_renderer
                .render_with_validation(
                    draw_data,
                    &render_context.queue,
                    &render_context.device,
                    &mut render_pass,
                    RenderTargetInfo {
                        width: surface_size.0,
                        height: surface_size.1,
                    },
                )
                .expect("ImGui rendering failed");
        }

        // Update platform windows after render
        self.imgui_context.update_platform_windows();

        // Set window manager on platform backend before rendering
        if let Some(backend) = &mut self.viewport_backend {
            unsafe {
                backend.set_window_manager(window_manager);
            }
        }

        // Set window manager on renderer backend before rendering
        if let Some(backend) = &self.viewport_renderer_backend {
            unsafe {
                backend.lock().unwrap().set_window_manager(window_manager);
            }
        }

        // Render additional platform windows
        self.imgui_context.render_platform_windows_default();

        // Clear window manager reference after rendering
        if let Some(backend) = &mut self.viewport_backend {
            backend.clear_window_manager();
        }
        if let Some(backend) = &self.viewport_renderer_backend {
            backend.lock().unwrap().clear_window_manager();
        }

        debug!("Viewport rendering cycle complete");

        // Handle deferred actions after rendering
        self.handle_deferred_actions();
    }

    /// Handle deferred menu and dialog actions after UI rendering
    fn handle_deferred_actions(&mut self) {
        // Handle menu actions
        if let Some(actions) = self.pending_menu_actions.take() {
            if actions.new_scene {
                self.new_scene_action();
            }
            if actions.load_scene {
                self.load_scene_action();
            }
            if actions.save_scene {
                self.save_scene_action();
            }
            if actions.save_scene_as {
                self.save_scene_as_action();
            }
            if actions.exit {
                if self.scene_modified {
                    self.show_unsaved_dialog = true;
                    self.pending_action = Some(PendingAction::Exit);
                } else {
                    std::process::exit(0);
                }
            }

            // Handle layout actions
            if actions.save_layout {
                match self.panel_manager.save_default_layout() {
                    Ok(_) => {
                        info!("Layout saved successfully");
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to save layout: {e}"));
                    }
                }
            }
            if actions.load_layout {
                match self.panel_manager.load_default_layout() {
                    Ok(_) => {
                        info!("Layout loaded successfully");
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load layout: {e}"));
                    }
                }
            }
            if actions.reset_layout {
                self.panel_manager = PanelManager::default();
                info!("Layout reset to default");
            }
        }

        // Handle dialog actions
        if let Some(actions) = self.pending_dialog_actions.take() {
            if actions.save_and_proceed && self.save_scene_action() {
                self.show_unsaved_dialog = false;
                if let Some(action) = self.pending_action.take() {
                    match action {
                        PendingAction::NewScene => self.perform_new_scene(),
                        PendingAction::LoadScene => self.perform_load_scene(),
                        PendingAction::Exit => std::process::exit(0),
                    }
                }
            }
            if actions.dont_save_and_proceed {
                self.show_unsaved_dialog = false;
                if let Some(action) = self.pending_action.take() {
                    match action {
                        PendingAction::NewScene => self.perform_new_scene(),
                        PendingAction::LoadScene => self.perform_load_scene(),
                        PendingAction::Exit => std::process::exit(0),
                    }
                }
            }
            if actions.cancel {
                self.show_unsaved_dialog = false;
                self.pending_action = None;
            }
            if actions.clear_error {
                self.error_message = None;
            }
        }
    }

    /// Render using single window (original implementation)
    fn render_single_window(
        &mut self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        window: &winit::window::Window,
    ) {
        // -------------------------------------------------------------------- sizes
        // Get surface size from window
        let window_size = window.inner_size();
        let surface_size = (window_size.width, window_size.height); // physical pixels
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

        // Build the editor UI inline to avoid borrow issues
        {
            // Store actions to take after UI rendering
            let mut action_new_scene = false;
            let mut action_load_scene = false;
            let mut action_save_scene = false;
            let mut action_save_scene_as = false;
            let mut action_exit = false;
            let mut action_save_layout = false;
            let mut action_load_layout = false;
            let mut action_reset_layout = false;

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
                    if ui.menu_item("Save Layout") {
                        action_save_layout = true;
                    }
                    if ui.menu_item("Load Layout") {
                        action_load_layout = true;
                    }
                    ui.separator();
                    if ui.menu_item("Reset Layout") {
                        action_reset_layout = true;
                    }
                });
                ui.menu("Help", || {
                    if ui.menu_item("About") {
                        info!("About requested");
                    }
                });
            });

            // Panels ---------------------------------------------------------------
            crate::panels::render_hierarchy_panel(ui, &self.shared_state, &mut self.panel_manager);
            crate::panels::render_inspector_panel(ui, &self.shared_state, &mut self.panel_manager);
            crate::panels::render_assets_panel(ui, &self.shared_state, &mut self.panel_manager);

            // Central viewport that displays the 3D scene
            crate::panels::render_viewport_panel(
                ui,
                self.texture_id,
                &self.render_target,
                &self.shared_state,
                &mut self.panel_manager,
            );

            // Dialog handling
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

            // Defer action handling until after UI is rendered (important for viewport mode)
            self.pending_menu_actions = Some(MenuActions {
                new_scene: action_new_scene,
                load_scene: action_load_scene,
                save_scene: action_save_scene,
                save_scene_as: action_save_scene_as,
                exit: action_exit,
                save_layout: action_save_layout,
                load_layout: action_load_layout,
                reset_layout: action_reset_layout,
            });

            self.pending_dialog_actions = Some(DialogActions {
                save_and_proceed: dialog_save_and_proceed,
                dont_save_and_proceed: dialog_dont_save_and_proceed,
                cancel: dialog_cancel,
                clear_error,
            });
        }

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
                let entity_count = self
                    .shared_state
                    .with_world_read(|world| world.query::<()>().iter().count())
                    .unwrap_or(0);
                ui.text(format!("Entities: {entity_count}"));
                ui.same_line();
                ui.separator();
                ui.same_line();

                // Selection
                match self.shared_state.selected_entity() {
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

            if let Err(e) = self.imgui_renderer.render_with_validation(
                draw_data,
                &render_context.queue,
                &render_context.device,
                &mut pass,
                RenderTargetInfo {
                    width: surface_size.0,
                    height: surface_size.1,
                },
            ) {
                tracing::error!("ImGui render failed: {e:?}");
            }
        } // Pass is dropped here

        // Update and render viewports (for multi-window support)
        #[cfg(feature = "viewport")]
        {
            if self.viewport_backend.is_some() {
                // Don't call render_platform_windows_default here
                // It will be handled by the viewport renderer in main loop
                debug!("Viewport rendering will be handled by enhanced renderer");
            }
        }

        // Mark that we're no longer in a frame
        self.in_frame = false;

        // Handle deferred actions after UI rendering
        self.handle_deferred_actions();
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

        // Ignore zero-sized windows (minimized, etc)
        if new_size.width == 0 || new_size.height == 0 {
            debug!("Ignoring resize to zero size");
            return;
        }

        // Get the actual surface size
        // This is critical - we must use the actual size, not the requested size
        let (actual_width, actual_height) = (new_size.width, new_size.height);

        debug!(
            "Surface config size: {}x{} (requested: {}x{})",
            actual_width, actual_height, new_size.width, new_size.height
        );

        // Update ImGui display size to match surface configuration
        let io = self.imgui_context.io_mut();
        io.display_size = [actual_width as f32, actual_height as f32];

        // Use the stored surface format
        let surface_format = self.surface_format;

        // Store the old texture existence status before recreating renderer
        let old_texture_exists = self
            .imgui_renderer
            .textures()
            .get(self.texture_id)
            .is_some();
        debug!(
            "Editor resize: old_texture_exists={}, new_size={:?}",
            old_texture_exists, new_size
        );

        // Only remove if it exists in current renderer
        if old_texture_exists {
            self.imgui_renderer.textures().remove(self.texture_id);
        }

        // CRITICAL: Recreate the imgui renderer to ensure it uses the new viewport size
        // This prevents scissor rect validation errors from cached viewport dimensions
        let renderer_config = RendererConfig {
            texture_format: surface_format,
            ..Default::default()
        };

        self.imgui_renderer = SafeImGuiRenderer::new(
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
            self.imgui_renderer.inner(),
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
        self.texture_id = self.imgui_renderer.textures().insert(imgui_texture);
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

    /// Shutdown the editor and clean up all resources
    pub fn shutdown(&mut self, window_manager: &mut WindowManager) {
        info!("Shutting down editor state");

        // Save panel layout before shutdown
        if let Err(e) = self.panel_manager.save_default_layout() {
            warn!("Failed to save panel layout during shutdown: {}", e);
        }

        // Clean up all detached windows
        if let Some(detached_window_manager) = &mut self.detached_window_manager {
            detached_window_manager.shutdown_all_windows(&mut self.panel_manager, window_manager);
        }

        // Report final state
        if let Some(detached_window_manager) = &self.detached_window_manager {
            info!(
                "Editor shutdown complete. Detached windows cleaned up: {}",
                detached_window_manager.active_window_count() == 0
            );
        }

        info!("Editor shutdown complete");
    }

    /// Get the count of active detached windows
    pub fn active_detached_window_count(&self) -> usize {
        self.detached_window_manager
            .as_ref()
            .map(|mgr| mgr.active_window_count())
            .unwrap_or(0)
    }
}
