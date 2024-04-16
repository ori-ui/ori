use std::{
    ffi::{c_void, CString},
    num::NonZeroU32,
};

use glutin::{
    config::{Config, ConfigTemplate, ConfigTemplateBuilder},
    context::{
        ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext,
        PossiblyCurrentContext, PossiblyCurrentGlContext,
    },
    display::{Display, DisplayApiPreference, GlDisplay},
    error::{Error, ErrorKind},
    surface::{GlSurface as _, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

/// An error that can occur when creating a [`GlutinContext`].
pub type GlutinError = Error;

/// A context for rendering using [`glutin`].
pub struct GlutinContext {
    display: Display,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    width: u32,
    height: u32,
}

impl GlutinContext {
    fn find_first_config(
        display: &Display,
        template: ConfigTemplate,
    ) -> Result<Option<Config>, GlutinError> {
        let mut displays = unsafe { display.find_configs(template)? };
        Ok(displays.next())
    }

    fn find_config(display: &Display, samples: u8) -> Result<Option<Config>, GlutinError> {
        let fallback = ConfigTemplateBuilder::new();

        let transparent = (fallback.clone())
            .with_transparency(true)
            .with_multisampling(samples);

        if let Some(config) = Self::find_first_config(display, transparent.build())? {
            return Ok(Some(config));
        }

        Self::find_first_config(display, fallback.build())
    }

    /// Create a new context.
    pub fn new(
        window_handle: RawWindowHandle,
        display_handle: RawDisplayHandle,
        width: u32,
        height: u32,
        samples: u8,
    ) -> Result<Self, GlutinError> {
        #[allow(unused)]
        let mut api = DisplayApiPreference::Egl;

        #[cfg(target_os = "windows")]
        {
            api = DisplayApiPreference::Wgl(Some(window_handle));
        }

        let display = unsafe { Display::new(display_handle, api)? };
        let config = match Self::find_config(&display, samples)? {
            Some(config) => config,
            None => return Err(Error::from(ErrorKind::Misc)),
        };

        let non_zero_width = NonZeroU32::new(width).unwrap();
        let non_zero_height = NonZeroU32::new(height).unwrap();

        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new()
            .with_srgb(None)
            .build(window_handle, non_zero_width, non_zero_height);

        let surface = unsafe { display.create_window_surface(&config, &surface_attributes)? };

        #[cfg(not(target_os = "android"))]
        let context_api = ContextApi::OpenGl(None);

        #[cfg(target_os = "android")]
        let context_api = ContextApi::Gles(None);

        let context_attributes = ContextAttributesBuilder::new()
            .with_profile(GlProfile::Core)
            .with_context_api(context_api)
            .build(Some(window_handle));

        let context = unsafe { display.create_context(&config, &context_attributes)? };
        let context = context.treat_as_possibly_current();

        Ok(Self {
            display,
            surface,
            context,
            width,
            height,
        })
    }

    /// Get the address of a function.
    pub fn get_proc_address(&self, addr: &str) -> *const c_void {
        let cstring = CString::new(addr).unwrap();
        self.display.get_proc_address(&cstring)
    }

    /// Make the context current.
    pub fn make_current(&self) -> Result<(), GlutinError> {
        self.context.make_current(&self.surface)
    }

    /// Swap the buffers.
    pub fn swap_buffers(&self) -> Result<(), GlutinError> {
        self.surface.swap_buffers(&self.context)
    }

    /// Resize the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);

        self.surface.resize(&self.context, width, height);
    }
}
