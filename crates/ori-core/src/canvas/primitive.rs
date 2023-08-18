use crate::{BorderRadius, BorderWidth, Color, Mesh, Rect};

/// A quad primitive.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quad {
    /// The rectangle of the quad.
    pub rect: Rect,
    /// The color of the quad.
    pub color: Color,
    /// The border radius of the quad.
    pub border_radius: BorderRadius,
    /// The border width of the quad.
    pub border_width: BorderWidth,
    /// The border color of the quad.
    pub border_color: Color,
}

/// A primitive to be rendered.
#[derive(Clone, Debug)]
pub enum Primitive {
    /// A quad primitive.
    Quad(Quad),
    /// A mesh primitive.
    Mesh(Mesh),
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
