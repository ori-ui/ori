use crate::{Mesh, Quad, Rect, TextSection};

#[derive(Clone, Debug)]
pub enum Primitive {
    Group(Vec<Primitive>),
    Text(TextSection),
    Quad(Quad),
    Mesh(Mesh),
    Clip { rect: Rect, content: Box<Primitive> },
}

impl Primitive {
    pub fn visit(&self, f: &mut dyn FnMut(&Primitive)) {
        f(self);

        match self {
            Self::Group(primitives) => {
                for primitive in primitives {
                    primitive.visit(f);
                }
            }
            Self::Clip { content, .. } => {
                content.visit(f);
            }
            _ => {}
        }
    }
}

impl From<TextSection> for Primitive {
    fn from(text: TextSection) -> Self {
        Self::Text(text)
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

pub struct Frame {
    primitives: Vec<Primitive>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    pub fn layer(&mut self, f: impl FnOnce(&mut Self)) {
        let mut renderer = Self::new();
        f(&mut renderer);
        self.primitives.push(Primitive::Group(renderer.primitives));
    }

    pub fn draw_primitive(&mut self, primitive: impl Into<Primitive>) {
        self.primitives.push(primitive.into());
    }

    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub fn visit_primitives(&self, mut f: impl FnMut(&Primitive)) {
        for primitive in &self.primitives {
            primitive.visit(&mut f);
        }
    }
}
