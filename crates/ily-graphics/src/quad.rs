use crate::{Color, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub background: Color,
    pub border_radius: [f32; 4],
    pub border_width: f32,
    pub border_color: Color,
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            background: Color::WHITE,
            border_radius: [0.0; 4],
            border_width: 0.0,
            border_color: Color::BLACK,
        }
    }
}

impl Quad {
    pub fn rounded(self) -> Self {
        Self {
            rect: self.rect.rounded(),
            ..self
        }
    }
}
