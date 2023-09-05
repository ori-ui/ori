use std::sync::Arc;

use wgpu::{
    Adapter, Device, DeviceDescriptor, Features, Instance, PowerPreference, Queue,
    RequestAdapterOptions, Surface,
};

use crate::RenderError;

#[derive(Debug)]
pub struct WgpuRenderInstance {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl WgpuRenderInstance {
    /// # Safety
    /// - See the `Safety` section on [`wgpu::Instance::create_surface`].
    pub async unsafe fn new(
        window: &winit::window::Window,
    ) -> Result<(Self, Surface), RenderError> {
        let instance = Instance::default();

        let surface = instance.create_surface(window)?;

        let options = RequestAdapterOptions {
            power_preference: PowerPreference::None,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        let adapter = instance.request_adapter(&options).await;
        let adapter = adapter.ok_or(RenderError::AdapterNotFound)?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("ori-device"),
                    features: Features::empty(),
                    #[cfg(target_os = "android")]
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    ..Default::default()
                },
                None,
            )
            .await?;

        let instance = Self {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
        };

        Ok((instance, surface))
    }

    /// Create a new surface from the given window.
    ///
    /// # Safety
    /// - See the `Safety` section on [`wgpu::Instance::create_surface`].
    #[allow(dead_code)]
    pub unsafe fn create_surface(
        &self,
        window: &winit::window::Window,
    ) -> Result<Surface, RenderError> {
        Ok(self.instance.create_surface(window)?)
    }
}
