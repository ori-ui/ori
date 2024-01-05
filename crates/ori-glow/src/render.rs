use std::num::NonZeroU32;

use glow::HasContext;
use glutin::{
    config::{Config, ConfigTemplate, ConfigTemplateBuilder},
    context::{
        ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentGlContext,
        PossiblyCurrentContext, PossiblyCurrentGlContext,
    },
    display::{Display, DisplayApiPreference, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use ori_core::{
    canvas::{Color, Scene},
    layout::Size,
};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::{mesh::MeshRender, GlowError};

/// A renderer for a [`ori_core::canvas::Scene`].
#[derive(Debug)]
pub struct GlowRender {
    #[allow(dead_code)]
    display: Display,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    gl: glow::Context,
    width: u32,
    height: u32,
    mesh: MeshRender,
}

impl GlowRender {
    fn find_first_config(display: &Display, template: ConfigTemplate) -> Result<Config, GlowError> {
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

    /// Create a new renderer.
    pub fn new(
        window_handle: RawWindowHandle,
        display_handle: RawDisplayHandle,
        width: u32,
        height: u32,
        samples: u8,
    ) -> Result<Self, GlowError> {
        let display = unsafe { Display::new(display_handle, DisplayApiPreference::Egl)? };

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
        let context = context.make_current(&surface)?;

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s) as *const _)
        };

        let mesh = unsafe { MeshRender::new(&gl)? };

        Ok(Self {
            display,
            surface,
            context,
            gl,
            width,
            height,
            mesh,
        })
    }

    fn make_current(&self) -> Result<(), GlowError> {
        Ok(self.context.make_current(&self.surface)?)
    }

    unsafe fn resize(&mut self, physical_size: Size) {
        let width = physical_size.width as u32;
        let height = physical_size.height as u32;

        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        let non_zero_width = NonZeroU32::new(width).unwrap();
        let non_zero_height = NonZeroU32::new(height).unwrap();
        (self.surface).resize(&self.context, non_zero_width, non_zero_height);
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

        self.gl.enable(glow::BLEND);
        self.gl.blend_equation(glow::FUNC_ADD);
        (self.gl).blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        self.gl.disable(glow::DEPTH_TEST);
        self.gl.disable(glow::CULL_FACE);
        self.gl.disable(glow::FRAMEBUFFER_SRGB);

        // msaa
        self.gl.enable(glow::MULTISAMPLE);

        for batch in batches.iter() {
            self.mesh.render_batch(&self.gl, batch, logical_size)?;
        }

        self.surface.swap_buffers(&self.context)?;

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
