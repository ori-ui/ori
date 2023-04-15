use glam::Vec2;
use ily_graphics::TextSection;

use crate::{BoxConstraints, DrawContext, LayoutContext, Properties, Style, View};

#[derive(Clone)]
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
        let font: String = cx.style("font");
        let font_size = cx.style_range("font-size", 0.0..bc.max.y);
        let color = cx.style("color");
        let h_align = cx.style("text-align");
        let v_align = cx.style("text-valign");
        let wrap = cx.style("text-wrap");

        *state = font_size;

        let section = TextSection {
            position: Vec2::ZERO,
            bounds: bc.max,
            scale: font_size,
            h_align,
            v_align,
            wrap,
            text: self.text.clone(),
            font: (font.is_empty()).then(|| font),
            color,
        };

        let bounds = cx.text_bounds(&section).unwrap_or_default();
        bounds.size()
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let font: String = cx.style("font");
        let color = cx.style("color");
        let h_align = cx.style("text-align");
        let v_align = cx.style("text-valign");
        let wrap = cx.style("text-wrap");

        let mut section = TextSection {
            scale: *state,
            h_align,
            v_align,
            wrap,
            text: self.text.clone(),
            font: (font.is_empty()).then(|| font),
            color,
            ..Default::default()
        };

        section.set_rect(cx.rect());
        section.bounds += Vec2::ONE;

        cx.draw_primitive(section);
    }
}
