use std::num::NonZeroU32;

use glow::HasContext;
use glutin::{
    config::{Config, ConfigTemplate, ConfigTemplateBuilder},
    context::{
        ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext,
        PossiblyCurrentGlContext, Version,
    },
    display::{Display, DisplayApiPreference, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use ori_core::{
    canvas::{Color, Scene},
    layout::Size,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::Window;

use super::{mesh::MeshRender, GlowError};

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
        let transparent = ConfigTemplateBuilder::new()
            .with_transparency(true)
            .with_multisampling(samples);

        if let Ok(config) = Self::find_first_config(display, transparent.build()) {
            return Ok(config);
        }

        Self::find_first_config(display, Default::default())
    }

    pub fn new(window: &Window, samples: u8) -> Result<Self, GlowError> {
        let display_handle = window.raw_display_handle();
        let display = unsafe { Display::new(display_handle, DisplayApiPreference::Egl)? };

        let config = Self::find_config(&display, samples)?;

        let size = window.inner_size();
        let width = NonZeroU32::new(size.width).unwrap();
        let height = NonZeroU32::new(size.height).unwrap();

        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window.raw_window_handle(),
            width,
            height,
        );

        let surface = unsafe { display.create_window_surface(&config, &surface_attributes)? };

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(window.raw_window_handle()));

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
            width: size.width,
            height: size.height,

            mesh,
        })
    }

    fn make_current(&self) -> Result<(), GlowError> {
        self.context.make_current(&self.surface)?;
        Ok(())
    }

    unsafe fn resize(&mut self, width: u32, height: u32) {
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

    unsafe fn render(&mut self, scene: &Scene, clear: Color) -> Result<(), GlowError> {
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
            let size = Size::new(self.width as f32, self.height as f32);
            self.mesh.render_batch(&self.gl, batch, size)?;
        }

        self.surface.swap_buffers(&self.context)?;

        Ok(())
    }

    pub fn render_scene(
        &mut self,
        scene: &Scene,
        clear: Color,
        width: u32,
        height: u32,
    ) -> Result<(), GlowError> {
        self.make_current()?;

        unsafe {
            self.resize(width, height);
            self.render(scene, clear)
        }?;

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
