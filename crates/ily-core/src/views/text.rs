use glam::Vec2;
use ily_graphics::{Color, Rect, TextAlign, TextSection};

use crate::{attributes, BoxConstraints, DrawContext, LayoutContext, Length, Properties, View};

#[derive(Clone)]
pub struct Text {
    text: String,
    font_size: Option<Length>,
    font: Option<String>,
    color: Option<Color>,
    align: Option<TextAlign>,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: None,
            font: None,
            color: None,
            align: None,
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
    pub fn scale(mut self, scale: impl Into<Length>) -> Self {
        self.font_size = Some(scale.into());
        self
    }

    /// Set the font to use.
    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Set the color of the text.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the alignment of the text.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = Some(align);
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

    pub fn scale(&mut self, scale: impl Into<Length>) {
        self.text.font_size = Some(scale.into());
    }

    pub fn font(&mut self, font: impl Into<String>) {
        self.text.font = Some(font.into());
    }

    pub fn color(&mut self, color: Color) {
        self.text.color = Some(color);
    }

    pub fn align(&mut self, align: TextAlign) {
        self.text.align = Some(align);
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

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        attributes! {
            cx, self,
            font_size: "font-size",
            font: "font",
            color: "color",
            align: "align",
        }

        let font_size = font_size.pixels();

        let section = TextSection {
            bounds: Rect::min_size(Vec2::ZERO, bc.max),
            scale: font_size,
            align,
            text: self.text.clone(),
            font: (font.is_empty()).then(|| font),
            color,
        };

        let bounds = cx.text_bounds(&section).unwrap_or_default();
        bounds.size()
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        attributes! {
            cx, self,
            font_size: "font-size",
            font: "font",
            color: "color",
            align: "align",
        }

        let font_size = font_size.pixels();

        let section = TextSection {
            bounds: cx.rect(),
            align,
            text: self.text.clone(),
            scale: font_size,
            font: (font.is_empty()).then(|| font),
            color,
        };

        cx.draw_primitive(section);
    }
}
