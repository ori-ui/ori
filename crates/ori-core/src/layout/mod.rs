//! Layout of [`View`](crate::view::View)s.

mod affine;
mod alignment;
mod axis;
mod justify;
mod matrix;
mod padding;
mod point;
mod rect;
mod size;
mod space;
mod vector;

pub use affine::*;
pub use alignment::*;
pub use axis::*;
pub use justify::*;
pub use matrix::*;
pub use padding::*;
pub use point::*;
pub use rect::*;
pub use size::*;
pub use space::*;
pub use vector::*;

/// A constant used to indicate that a dimension should fill the available space.
pub const FILL: f32 = f32::INFINITY;
