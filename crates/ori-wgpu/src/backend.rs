use std::sync::Arc;

use ori_graphics::RenderBackend;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use wgpu::{Adapter, Device, Instance, Queue, RequestAdapterOptions, RequestDeviceError, Surface};

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

#[derive(Debug)]
pub enum WgpuBackendError {
    NoAdapter,
    RequestDevice(RequestDeviceError),
    IncompatibleSurface,
}

struct WgpuBackendState {
    adapter: Adapter,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

#[derive(Default)]
pub struct WgpuBackend {
    instance: Instance,
    state: Option<WgpuBackendState>,
}

impl WgpuBackend {
    async fn crate_state_async(
        &self,
        surface: &Surface,
    ) -> Result<WgpuBackendState, WgpuBackendError> {
        let adapter = self
            .instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        Ok(WgpuBackendState {
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
        })
    }

    fn crate_state(&self, surface: &Surface) -> Result<WgpuBackendState, WgpuBackendError> {
        pollster::block_on(self.crate_state_async(surface))
    }

    fn state(&mut self, surface: &Surface) -> Result<&WgpuBackendState, WgpuBackendError> {
        if let Some(ref state) = self.state {
            return Ok(state);
        }

        self.state = Some(self.crate_state(surface)?);
        Ok(self.state.as_ref().unwrap())
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderBackend for WgpuBackend {
    type Surface = (RawDisplayHandle, RawWindowHandle);
    type Renderer = WgpuRenderer;
    type Error = WgpuBackendError;

    fn create_renderer(
        &mut self,
        (display, surface): Self::Surface,
        width: u32,
        height: u32,
    ) -> Result<Self::Renderer, Self::Error> {
        let wgpu_surface = WgpuSurface(display, surface);
        let surface = unsafe { self.instance.create_surface(&wgpu_surface).unwrap() };
        let state = self.state(&surface)?;

        if !state.adapter.is_surface_supported(&surface) {
            return Err(WgpuBackendError::IncompatibleSurface);
        }

        Ok(WgpuRenderer::new(
            &state.adapter,
            state.device.clone(),
            state.queue.clone(),
            surface,
            width,
            height,
        ))
    }
}
