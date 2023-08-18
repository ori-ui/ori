#![allow(clippy::module_inception)]
#![warn(missing_docs)]

//! Core library for the Ori UI framework.

mod any_view;
mod canvas;
mod color;
mod content;
mod contexts;
mod delegate;
mod event;
mod image;
mod layout;
mod rebuild;
mod style;
mod text;
mod transition;
mod ui;
mod view;
mod window;

pub use any_view::*;
pub use canvas::*;
pub use color::*;
pub use content::*;
pub use contexts::*;
pub use delegate::*;
pub use event::*;
pub use image::*;
pub use layout::*;
pub use rebuild::*;
pub use style::*;
pub use text::*;
pub use transition::*;
pub use ui::*;
pub use view::*;
pub use window::*;

pub use tracing;
pub mod views;

pub mod math {
    //! Math types and functions, powered by [`glam`].

    pub use glam::*;
}
