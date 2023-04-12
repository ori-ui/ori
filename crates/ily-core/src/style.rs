use ily_graphics::Color;

#[derive(Clone, Debug, Default)]
pub struct Style {}

#[derive(Clone, Debug)]
pub enum AttributeKind {
    String(String),
    Length(f32),
    Color(Color),
}
