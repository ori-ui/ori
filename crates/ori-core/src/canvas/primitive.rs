use crate::layout::{Point, Rect};

use super::{Mesh, Quad};

/// A primitive to be rendered.
#[derive(Clone, Debug)]
pub enum Primitive {
    /// A trigger primitive.
    Trigger(Rect),
    /// A quad primitive.
    Quad(Quad),
    /// A mesh primitive.
    Mesh(Mesh),
}

impl Primitive {
    /// Get whether the primitive is ineffective, i.e. it has no effect on the canvas.
    pub fn is_ineffective(&self) -> bool {
        match self {
            Self::Quad(quad) => quad.is_ineffective(),
            _ => false,
        }
    }

    /// Get whether the primitive intersects with the given point.
    pub fn intersects_point(&self, point: Point) -> bool {
        match self {
            Self::Trigger(rect) => rect.contains(point),
            Self::Quad(quad) => quad.rect.contains(point),
            Self::Mesh(mesh) => mesh.intersects_point(point),
        }
    }

    /// Get whether `self` is a `Mesh`.
    pub fn is_mesh(&self) -> bool {
        matches!(self, Self::Mesh(_))
    }
}

impl From<Quad> for Primitive {
    fn from(quad: Quad) -> Self {
        Self::Quad(quad)
    }
}

impl From<Mesh> for Primitive {
    fn from(mesh: Mesh) -> Self {
        Self::Mesh(mesh)
    }
}
