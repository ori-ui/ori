use crate::{Mesh, Quad, Rect};

/// A primitive that can be drawn to the screen, see [`Primitive`] for more information.
pub enum PrimitiveKind {
    Quad(Quad),
    Mesh(Mesh),
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

/// A primitive that can be drawn to the screen, see [`Frame`] for more information.
///
/// Primitives are drawn in order of their z-index, with primitives with a higher z-index being
/// drawn on top of primitives with a lower z-index. Primitives with the same z-index are drawn in
/// the order they are added to the frame.
///
/// Primitives can be clipped to a rectangle, see [`Frame::clip`] for more information.
pub struct Primitive {
    pub kind: PrimitiveKind,
    pub z_index: f32,
    pub clip: Option<Rect>,
}

/// A collection of primitives that can be drawn to the screen.
#[derive(Default)]
pub struct Frame {
    primitives: Vec<Primitive>,
    z_index: f32,
    clip: Option<Rect>,
}

impl Frame {
    /// Create a new frame.
    pub fn new() -> Self {
        Self {
            z_index: 0.0,
            primitives: Vec::new(),
            clip: None,
        }
    }

    /// Clear the frame.
    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    /// Get the z-index of the frame.
    pub fn z_index(&self) -> f32 {
        self.z_index
    }

    /// Get the clipping rectangle of the frame.
    pub fn clip(&self) -> Option<Rect> {
        self.clip
    }

    /// Draw a [`PrimitiveKind`] to the frame.
    pub fn draw(&mut self, primitive: impl Into<PrimitiveKind>) {
        self.draw_primitive(Primitive {
            kind: primitive.into(),
            z_index: self.z_index,
            clip: self.clip,
        });
    }

    /// Draw a [`Primitive`] to the frame.
    pub fn draw_primitive(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
    }

    /// Draws a [`Layer`] to the frame.
    pub fn layer(&mut self) -> Layer<'_> {
        Layer {
            frame: self,
            z_index: 1.0,
            clip: None,
        }
    }

    /// Draws a [`Layer`] to the frame.
    pub fn draw_layer(&mut self, f: impl FnOnce(&mut Self)) {
        self.layer().draw(f);
    }

    /// Get the primitives in the frame.
    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }
}

/// A layer is a frame with a z-index and a clipping rectangle, usually the z-index is one greater
/// than the z-index of the frame it is drawn to.
pub struct Layer<'a> {
    frame: &'a mut Frame,
    z_index: f32,
    clip: Option<Rect>,
}

impl<'a> Layer<'a> {
    /// Set the z-index of the layer.
    pub fn z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Set the clipping rectangle of the layer.
    pub fn clip(mut self, clip: impl Into<Option<Rect>>) -> Self {
        self.clip = clip.into().map(Rect::round);
        self
    }

    /// Draw to the layer, with `f` being called with a [`Frame`].
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
