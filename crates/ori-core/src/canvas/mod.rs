//! Canvas module.
//!
//! This module contains the [`Canvas`] type, which is used to draw primitives
//! to a [`Scene`].

mod background;
mod border;
mod canvas;
mod color;
mod curve;
mod mesh;
mod primitive;
mod render;
mod scene;
mod shadow;
mod stroke;

pub use background::*;
pub use border::*;
pub use canvas::*;
pub use color::*;
pub use curve::*;
pub use mesh::*;
pub use primitive::*;
pub use render::*;
pub use scene::*;
pub use shadow::*;
pub use stroke::*;
