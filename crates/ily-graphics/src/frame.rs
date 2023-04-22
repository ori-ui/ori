use crate::{Mesh, Quad, TextSection};

#[derive(Clone, Debug)]
pub enum PrimitiveKind {
    Text(TextSection),
    Quad(Quad),
    Mesh(Mesh),
}

impl From<TextSection> for PrimitiveKind {
    fn from(text: TextSection) -> Self {
        Self::Text(text)
    }
}

impl From<Quad> for PrimitiveKind {
    fn from(quad: Quad) -> Self {
        Self::Quad(quad)
    }
}

impl From<Mesh> for PrimitiveKind {
    fn from(mesh: Mesh) -> Self {
        Self::Mesh(mesh)
    }
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub kind: PrimitiveKind,
    pub depth: f32,
}

pub struct Frame {
    layer: f32,
    primitives: Vec<Primitive>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            layer: 0.0,
            primitives: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    pub fn layer(&self) -> f32 {
        self.layer
    }

    // a Sigmoid function that maps the range [-inf, inf] to [0, 1]
    fn sigmoid(layer: f32) -> f32 {
        1.0 / (1.0 + (-layer).exp())
    }

    pub fn draw(&mut self, primitive: impl Into<PrimitiveKind>) {
        self.draw_with_layer(primitive, self.layer);
    }

    pub fn draw_with_layer(&mut self, primitive: impl Into<PrimitiveKind>, layer: f32) {
        self.primitives.push(Primitive {
            kind: primitive.into(),
            depth: Self::sigmoid(self.layer + layer),
        });
    }

    pub fn draw_layer(&mut self, offset: f32, f: impl FnOnce(&mut Self)) {
        self.layer += offset;
        f(self);
        self.layer -= offset;
    }

    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }
}
