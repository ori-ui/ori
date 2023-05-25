use std::{ops::Deref, sync::Arc};

use ash::{extensions::khr, vk};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{AshDevice, AshError, AshInstance};

struct AshSurfaceInner {
    loader: khr::Surface,
    surface: vk::SurfaceKHR,
    display_handle: RawDisplayHandle,
    window_handle: RawWindowHandle,
    instance: AshInstance,
}

impl Drop for AshSurfaceInner {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.surface, None);
        }
    }
}

#[derive(Clone)]
pub struct AshSurface {
    inner: Arc<AshSurfaceInner>,
}

impl AshSurface {
    pub unsafe fn new(
        instance: &AshInstance,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<Self, AshError> {
        let surface = khr::Surface::new(instance.entry(), &instance.instance());
        let surface_khr = ash_window::create_surface(
            instance.entry(),
            &instance,
            display_handle,
            window_handle,
            None,
        )?;

        let inner = AshSurfaceInner {
            loader: surface,
            surface: surface_khr,
            display_handle,
            window_handle,
            instance: instance.clone(),
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn instance(&self) -> &AshInstance {
        &self.inner.instance
    }

    pub fn loader(&self) -> &khr::Surface {
        &self.inner.loader
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.inner.surface
    }

    pub fn display_handle(&self) -> &RawDisplayHandle {
        &self.inner.display_handle
    }

    pub fn window_handle(&self) -> &RawWindowHandle {
        &self.inner.window_handle
    }

    pub fn supported_formats(
        &self,
        device: &AshDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe { self.get_physical_device_surface_formats(device.physical(), self.surface()) }
    }

    pub fn capabilities(
        &self,
        device: &AshDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe { self.get_physical_device_surface_capabilities(device.physical(), self.surface()) }
    }

    pub fn present_modes(&self, device: &AshDevice) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
        unsafe { self.get_physical_device_surface_present_modes(device.physical(), self.surface()) }
    }
}

impl Deref for AshSurface {
    type Target = khr::Surface;

    fn deref(&self) -> &Self::Target {
        &self.inner.loader
    }
}
