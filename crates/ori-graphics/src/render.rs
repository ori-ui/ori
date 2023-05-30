use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use cosmic_text::FontSystem;

use crate::{Color, Frame, ImageData, ImageHandle};

/// A render backend.
///
/// This trait is used to create a [`Renderer`] from a surface, width and height.
pub trait RenderBackend {
    /// The type of the surface required to create a [`Renderer`].
    type Surface;
    /// The type of the [`Renderer`] created by this backend.
    type Renderer: Renderer;
    /// The type of the error returned by this backend.
    type Error: Debug;

    /// Create a [`Renderer`] from a surface, width and height.
    fn create_renderer(
        &self,
        surface: Self::Surface,
        width: u32,
        height: u32,
    ) -> Result<Self::Renderer, Self::Error>;
}

/// A renderer, used to render frames. See [`RenderBackend::create_renderer`].
pub trait Renderer: Any {
    /// This is called when the window is resized.
    fn resize(&mut self, width: u32, height: u32);
    /// Create an image from the given data.
    fn create_image(&self, data: &ImageData) -> ImageHandle;
    /// Render a frame.
    fn render_frame(&mut self, font_system: &mut FontSystem, frame: &Frame, clear_color: Color);
}

impl dyn Renderer {
    /// Try to downcast this renderer to a specific type.
    pub fn downcast_ref<T: Renderer>(&self) -> Option<&T> {
        if <dyn Renderer>::type_id(self) == TypeId::of::<T>() {
            // SAFETY: We just checked that the type is correct.
            unsafe { Some(&*(self as *const dyn Renderer as *const T)) }
        } else {
            None
        }
    }

    /// Try to downcast this renderer to a specific type.
    pub fn downcast_mut<T: Renderer>(&mut self) -> Option<&mut T> {
        if <dyn Renderer>::type_id(self) == TypeId::of::<T>() {
            // SAFETY: We just checked that the type is correct.
            unsafe { Some(&mut *(self as *mut dyn Renderer as *mut T)) }
        } else {
            None
        }
    }
}
