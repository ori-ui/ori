use crate::{Mesh, Quad, Rect, TextSection};

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
    pub z_index: f32,
    pub clip: Option<Rect>,
}

pub struct Frame {
    primitives: Vec<Primitive>,
    z_index: f32,
    clip: Option<Rect>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            z_index: 0.0,
            primitives: Vec::new(),
            clip: None,
        }
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    pub fn z_index(&self) -> f32 {
        self.z_index
    }

    pub fn clip(&self) -> Option<Rect> {
        self.clip
    }

    pub fn draw(&mut self, primitive: impl Into<PrimitiveKind>) {
        self.draw_primitive(Primitive {
            kind: primitive.into(),
            z_index: self.z_index,
            clip: self.clip,
        });
    }

    pub fn draw_primitive(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
    }

    pub fn layer(&mut self) -> Layer<'_> {
        Layer {
            frame: self,
            z_index: 1.0,
            clip: None,
        }
    }

    pub fn draw_layer(&mut self, f: impl FnOnce(&mut Self)) {
        self.layer().draw(f);
    }

    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }
}

pub struct Layer<'a> {
    frame: &'a mut Frame,
    z_index: f32,
    clip: Option<Rect>,
}

impl<'a> Layer<'a> {
    pub fn z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn clip(mut self, clip: impl Into<Option<Rect>>) -> Self {
        self.clip = clip.into().map(Rect::round);
        self
    }

    pub fn draw(self, f: impl FnOnce(&mut Frame)) {
        self.frame.z_index += self.z_index;

        if let Some(clip) = self.clip {
            let old_clip = self.frame.clip;
            self.frame.clip = Some(clip);
            f(self.frame);
            self.frame.clip = old_clip;
        } else {
            f(self.frame);
        }

        self.frame.z_index -= self.z_index;
    }
}
