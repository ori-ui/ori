use glam::Vec2;
use ily_graphics::{Rect, TextSection};

use crate::{BoxConstraints, DrawContext, LayoutContext, Properties, Style, View};

#[derive(Clone, Debug)]
pub struct Text {
    text: String,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
        }
    }
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Set the text to display.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
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
}

impl Properties for Text {
    type Setter<'a> = TextProperties<'a>;

    fn setter(&mut self) -> Self::Setter<'_> {
        TextProperties { text: self }
    }
}

impl View for Text {
    type State = f32;

    fn build(&self) -> Self::State {
        0.0
    }

    fn style(&self) -> Style {
        Style::new("text")
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let font_size = cx.style_range("font-size", 0.0..bc.max.y);

        *state = font_size;

        let section = TextSection {
            rect: Rect::min_size(Vec2::ZERO, bc.max),
            scale: font_size,
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            wrap: cx.style("text-wrap"),
            text: self.text.clone(),
            font: cx.style("font"),
            color: cx.style("color"),
        };

        let bounds = cx.text_bounds(&section).unwrap_or_default();
        bounds.size()
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let section = TextSection {
            rect: cx.rect(),
            scale: *state,
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            wrap: cx.style("text-wrap"),
            text: self.text.clone(),
            font: cx.style("font"),
            color: cx.style("color"),
        };

        cx.draw_primitive(section);
    }
}
