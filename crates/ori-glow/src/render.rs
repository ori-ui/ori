use std::fmt::Debug;

use glow::HasContext;

use ori_core::{
    canvas::{Color, Scene},
    layout::Size,
};

use super::{mesh::MeshRender, GlowError};

/// A renderer for a [`ori_core::canvas::Scene`].
#[derive(Debug)]
pub struct GlowRender {
    gl: glow::Context,
    width: u32,
    height: u32,
    mesh: MeshRender,
}

impl GlowRender {
    /// Create a new renderer.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        loader: impl FnMut(&str) -> *const std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<Self, GlowError> {
        let gl = unsafe { glow::Context::from_loader_function(loader) };

        let mesh = unsafe { MeshRender::new(&gl)? };

        Ok(Self {
            gl,
            width,
            height,
            mesh,
        })
    }

    /// Create a new renderer for WebGL.
    #[cfg(target_arch = "wasm32")]
    pub fn new_webgl(
        canvas: web_sys::HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, GlowError> {
        use web_sys::wasm_bindgen::JsCast;

        let webgl = canvas.get_context("webgl2").unwrap().unwrap();
        let context = webgl.dyn_into::<web_sys::WebGlRenderingContext>().unwrap();
        let gl = glow::Context::from_webgl1_context(context);

        let mesh = unsafe { MeshRender::new(&gl)? };

        Ok(Self {
            gl,
            width,
            height,
            mesh,
        })
    }

    unsafe fn resize(&mut self, physical_size: Size) {
        let width = physical_size.width as u32;
        let height = physical_size.height as u32;

        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

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

        Ok(())
    }

    /// Clean up unused resources.
    pub fn clean(&mut self) {
        unsafe {
            self.mesh.clean(&self.gl);
        }
    }

    /// Render the given [`ori_core::canvas::Scene`].
    ///
    /// Before calling this method, the context must be made current.
    /// And after calling this method, buffers must be swapped.
    pub fn render_scene(
        &mut self,
        scene: &Scene,
        clear: Color,
        logical_size: Size,
        physical_size: Size,
    ) -> Result<(), GlowError> {
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
