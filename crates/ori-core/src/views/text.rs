use glam::Vec2;
use ori_graphics::{Rect, TextSection};
use ori_macro::Build;
use ori_style::Style;

use crate::{AvailableSpace, Context, DrawContext, IntoNode, LayoutContext, Node, View};

impl IntoNode for String {
    fn into_node(self) -> Node {
        Node::new(Text::new(self))
    }
}

impl IntoNode<Text> for String {
    fn into_node(self) -> Node<Text> {
        Node::new(Text::new(self))
    }
}

impl IntoNode for &str {
    fn into_node(self) -> Node {
        Node::new(Text::new(self))
    }
}

impl IntoNode<Text> for &str {
    fn into_node(self) -> Node<Text> {
        Node::new(Text::new(self))
    }
}

/// A text view.
#[derive(Clone, Debug, Default, Build)]
pub struct Text {
    /// The text to display.
    #[prop]
    pub text: String,
}

impl Text {
    /// Create a new text view.
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
    type State = Option<f32>;

    fn build(&self) -> Self::State {
        None
    }

    fn style(&self) -> Style {
        Style::new("text")
    }

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        let font_size = cx.style_range("font-size", 0.0..cx.parent_space.max.y);
        *state = Some(font_size);

        let section = TextSection {
            text: &self.text,
            font_size,
            font_family: cx.style("font-family"),
            font_weight: cx.style("font-weight"),
            font_stretch: cx.style("font-stretch"),
            font_style: cx.style("font-style"),
            color: cx.style("color"),
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            line_height: cx.style("line-height"),
            wrap: cx.style("text-wrap"),
            rect: Rect::min_size(Vec2::ZERO, space.max),
        };

        let text_rect = cx.measure_text(&section);
        space.constrain(text_rect.size())
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let font_size = state.unwrap_or(16.0);
        let section = TextSection {
            text: &self.text,
            font_size,
            font_family: cx.style("font-family"),
            font_weight: cx.style("font-weight"),
            font_stretch: cx.style("font-stretch"),
            font_style: cx.style("font-style"),
            color: cx.style("color"),
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            line_height: cx.style("line-height"),
            wrap: cx.style("text-wrap"),
            rect: cx.rect(),
        };

        cx.draw_text(&section);
    }
}
