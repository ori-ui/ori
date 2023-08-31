use crate::layout::Rect;

use super::{BorderRadius, BorderWidth, Color, Mesh};

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

impl Quad {
    /// Get whether the quad is ineffective, i.e. it has no effect on the canvas.
    pub fn is_ineffective(&self) -> bool {
        // if the rect has zero area, the quad is ineffective
        let rect = self.rect.area() == 0.0;

        let color = self.color.a == 0.0;
        let border = self.border_width == BorderWidth::ZERO || self.border_color.a == 0.0;

        rect || (color && border)
    }
}

/// A primitive to be rendered.
#[derive(Clone, Debug)]
pub enum Primitive {
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
