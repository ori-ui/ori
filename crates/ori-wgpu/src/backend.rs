use ori_graphics::RenderBackend;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use crate::WgpuRenderer;

struct WgpuSurface(RawDisplayHandle, RawWindowHandle);

unsafe impl HasRawDisplayHandle for WgpuSurface {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.0
    }
}

unsafe impl HasRawWindowHandle for WgpuSurface {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.1
    }
}

pub struct WgpuBackend {}

impl Default for WgpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl WgpuBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderBackend for WgpuBackend {
    type Surface = (RawDisplayHandle, RawWindowHandle);
    type Renderer = WgpuRenderer;
    type Error = ();

    fn create_renderer(
        &self,
        (display, surface): Self::Surface,
        width: u32,
        height: u32,
    ) -> Result<Self::Renderer, Self::Error> {
        let surface = WgpuSurface(display, surface);
        let renderer = unsafe { WgpuRenderer::new(&surface, width, height) };
        Ok(renderer)
    }
}
