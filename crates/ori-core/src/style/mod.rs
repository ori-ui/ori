//! Styleing and theming.

pub mod builtin;
mod key;
mod palette;
mod theme;

pub use builtin::*;
pub use key::*;
pub use palette::*;
pub use theme::*;

pub(crate) const SCALE_FACTOR: Key<f32> = Key::new("window.scale_factor");

/// Get a size in pixels, relative to the [`scale factor`].
///
/// [`scale factor`]: crate::window::Window::scale_factor
pub fn pt(size: f32) -> f32 {
    size * SCALE_FACTOR.get()
}

/// Get a size in pixels, relative to the default font size.
///
/// This is a shorthand for `pt(size) * 16.0`.
pub fn em(size: f32) -> f32 {
    pt(size) * 16.0
}
