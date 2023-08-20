//! Styleing and theming.

pub mod builtin;
mod key;
mod palette;
mod theme;

pub use builtin::*;
pub use key::*;
pub use palette::*;
pub use theme::*;

/// The text size in pixels.
pub const TEXT_SIZE: Key<f32> = Key::new("text.size");

/// Get a size in pixels, relative to the text size.
pub fn em(size: f32) -> f32 {
    size * TEXT_SIZE.get()
}

/// Set the text size in pixels.
pub fn set_text_size(size: f32) {
    Theme::global(|theme| theme.set(TEXT_SIZE, size));
}
