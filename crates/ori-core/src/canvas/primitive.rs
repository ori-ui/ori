use crate::{BorderRadius, BorderWidth, Color, Mesh, Rect};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub color: Color,
    pub border_radius: BorderRadius,
    pub border_width: BorderWidth,
    pub border_color: Color,
}

#[derive(Clone, Debug)]
pub enum Primitive {
    Quad(Quad),
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
