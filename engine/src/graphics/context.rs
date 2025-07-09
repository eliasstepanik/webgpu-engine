//! WebGPU render context management
//!
//! Provides the main rendering context that manages the WebGPU device,
//! queue, surface, and configuration for rendering operations.

use std::sync::{Arc, Mutex};
use tracing::info;

/// Main rendering context for the engine
///
/// This struct owns the WebGPU resources needed for rendering:
/// - Device and queue for GPU operations
/// - Surface for presenting to the window
/// - Configuration for the surface
pub struct RenderContext<'window> {
    /// WebGPU device for creating GPU resources
    pub device: Arc<wgpu::Device>,
    /// Command queue for submitting GPU work
    pub queue: Arc<wgpu::Queue>,
    /// Surface for presenting rendered frames
    pub surface: Mutex<wgpu::Surface<'window>>,
    /// Current surface configuration
    pub surface_config: Mutex<wgpu::SurfaceConfiguration>,
    /// Adapter information for debugging
    pub adapter_info: wgpu::AdapterInfo,
}

impl<'window> RenderContext<'window> {
    /// Create a new render context from a window
    ///
    /// This will initialize WebGPU and configure the surface for rendering.
    pub async fn new(
        window: &'window winit::window::Window,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();

        // Create the WebGPU instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from window
        let surface = instance.create_surface(window)?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| "Failed to find suitable GPU adapter")?;

        let adapter_info = adapter.get_info();
        info!(
            gpu_name = %adapter_info.name,
            backend = ?adapter_info.backend,
            "GPU adapter selected"
        );

        // Request device and queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Render Device"),
                memory_hints: Default::default(),
                trace: Default::default(),
            })
            .await?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            surface: Mutex::new(surface),
            surface_config: Mutex::new(surface_config),
            adapter_info,
        })
    }

    /// Resize the surface when the window size changes
    pub fn resize(&self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let mut surface_config = self.surface_config.lock().unwrap();
            surface_config.width = new_size.width;
            surface_config.height = new_size.height;

            let surface = self.surface.lock().unwrap();
            surface.configure(&self.device, &surface_config);
        }
    }

    /// Get the current surface texture for rendering
    ///
    /// This should be called at the beginning of each frame.
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        let surface = self.surface.lock().unwrap();
        surface.get_current_texture()
    }

    /// Create a command encoder for recording GPU commands
    pub fn create_command_encoder(&self, label: Option<&str>) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label })
    }

    /// Submit command buffers to the GPU queue
    pub fn submit<I: IntoIterator<Item = wgpu::CommandBuffer>>(&self, command_buffers: I) {
        self.queue.submit(command_buffers);
    }

    /// Get the current surface configuration
    pub fn surface_config(&self) -> wgpu::SurfaceConfiguration {
        self.surface_config.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_render_context_creation() {
        // Note: We can't actually create a RenderContext in tests without a window
        // This is more of a compile-time check
        // Real testing would require integration tests with a window
    }
}
