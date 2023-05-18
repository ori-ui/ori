use glam::Vec2;
use ori_graphics::{Rect, TextSection};
use ori_macro::Build;

use crate::{BoxConstraints, Context, DrawContext, IntoView, LayoutContext, Style, View};

impl IntoView for String {
    type View = Text;

    fn into_view(self) -> Self::View {
        Text::new(self)
    }
}

impl IntoView for &str {
    type View = Text;

    fn into_view(self) -> Self::View {
        Text::new(self)
    }
}

#[derive(Clone, Debug, Build)]
pub struct Text {
    #[prop]
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

impl View for Text {
    type State = f32;

    fn build(&self) -> Self::State {
        0.0
    }

    fn style(&self) -> Style {
        Style::new("text")
    }

    #[tracing::instrument(name = "Text", skip(self, state, cx, bc))]
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
            font_family: cx.style("font-family"),
            color: cx.style("color"),
        };

        let bounds = cx.messure_text(&section).unwrap_or_default();
        bc.constrain(bounds.size())
    }

    #[tracing::instrument(name = "Text", skip(self, state, cx))]
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let section = TextSection {
            rect: cx.rect().ceil(),
            scale: *state,
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            wrap: cx.style("text-wrap"),
            text: self.text.clone(),
            font_family: cx.style("font-family"),
            color: cx.style("color"),
        };

        cx.draw(section);
    }
}
