//! Graphics library for Ori.
//!
//! This crate provides all the necessary types and traits for rendering graphics.
//! - [`Color`]
//! - [`Curve`]
//! - [`PrimitiveKind`], [`Primitive`] and [`Frame`]
//! - [`ImageSource`], [`ImageData`] and [`ImageHandle`]
//! - [`Mesh`] and [`Vertex`]

mod color;
mod curve;
mod frame;
mod image;
mod mesh;
mod quad;
mod rect;
mod render;
mod text;

pub use self::image::*;
pub use color::*;
pub use curve::*;
pub use frame::*;
pub use mesh::*;
pub use quad::*;
pub use rect::*;
pub use render::*;
pub use text::*;

pub use cosmic_text;
pub use glam as math;

pub mod prelude {
    //! A collection of commonly used types and traits.

    pub use crate::color::Color;
    pub use crate::curve::Curve;
    pub use crate::image::{ImageData, ImageHandle, ImageLoadError, ImageSource};
    pub use crate::mesh::{Mesh, Vertex};
    pub use crate::quad::Quad;
    pub use crate::rect::Rect;
    pub use crate::text::TextAlign;

    pub use glam::*;
}
