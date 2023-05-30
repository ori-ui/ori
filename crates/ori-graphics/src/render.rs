use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use cosmic_text::FontSystem;

use crate::{Color, Frame, ImageData, ImageHandle};

pub trait RenderBackend {
    type Surface;
    type Renderer: Renderer;
    type Error: Debug;

    fn create_renderer(
        &self,
        surface: Self::Surface,
        width: u32,
        height: u32,
    ) -> Result<Self::Renderer, Self::Error>;
}

pub trait Renderer: Any {
    fn resize(&mut self, width: u32, height: u32);
    fn create_image(&self, data: &ImageData) -> ImageHandle;
    fn render_frame(&mut self, font_system: &mut FontSystem, frame: &Frame, clear_color: Color);
}

impl dyn Renderer {
    pub fn downcast_ref<T: Renderer>(&self) -> Option<&T> {
        if <dyn Renderer>::type_id(self) == TypeId::of::<T>() {
            // SAFETY: We just checked that the type is correct.
            unsafe { Some(&*(self as *const dyn Renderer as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Renderer>(&mut self) -> Option<&mut T> {
        if <dyn Renderer>::type_id(self) == TypeId::of::<T>() {
            // SAFETY: We just checked that the type is correct.
            unsafe { Some(&mut *(self as *mut dyn Renderer as *mut T)) }
        } else {
            None
        }
    }
}
