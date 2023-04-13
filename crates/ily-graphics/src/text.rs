use crate::{Color, Rect};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, Default)]
pub struct TextSection {
    pub bounds: Rect,
    pub scale: f32,
    pub align: TextAlign,
    pub text: String,
    pub font: Option<String>,
    pub color: Color,
}

pub trait TextLayout {
    fn bounds(&self, section: &TextSection) -> Option<Rect>;
}
