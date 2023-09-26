//! Styleing and theming.

mod builder;
pub mod builtin;
mod key;
mod palette;
mod theme;

pub use builder::*;
pub use builtin::*;
pub use key::*;
pub use palette::*;
pub use theme::*;

use crate::layout::Size;

pub(crate) const SCALE_FACTOR: Key<f32> = Key::new("window.scale_factor");
pub(crate) const WINDOW_SIZE: Key<Size> = Key::new("window.size");

/// Get a size in pixels, relative to the [`scale factor`].
///
/// [`scale factor`]: crate::window::Window::scale_factor
pub fn pt(size: f32) -> f32 {
    size * SCALE_FACTOR.get()
}

/// Get a size in pixels, relative to the default font size.
///
/// This is a shorthand for `pt(size) * 16.0`.
pub fn rem(size: f32) -> f32 {
    pt(size) * 16.0
}

/// Get the window size in physical pixels.
pub fn window_size() -> Size {
    WINDOW_SIZE.get()
}

/// Get a size in pixels, relative to the window width.
pub fn vw(size: f32) -> f32 {
    size * window_size().width
}

/// Get a size in pixels, relative to the window height.
pub fn vh(size: f32) -> f32 {
    size * window_size().height
}
