use std::{ffi::c_void, fmt::Debug};

use glow::HasContext;

use ori_core::{
    canvas::{Color, Scene},
    layout::Size,
};

use super::{mesh::MeshRender, GlowError};

/// A trait for a OpenGL context.
pub trait GlSurface {
    /// Make the context current.
    fn make_current(&self) -> Result<(), GlowError>;

    /// Get the address of a function.
    fn get_proc_address(&self, addr: &str) -> *const c_void;

    /// Resize the surface.
    fn resize(&self, width: u32, height: u32);

    /// Swap the buffers.
    fn swap_buffers(&self) -> Result<(), GlowError>;
}

impl Debug for dyn GlSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GlContext")
    }
}

#[cfg(feature = "glutin")]
mod glutin {
    use std::{ffi::CString, num::NonZeroU32};

    use glutin::{
        config::{Config, ConfigTemplate, ConfigTemplateBuilder},
        context::{
            ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext,
            PossiblyCurrentContext, PossiblyCurrentGlContext,
        },
        display::{Display, DisplayApiPreference, GlDisplay},
        surface::{GlSurface as _, Surface, SurfaceAttributesBuilder, WindowSurface},
    };
    use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

    use crate::{GlSurface, GlowError};

    pub struct GlutinContext {
        #[allow(dead_code)]
        display: Display,
        surface: Surface<WindowSurface>,
        context: PossiblyCurrentContext,
    }

    impl GlutinContext {
        fn find_first_config(
            display: &Display,
            template: ConfigTemplate,
        ) -> Result<Config, GlowError> {
            let mut displays = unsafe { display.find_configs(template)? };
            displays.next().ok_or(GlowError::ConfigNotFound)
        }

        fn find_config(display: &Display, samples: u8) -> Result<Config, GlowError> {
            let fallback = ConfigTemplateBuilder::new();

            let transparent = (fallback.clone())
                .with_transparency(true)
                .with_multisampling(samples);

            if let Ok(config) = Self::find_first_config(display, transparent.build()) {
                return Ok(config);
            }

            Self::find_first_config(display, fallback.build())
        }

        pub fn new(
            window_handle: RawWindowHandle,
            display_handle: RawDisplayHandle,
            width: u32,
            height: u32,
            samples: u8,
        ) -> Result<Self, GlowError> {
            #[allow(unused)]
            let mut api = DisplayApiPreference::Egl;

            #[cfg(target_os = "windows")]
            {
                api = DisplayApiPreference::Wgl(Some(window_handle));
            }

            let display = unsafe { Display::new(display_handle, api)? };
            let config = Self::find_config(&display, samples)?;

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
            })
        }
    }

    impl GlSurface for GlutinContext {
        fn make_current(&self) -> Result<(), GlowError> {
            self.context.make_current(&self.surface)?;
            Ok(())
        }

        fn get_proc_address(&self, addr: &str) -> *const std::ffi::c_void {
            let cstr = CString::new(addr).unwrap();
            self.display.get_proc_address(&cstr)
        }

        fn resize(&self, width: u32, height: u32) {
            let non_zero_width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
            let non_zero_height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);

            (self.surface).resize(&self.context, non_zero_width, non_zero_height);
        }

        fn swap_buffers(&self) -> Result<(), GlowError> {
            self.surface.swap_buffers(&self.context)?;
            Ok(())
        }
    }
}

/// A renderer for a [`ori_core::canvas::Scene`].
#[derive(Debug)]
pub struct GlowRender {
    surface: Box<dyn GlSurface>,
    gl: glow::Context,
    width: u32,
    height: u32,
    mesh: MeshRender,
}

impl GlowRender {
    /// Create a new renderer.
    pub fn new(
        context: impl GlSurface + 'static,
        width: u32,
        height: u32,
    ) -> Result<Self, GlowError> {
        let gl = unsafe { glow::Context::from_loader_function(|s| context.get_proc_address(s)) };

        let mesh = unsafe { MeshRender::new(&gl)? };

        Ok(Self {
            surface: Box::new(context),
            gl,
            width,
            height,
            mesh,
        })
    }

    /// Create a new renderer using [`glutin`].
    #[cfg(feature = "glutin")]
    pub fn glutin(
        window_handle: raw_window_handle::RawWindowHandle,
        display_handle: raw_window_handle::RawDisplayHandle,
        width: u32,
        height: u32,
        samples: u8,
    ) -> Result<Self, GlowError> {
        let context =
            glutin::GlutinContext::new(window_handle, display_handle, width, height, samples)?;

        Self::new(context, width, height)
    }

    fn make_current(&self) -> Result<(), GlowError> {
        self.surface.make_current()
    }

    unsafe fn resize(&mut self, physical_size: Size) {
        let width = physical_size.width as u32;
        let height = physical_size.height as u32;

        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        self.surface.resize(width, height);
        self.gl.viewport(0, 0, width as i32, height as i32);
    }

    unsafe fn render(
        &mut self,
        scene: &Scene,
        clear: Color,
        logical_size: Size,
    ) -> Result<(), GlowError> {
        let batches = scene.batches();

        self.gl.clear_color(clear.r, clear.g, clear.b, clear.a);
        self.gl.clear(glow::COLOR_BUFFER_BIT);

        // enable alpha blending
        self.gl.enable(glow::BLEND);
        self.gl.blend_equation(glow::FUNC_ADD);
        self.gl.blend_func_separate(
            glow::SRC_ALPHA,
            glow::ONE_MINUS_SRC_ALPHA,
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
        );

        self.gl.enable(glow::SCISSOR_TEST);

        self.gl.disable(glow::DEPTH_TEST);
        self.gl.disable(glow::CULL_FACE);
        self.gl.disable(glow::FRAMEBUFFER_SRGB);

        // msaa
        self.gl.enable(glow::MULTISAMPLE);

        for batch in batches.iter() {
            self.mesh.render_batch(&self.gl, batch, logical_size)?;
        }

        self.gl.disable(glow::SCISSOR_TEST);

        self.surface.swap_buffers()?;

        Ok(())
    }

    /// Clean up unused resources.
    pub fn clean(&mut self) {
        unsafe {
            self.mesh.clean(&self.gl);
        }
    }

    /// Render the given [`ori_core::canvas::Scene`].
    pub fn render_scene(
        &mut self,
        scene: &Scene,
        clear: Color,
        logical_size: Size,
        physical_size: Size,
    ) -> Result<(), GlowError> {
        self.make_current()?;

        unsafe {
            self.resize(physical_size);
            self.render(scene, clear, logical_size)?;
        };

        Ok(())
    }
}

impl Drop for GlowRender {
    fn drop(&mut self) {
        unsafe {
            self.mesh.delete(&self.gl);
        }
    }
}
