use crate::{Color, Rect};

#[derive(Clone, Debug, Default)]
pub struct TextSection {
    pub bounds: Rect,
    pub scale: f32,
    pub text: String,
    pub font: Option<String>,
    pub color: Color,
}

pub trait TextLayout {
    fn bounds(&self, section: &TextSection) -> Option<Rect>;
}
