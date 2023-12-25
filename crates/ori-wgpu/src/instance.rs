use std::sync::Arc;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::{
    Adapter, Device, DeviceDescriptor, Features, Instance, PowerPreference, Queue,
    RequestAdapterOptions, Surface,
};

use super::WgpuError;

/// A context containing the [`wgpu::Instance`], [`wgpu::Adapter`], [`wgpu::Device`], and [`wgpu::Queue`].
#[derive(Debug)]
pub struct WgpuRenderInstance {
    /// The [`wgpu::Instance`] used for rendering.
    pub instance: Instance,
    /// The [`wgpu::Adapter`] used for rendering.
    pub adapter: Adapter,
    /// The [`wgpu::Device`] used for rendering.
    pub device: Arc<Device>,
    /// The [`wgpu::Queue`] used for rendering.
    pub queue: Arc<Queue>,
}

impl WgpuRenderInstance {
    /// # Safety
    /// - See the `Safety` section on [`wgpu::Instance::create_surface`].
    pub async unsafe fn new_async(
        window: &(impl HasRawWindowHandle + HasRawDisplayHandle),
    ) -> Result<(Self, Surface), WgpuError> {
        let instance = Instance::default();

        let surface = instance.create_surface(window)?;

        let options = RequestAdapterOptions {
            power_preference: PowerPreference::None,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        let adapter = instance.request_adapter(&options).await;
        let adapter = adapter.ok_or(WgpuError::AdapterNotFound)?;

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
    pub unsafe fn new(
        window: &(impl HasRawWindowHandle + HasRawDisplayHandle),
    ) -> Result<(Self, Surface), WgpuError> {
        pollster::block_on(Self::new_async(window))
    }

    /// Create a new surface from the given window.
    ///
    /// # Safety
    /// - See the `Safety` section on [`wgpu::Instance::create_surface`].
    #[allow(dead_code)]
    pub unsafe fn create_surface(
        &self,
        window: &(impl HasRawWindowHandle + HasRawDisplayHandle),
    ) -> Result<Surface, WgpuError> {
        Ok(self.instance.create_surface(window)?)
    }
}
