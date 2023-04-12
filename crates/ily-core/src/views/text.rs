use glam::Vec2;
use ily_graphics::{Color, Rect, TextSection};

use crate::{BoxConstraints, DrawContext, LayoutContext, Properties, View};

#[derive(Clone)]
pub struct Text {
    text: String,
    scale: f32,
    font: Option<String>,
    color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            scale: 24.0,
            font: None,
            color: Color::hex("#333333"),
        }
    }
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Set the text to display.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set the scale of the text.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set the font to use.
    pub fn font(mut self, font: impl Into<Option<String>>) -> Self {
        self.font = font.into();
        self
    }

    /// Set the color of the text.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

pub struct TextProperties<'a> {
    text: &'a mut Text,
}

impl<'a> TextProperties<'a> {
    pub fn text(&mut self, text: impl Into<String>) {
        self.text.text = text.into();
    }

    pub fn scale(&mut self, scale: f32) {
        self.text.scale = scale;
    }

    pub fn font(&mut self, font: impl Into<Option<String>>) {
        self.text.font = font.into();
    }

    pub fn color(&mut self, color: Color) {
        self.text.color = color;
    }
}

impl Properties for Text {
    type Setter<'a> = TextProperties<'a>;

    fn setter(&mut self) -> Self::Setter<'_> {
        TextProperties { text: self }
    }
}

impl View for Text {
    type State = ();

    fn build(&self) -> Self::State {}

    fn element(&self) -> Option<&'static str> {
        Some("text")
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let section = TextSection {
            bounds: Rect::min_size(Vec2::ZERO, bc.max),
            text: self.text.clone(),
            scale: self.scale,
            font: self.font.clone(),
            color: self.color,
        };

        let bounds = cx.text_layout.bounds(&section).unwrap_or_default();
        bounds.size()
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let section = TextSection {
            bounds: cx.rect(),
            text: self.text.clone(),
            scale: self.scale,
            font: self.font.clone(),
            color: self.color,
        };

        cx.draw_primitive(section);
    }
}
