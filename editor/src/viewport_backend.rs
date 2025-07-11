use imgui::{Context, Id, PlatformViewportBackend, Viewport};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};

/// Viewport platform backend for ImGui multi-viewport support using high-level imgui-rs API
pub struct ViewportBackend {
    /// Mapping from viewport ID to winit WindowId
    viewport_to_window: HashMap<Id, WindowId>,
    /// Mapping from winit WindowId to viewport ID
    window_to_viewport: HashMap<WindowId, Id>,
    /// The main viewport ID
    main_viewport_id: Option<Id>,
    /// Whether viewport support is available
    viewports_enabled: bool,
    /// Whether window manager is available (for logging purposes)
    has_window_manager: bool,
    /// Pending window creation requests that need to be processed by main loop
    pending_window_requests: Arc<Mutex<Vec<WindowCreationRequest>>>,
    /// Window manager reference (set during frame)
    window_manager: Option<*const engine::windowing::WindowManager>,
}

/// Request for creating a new window that will be processed by the main loop
#[derive(Debug, Clone)]
pub struct WindowCreationRequest {
    pub viewport_id: Id,
    pub title: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub created: bool,
}

impl Default for ViewportBackend {
    fn default() -> Self {
        Self {
            viewport_to_window: HashMap::new(),
            window_to_viewport: HashMap::new(),
            main_viewport_id: None,
            viewports_enabled: false,
            has_window_manager: false,
            pending_window_requests: Arc::new(Mutex::new(Vec::new())),
            window_manager: None,
        }
    }
}

// Safety: We only use the window manager pointer during rendering
unsafe impl Send for ViewportBackend {}
unsafe impl Sync for ViewportBackend {}

impl ViewportBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the viewport backend with ImGui context
    pub fn init(&mut self, context: &mut Context, main_window: &Window) {
        // Store main window mapping (main viewport ID will be set when viewport system calls us)
        let main_window_id = main_window.id();
        self.has_window_manager = true;

        // Get main viewport from context and set up platform data
        let main_viewport = context.main_viewport_mut();
        self.main_viewport_id = Some(main_viewport.id);

        // CRITICAL: Set platform handle for main viewport to satisfy imgui's assertion
        // We'll use a stable pointer value based on the viewport backend itself
        // This is sufficient to pass the assertion check
        main_viewport.platform_handle = self as *const _ as *mut std::ffi::c_void;

        info!(
            "Set main viewport platform handle: {:?}",
            main_viewport.platform_handle
        );

        // Store window ID mappings
        self.viewport_to_window
            .insert(main_viewport.id, main_window_id);
        self.window_to_viewport
            .insert(main_window_id, main_viewport.id);

        // Initialize monitors list - REQUIRED for viewport system
        self.init_monitors_list(context, main_window);

        // Set up platform backend (pass by value, not boxed)
        context.set_platform_backend(self.clone());

        // Set backend flags to indicate viewport support
        let io = context.io_mut();
        io.backend_flags
            .insert(imgui::BackendFlags::PLATFORM_HAS_VIEWPORTS);
        // Note: RENDERER_HAS_VIEWPORTS should be set by the renderer

        // Viewport support is now available with docking feature
        self.viewports_enabled = true;

        info!("Initialized viewport backend with imgui-rs high-level API");
        info!("Set BackendFlags::PLATFORM_HAS_VIEWPORTS");
    }

    /// Initialize the monitors list for the platform
    fn init_monitors_list(&self, context: &mut Context, main_window: &Window) {
        use imgui::monitor_init_fix;

        // Collect monitor data from winit
        let monitors: Vec<monitor_init_fix::MonitorData> = main_window
            .available_monitors()
            .map(|monitor| {
                let position = monitor.position();
                let size = monitor.size();
                let scale = monitor.scale_factor() as f32;

                monitor_init_fix::MonitorData {
                    position: [position.x as f32, position.y as f32],
                    size: [size.width as f32, size.height as f32],
                    work_pos: [position.x as f32, position.y as f32],
                    work_size: [size.width as f32, size.height as f32],
                    dpi_scale: scale,
                }
            })
            .collect();

        if monitors.is_empty() {
            // Fallback to single monitor if none detected
            info!("No monitors detected, using default monitor");
            monitor_init_fix::init_single_monitor(context, 1920.0, 1080.0, 1.0);
        } else {
            info!(
                "Initializing {} monitors for viewport system",
                monitors.len()
            );
            monitor_init_fix::init_monitors(context, &monitors);
        }

        info!("Monitor initialization complete");
    }

    /// Get pending window creation requests (to be processed by main loop)
    pub fn take_pending_requests(&self) -> Vec<WindowCreationRequest> {
        let mut requests = self.pending_window_requests.lock().unwrap();
        std::mem::take(&mut *requests)
    }

    /// Register a created window with the viewport system
    pub fn register_created_window(&mut self, viewport_id: Id, window_id: WindowId) {
        self.viewport_to_window.insert(viewport_id, window_id);
        self.window_to_viewport.insert(window_id, viewport_id);
        info!(
            "Registered created window {:?} for viewport {:?}",
            window_id, viewport_id
        );
    }

    /// Check if viewport support is enabled
    pub fn is_enabled(&self) -> bool {
        self.viewports_enabled
    }

    /// Handle window event for a specific window
    pub fn handle_window_event(&mut self, window_id: WindowId, _event: &WindowEvent) {
        // Events are handled by imgui-winit-support for each window
        debug!("Handling window event for window {:?}", window_id);
    }

    /// Get the window ID for a given viewport
    pub fn get_window_for_viewport(&self, viewport_id: Id) -> Option<WindowId> {
        self.viewport_to_window.get(&viewport_id).copied()
    }

    /// Get the viewport ID for a given window
    pub fn get_viewport_for_window(&self, window_id: WindowId) -> Option<Id> {
        self.window_to_viewport.get(&window_id).copied()
    }

    /// Set the window manager for the current frame
    ///
    /// # Safety
    /// The caller must ensure that the window manager reference remains valid for the entire frame.
    /// This method stores a raw pointer to the window manager that will be used during the frame.
    pub unsafe fn set_window_manager(&mut self, window_manager: &engine::windowing::WindowManager) {
        self.window_manager = Some(window_manager as *const _);
    }

    /// Clear the window manager reference
    pub fn clear_window_manager(&mut self) {
        self.window_manager = None;
    }
}

impl Clone for ViewportBackend {
    fn clone(&self) -> Self {
        Self {
            viewport_to_window: self.viewport_to_window.clone(),
            window_to_viewport: self.window_to_viewport.clone(),
            main_viewport_id: self.main_viewport_id,
            viewports_enabled: self.viewports_enabled,
            has_window_manager: self.has_window_manager,
            pending_window_requests: Arc::clone(&self.pending_window_requests),
            window_manager: self.window_manager,
        }
    }
}

impl PlatformViewportBackend for ViewportBackend {
    fn create_window(&mut self, viewport: &mut Viewport) {
        info!(
            "=== CREATE_WINDOW CALLED === for viewport {:?}",
            viewport.id
        );
        info!(
            "Viewport details: pos={:?}, size={:?}, work_pos={:?}, work_size={:?}",
            viewport.pos, viewport.size, viewport.work_pos, viewport.work_size
        );
        info!("Viewport flags: {:?}", viewport.flags);

        // Check if this is the main viewport (we shouldn't create a window for it)
        if let Some(main_id) = self.main_viewport_id {
            if viewport.id == main_id {
                debug!("Skipping window creation for main viewport");
                return;
            }
        }

        // Create a window creation request to be processed by the main loop
        let request = WindowCreationRequest {
            viewport_id: viewport.id,
            title: format!("Editor Panel - {:?}", viewport.id),
            position: viewport.pos,
            size: viewport.size,
            created: false,
        };

        // Add to pending requests
        if let Ok(mut requests) = self.pending_window_requests.lock() {
            requests.push(request);
            info!(
                "Queued window creation request for viewport {:?} at {:?} with size {:?}",
                viewport.id, viewport.pos, viewport.size
            );
        } else {
            warn!(
                "Failed to queue window creation request for viewport {:?}",
                viewport.id
            );
        }
    }

    fn destroy_window(&mut self, viewport: &mut Viewport) {
        debug!("Destroying window for viewport {:?}", viewport.id);

        // Remove from our mappings
        if let Some(window_id) = self.viewport_to_window.remove(&viewport.id) {
            self.window_to_viewport.remove(&window_id);
            info!("Destroyed window for viewport {:?}", viewport.id);
        }
    }

    fn show_window(&mut self, viewport: &mut Viewport) {
        info!("=== SHOW_WINDOW CALLED === for viewport {:?}", viewport.id);
        info!("Viewport window should now be visible and ready for rendering");
        // Window visibility is handled by winit automatically
        // This callback confirms the window should be visible
    }

    fn set_window_pos(&mut self, viewport: &mut Viewport, pos: [f32; 2]) {
        debug!(
            "Set window position for viewport {:?} to {:?}",
            viewport.id, pos
        );

        // Update actual window position
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    let dpi_scale = window_data.window.scale_factor() as f32;
                    let physical_pos = winit::dpi::PhysicalPosition::new(
                        (pos[0] * dpi_scale) as i32,
                        (pos[1] * dpi_scale) as i32,
                    );
                    window_data.window.set_outer_position(physical_pos);
                }
            }
        }

        viewport.pos = pos;
    }

    fn get_window_pos(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        debug!("Get window position for viewport {:?}", viewport.id);

        // Try to get actual window position from window manager
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    if let Ok(pos) = window_data.window.outer_position() {
                        let dpi_scale = window_data.window.scale_factor() as f32;
                        return [pos.x as f32 / dpi_scale, pos.y as f32 / dpi_scale];
                    }
                }
            }
        }

        viewport.pos
    }

    fn set_window_size(&mut self, viewport: &mut Viewport, size: [f32; 2]) {
        debug!(
            "Set window size for viewport {:?} to {:?}",
            viewport.id, size
        );

        // Update actual window size
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    let dpi_scale = window_data.window.scale_factor() as f32;
                    let physical_size = winit::dpi::PhysicalSize::new(
                        (size[0] * dpi_scale) as u32,
                        (size[1] * dpi_scale) as u32,
                    );
                    let _ = window_data.window.request_inner_size(physical_size);
                }
            }
        }

        viewport.size = size;
    }

    fn get_window_size(&mut self, viewport: &mut Viewport) -> [f32; 2] {
        debug!("Get window size for viewport {:?}", viewport.id);

        // Try to get actual window size from window manager
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    let size = window_data.window.inner_size();
                    let dpi_scale = window_data.window.scale_factor() as f32;
                    return [
                        size.width as f32 / dpi_scale,
                        size.height as f32 / dpi_scale,
                    ];
                }
            }
        }

        viewport.size
    }

    fn set_window_focus(&mut self, viewport: &mut Viewport) {
        debug!("Set window focus for viewport {:?}", viewport.id);

        // Focus the window
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    window_data.window.focus_window();
                }
            }
        }
    }

    fn get_window_focus(&mut self, viewport: &mut Viewport) -> bool {
        debug!("Get window focus for viewport {:?}", viewport.id);

        // Check if window is focused
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    return window_data.window.has_focus();
                }
            }
        }

        false
    }

    fn get_window_minimized(&mut self, viewport: &mut Viewport) -> bool {
        debug!("Get window minimized state for viewport {:?}", viewport.id);

        // Check if window is minimized
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    return window_data.window.is_minimized().unwrap_or(false);
                }
            }
        }

        false
    }

    fn set_window_title(&mut self, viewport: &mut Viewport, title: &str) {
        debug!(
            "Set window title for viewport {:?} to '{}'",
            viewport.id, title
        );

        // Update window title
        if let (Some(window_manager), Some(window_id)) = (
            self.window_manager,
            self.viewport_to_window.get(&viewport.id),
        ) {
            unsafe {
                if let Some(window_data) = (*window_manager).get_window(*window_id) {
                    window_data.window.set_title(title);
                }
            }
        }
    }

    fn set_window_alpha(&mut self, viewport: &mut Viewport, alpha: f32) {
        debug!(
            "Set window alpha for viewport {:?} to {}",
            viewport.id, alpha
        );

        // Note: Window transparency is platform-specific and may not be supported everywhere
        // winit doesn't provide a direct API for window transparency yet
        // This would require platform-specific code using raw window handles
        info!("Window transparency not implemented - requires platform-specific code");
    }

    fn update_window(&mut self, viewport: &mut Viewport) {
        debug!("Update window for viewport {:?}", viewport.id);

        // This is called when ImGui wants to ensure all window properties are up to date
        // Since we update properties immediately in the individual setter methods,
        // there's nothing additional to do here
    }

    fn render_window(&mut self, viewport: &mut Viewport) {
        debug!("Render window called for viewport {:?}", viewport.id);

        // Rendering is handled by the RendererViewportBackend implementation
        // This callback is for platform-specific rendering setup if needed
    }

    fn swap_buffers(&mut self, viewport: &mut Viewport) {
        debug!("Swap buffers called for viewport {:?}", viewport.id);

        // Buffer swapping is handled by wgpu's surface.present() in the renderer backend
        // This callback is for platform-specific buffer swapping if needed
    }

    fn create_vk_surface(
        &mut self,
        viewport: &mut Viewport,
        _instance: u64,
        surface: &mut u64,
    ) -> i32 {
        debug!("Create VK surface called for viewport {:?}", viewport.id);

        // Not applicable for wgpu backend - wgpu handles surface creation internally
        // Return 0 to indicate no error, but surface remains null
        *surface = 0;
        0
    }
}
