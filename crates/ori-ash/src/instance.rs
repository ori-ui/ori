use std::{ops::Deref, sync::Arc};

use ash::vk;
use raw_window_handle::HasRawDisplayHandle;

use crate::AshError;

struct AshInstanceInner {
    #[cfg(not(target_os = "macos"))]
    entry: ash::Entry,
    #[cfg(target_os = "macos")]
    entry: ash_molten::Entry,

    owned: bool,

    instance: ash::Instance,
}

impl Drop for AshInstanceInner {
    fn drop(&mut self) {
        if self.owned {
            unsafe { self.instance.destroy_instance(None) };
        }
    }
}

#[derive(Clone)]
pub struct AshInstance {
    inner: Arc<AshInstanceInner>,
}

impl AshInstance {
    pub unsafe fn external(
        #[cfg(not(target_os = "macos"))] entry: ash::Entry,
        #[cfg(target_os = "macos")] entry: ash_molten::Entry,
        instance: ash::Instance,
    ) -> Self {
        let inner = AshInstanceInner {
            entry,
            owned: false,
            instance,
        };

        Self {
            inner: Arc::new(inner),
        }
    }

    pub unsafe fn new(display: &impl HasRawDisplayHandle) -> Result<Self, AshError> {
        #[cfg(not(target_os = "macos"))]
        let entry = ash::Entry::load()?;
        #[cfg(target_os = "macos")]
        let entry = ash_molten::Entry::load()?;

        let application_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let display_handle = display.raw_display_handle();
        let extensions = ash_window::enumerate_required_extensions(display_handle)?;
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr() as *const *const i8,
            ..Default::default()
        };

        let instance = entry.create_instance(&create_info, None)?;

        let inner = AshInstanceInner {
            entry,
            owned: true,
            instance,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[cfg(not(target_os = "macos"))]
    pub fn entry(&self) -> &ash::Entry {
        &self.inner.entry
    }

    #[cfg(target_os = "macos")]
    pub fn entry(&self) -> &ash_molten::Entry {
        &self.inner.entry
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.inner.instance
    }
}

impl Deref for AshInstance {
    type Target = ash::Instance;

    fn deref(&self) -> &Self::Target {
        &self.inner.instance
    }
}
