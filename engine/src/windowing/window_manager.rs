use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use wgpu::{Instance, Surface, SurfaceConfiguration};
use winit::window::{Window, WindowId};

pub struct WindowData {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
}

pub struct WindowManager {
    windows: HashMap<WindowId, WindowData>,
    main_window_id: WindowId,
    instance: Arc<Instance>,
    device: Arc<wgpu::Device>,
    max_windows: usize,
}

impl WindowManager {
    pub fn new(
        main_window: Arc<Window>,
        instance: Arc<Instance>,
        device: Arc<wgpu::Device>,
        surface_config: SurfaceConfiguration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let main_window_id = main_window.id();
        let surface = instance.create_surface(Arc::clone(&main_window))?;

        surface.configure(&device, &surface_config);

        let mut windows = HashMap::new();
        windows.insert(
            main_window_id,
            WindowData {
                window: main_window,
                surface,
                surface_config,
            },
        );

        info!(window_id = ?main_window_id, "Created window manager with main window");

        Ok(Self {
            windows,
            main_window_id,
            instance,
            device,
            max_windows: 4, // Limit to 4 windows as per PRP
        })
    }

    pub fn create_window(
        &mut self,
        window: Arc<Window>,
        config: SurfaceConfiguration,
    ) -> Result<WindowId, Box<dyn std::error::Error>> {
        if self.windows.len() >= self.max_windows {
            warn!("Maximum window limit reached: {}", self.max_windows);
            return Err("Maximum window limit reached".into());
        }

        let window_id = window.id();
        let surface = self.instance.create_surface(Arc::clone(&window))?;

        surface.configure(&self.device, &config);

        self.windows.insert(
            window_id,
            WindowData {
                window,
                surface,
                surface_config: config,
            },
        );

        info!(window_id = ?window_id, "Created new window");
        Ok(window_id)
    }

    pub fn destroy_window(
        &mut self,
        window_id: WindowId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if window_id == self.main_window_id {
            return Err("Cannot destroy main window".into());
        }

        if self.windows.remove(&window_id).is_some() {
            info!(window_id = ?window_id, "Destroyed window");
            Ok(())
        } else {
            warn!(window_id = ?window_id, "Attempted to destroy non-existent window");
            Err("Window not found".into())
        }
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&WindowData> {
        self.windows.get(&window_id)
    }

    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut WindowData> {
        self.windows.get_mut(&window_id)
    }

    pub fn get_main_window(&self) -> &WindowData {
        self.windows
            .get(&self.main_window_id)
            .expect("Main window should always exist")
    }

    pub fn get_main_window_mut(&mut self) -> &mut WindowData {
        self.windows
            .get_mut(&self.main_window_id)
            .expect("Main window should always exist")
    }

    pub fn main_window_id(&self) -> WindowId {
        self.main_window_id
    }

    pub fn window_ids(&self) -> impl Iterator<Item = &WindowId> {
        self.windows.keys()
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn resize_window(&mut self, window_id: WindowId, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(window_data) = self.windows.get_mut(&window_id) {
            if new_size.width > 0 && new_size.height > 0 {
                window_data.surface_config.width = new_size.width;
                window_data.surface_config.height = new_size.height;
                window_data
                    .surface
                    .configure(&self.device, &window_data.surface_config);

                debug!(
                    window_id = ?window_id,
                    width = new_size.width,
                    height = new_size.height,
                    "Resized window"
                );
            }
        }
    }

    pub fn handle_scale_factor_changed(
        &mut self,
        window_id: WindowId,
        scale_factor: f64,
        new_inner_size: winit::dpi::PhysicalSize<u32>,
    ) {
        if self.windows.contains_key(&window_id) {
            debug!(
                window_id = ?window_id,
                scale_factor = scale_factor,
                "Scale factor changed"
            );
            self.resize_window(window_id, new_inner_size);
        }
    }

    pub fn is_window_minimized(&self, window_id: WindowId) -> bool {
        if let Some(window_data) = self.windows.get(&window_id) {
            let size = window_data.window.inner_size();
            size.width == 0 || size.height == 0
        } else {
            false
        }
    }
}
