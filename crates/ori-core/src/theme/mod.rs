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

/// Get the scale factor of the window.
pub fn scale_factor() -> f32 {
    SCALE_FACTOR.get()
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
