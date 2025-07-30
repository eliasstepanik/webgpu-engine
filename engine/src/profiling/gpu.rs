use tracing::{info, warn};
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};

pub struct GpuProfilerWrapper {
    profiler: Option<GpuProfiler>,
}

impl GpuProfilerWrapper {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        let features = device.features();
        if features.contains(wgpu::Features::TIMESTAMP_QUERY) {
            match GpuProfiler::new(device, GpuProfilerSettings::default()) {
                Ok(profiler) => {
                    info!("GPU profiling enabled with Tracy");
                    return Self {
                        profiler: Some(profiler),
                    };
                }
                Err(e) => warn!("Failed to create GPU profiler: {}", e),
            }
        } else {
            warn!("GPU timestamp queries not supported on this device");
        }

        Self { profiler: None }
    }

    pub fn scope<'a>(
        &'a self,
        label: &str,
        encoder: &'a mut wgpu::CommandEncoder,
        _device: &wgpu::Device,
    ) -> Option<wgpu_profiler::Scope<'a, wgpu::CommandEncoder>> {
        if let Some(ref profiler) = self.profiler {
            Some(profiler.scope(label, encoder))
        } else {
            None
        }
    }

    pub fn resolve_queries(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(ref mut profiler) = self.profiler {
            profiler.resolve_queries(encoder);
        }
    }

    pub fn end_frame(&mut self) -> Result<(), wgpu_profiler::EndFrameError> {
        if let Some(ref mut profiler) = self.profiler {
            profiler.end_frame()?;
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.profiler.is_some()
    }
}
