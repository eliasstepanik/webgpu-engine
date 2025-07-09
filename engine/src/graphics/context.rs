//! WebGPU render context management
//!
//! Provides the main rendering context that manages the WebGPU device,
//! queue, and adapter for rendering operations. Surfaces are managed
//! separately by the WindowManager.

use std::sync::Arc;
use tracing::info;

/// Main rendering context for the engine
///
/// This struct owns the WebGPU resources needed for rendering:
/// - Device and queue for GPU operations
/// - Adapter for capabilities queries
///
/// Note: Surfaces are managed by WindowManager, not RenderContext
pub struct RenderContext {
    /// WebGPU instance
    pub instance: wgpu::Instance,
    /// WebGPU device for creating GPU resources
    pub device: Arc<wgpu::Device>,
    /// Command queue for submitting GPU work
    pub queue: Arc<wgpu::Queue>,
    /// WebGPU adapter for capability queries
    adapter: wgpu::Adapter,
    /// Adapter information for debugging
    pub adapter_info: wgpu::AdapterInfo,
}

impl RenderContext {
    /// Create a new render context
    ///
    /// This will request a device from the provided instance.
    /// Surface creation and configuration is handled separately.
    pub async fn new(
        instance: wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await?;

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
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Self {
            instance,
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter,
            adapter_info,
        })
    }

    /// Create initial surface configuration for a given surface
    pub fn create_surface_configuration(
        &self,
        surface: &wgpu::Surface,
        width: u32,
        height: u32,
    ) -> wgpu::SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(self.adapter());
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    /// Get the stored adapter for querying capabilities
    fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Get the preferred surface format for a given surface
    pub fn get_preferred_format(&self, surface: &wgpu::Surface) -> wgpu::TextureFormat {
        let surface_caps = surface.get_capabilities(self.adapter());
        surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0])
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_render_context_creation() {
        // Note: We can't actually create a RenderContext in tests without proper GPU setup
        // This is more of a compile-time check
        // Real testing would require integration tests with a GPU
    }
}
