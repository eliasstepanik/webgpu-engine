//! Direct imgui-sys integration for viewport rendering
//!
//! This module bypasses imgui-rs limitations by using imgui-sys directly

use imgui::{Context, Id, internal::RawWrapper};
use imgui_sys as sys;
use imgui_wgpu::Renderer;
use std::collections::HashMap;
use std::ptr;
use std::sync::Arc;
use tracing::{debug, info, warn};
use wgpu::*;

/// Viewport renderer using imgui-sys directly
pub struct ViewportSysRenderer {
    /// Base renderer for reference
    base_renderer: Renderer,
    /// Per-viewport renderers
    viewport_renderers: HashMap<Id, Renderer>,
    /// Device reference
    device: Arc<Device>,
    /// Queue reference
    queue: Arc<Queue>,
    /// Renderer config
    config: imgui_wgpu::RendererConfig<'static>,
}

impl ViewportSysRenderer {
    pub fn new(
        imgui: &mut Context,
        device: Arc<Device>,
        queue: Arc<Queue>,
        config: imgui_wgpu::RendererConfig,
    ) -> Self {
        let base_renderer = Renderer::new(imgui, &device, &queue, config.clone());
        
        Self {
            base_renderer,
            viewport_renderers: HashMap::new(),
            device,
            queue,
            config,
        }
    }
    
    /// Check if viewports are enabled
    pub fn viewports_enabled(&self, imgui: &Context) -> bool {
        unsafe {
            let io = sys::igGetIO();
            ((*io).ConfigFlags & sys::ImGuiConfigFlags_ViewportsEnable as i32) != 0
        }
    }
    
    /// Get viewport count
    pub fn viewport_count(&self) -> usize {
        unsafe {
            let platform_io = sys::igGetPlatformIO();
            if platform_io.is_null() {
                return 1; // Only main viewport
            }
            
            (*platform_io).Viewports.Size as usize
        }
    }
    
    /// Render all viewports
    pub fn render_all_viewports(
        &mut self,
        imgui: &mut Context,
        main_surface: &Surface,
        main_clear_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // First render main viewport using imgui-rs
        self.render_main_viewport(imgui, main_surface, main_clear_color)?;
        
        // Then render additional viewports using imgui-sys
        if self.viewports_enabled(imgui) {
            self.render_platform_viewports(imgui)?;
        }
        
        Ok(())
    }
    
    /// Render the main viewport
    fn render_main_viewport(
        &mut self,
        imgui: &mut Context,
        surface: &Surface,
        clear_color: Color,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This uses the normal imgui-rs path
        let draw_data = imgui.render();
        
        let surface_texture = surface.get_current_texture()?;
        let view = surface_texture.texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Main Viewport Encoder"),
        });
        
        self.base_renderer.render(
            draw_data,
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            Some(clear_color),
        )?;
        
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
        
        Ok(())
    }
    
    /// Render additional platform viewports using imgui-sys
    fn render_platform_viewports(
        &mut self,
        imgui: &mut Context,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let platform_io = sys::igGetPlatformIO();
            if platform_io.is_null() {
                return Ok(());
            }
            
            let viewports = &(*platform_io).Viewports;
            
            // Iterate through viewports (skip main viewport at index 0)
            for i in 1..viewports.Size {
                let viewport_ptr = *viewports.Data.offset(i as isize);
                if viewport_ptr.is_null() {
                    continue;
                }
                
                let viewport = &*viewport_ptr;
                let viewport_id = Id::from_ptr(viewport_ptr as *const std::ffi::c_void);
                
                // Check if viewport has draw data
                if viewport.DrawData.is_null() {
                    continue;
                }
                
                // Get or create renderer for this viewport
                if !self.viewport_renderers.contains_key(&viewport_id) {
                    // Create new renderer for this viewport
                    let renderer = Renderer::new(imgui, &self.device, &self.queue, self.config.clone());
                    self.viewport_renderers.insert(viewport_id, renderer);
                    info!("Created renderer for viewport {:?}", viewport_id);
                }
                
                // Get the renderer
                let renderer = self.viewport_renderers.get_mut(&viewport_id).unwrap();
                
                // Here we would need to:
                // 1. Get the viewport's surface (from platform backend)
                // 2. Convert sys DrawData to imgui-rs DrawData
                // 3. Render using the viewport's renderer
                
                // The challenge is that we need access to the viewport's Surface
                // which requires integration with the ViewportBackend
                
                warn!("Viewport {:?} rendering not yet implemented - need surface access", viewport_id);
            }
        }
        
        Ok(())
    }
    
    /// Get draw data for a specific viewport (using imgui-sys)
    pub unsafe fn get_viewport_draw_data(viewport_ptr: *mut sys::ImGuiViewport) -> Option<*mut sys::ImDrawData> {
        if viewport_ptr.is_null() {
            return None;
        }
        
        let viewport = &*viewport_ptr;
        if viewport.DrawData.is_null() {
            None
        } else {
            Some(viewport.DrawData)
        }
    }
}

/// Convert imgui-sys DrawData to a format we can use
/// This is complex because imgui-rs DrawData is not directly constructible
pub unsafe fn convert_draw_data(sys_draw_data: *mut sys::ImDrawData) -> Result<(), Box<dyn std::error::Error>> {
    if sys_draw_data.is_null() {
        return Ok(());
    }
    
    let draw_data = &*sys_draw_data;
    
    // Access draw data fields
    let total_vtx_count = draw_data.TotalVtxCount;
    let total_idx_count = draw_data.TotalIdxCount;
    let cmd_lists_count = draw_data.CmdListsCount;
    
    debug!("Viewport draw data: {} vertices, {} indices, {} cmd lists", 
           total_vtx_count, total_idx_count, cmd_lists_count);
    
    // To properly render this, we would need to:
    // 1. Iterate through command lists
    // 2. Extract vertex and index data
    // 3. Process draw commands
    // 4. Submit to GPU
    
    // This requires deeper integration with imgui-wgpu internals
    
    Ok(())
}

/// The core issue remains: we need a way to associate viewport IDs with wgpu Surfaces
/// This requires tight integration between:
/// 1. ViewportBackend (creates windows)
/// 2. WindowManager (manages surfaces)
/// 3. ViewportSysRenderer (renders to surfaces)
///
/// A complete solution would require modifying the architecture to pass Surface
/// references through the viewport creation pipeline.