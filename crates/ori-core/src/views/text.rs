use ori_macro::Build;
use smol_str::SmolStr;

use crate::{
    canvas::{Canvas, Color},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    theme::{style, text},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Text`].
pub fn text(text: impl Into<SmolStr>) -> Text {
    Text::new(text)
}

/// A view that displays text.
#[derive(Build, Rebuild)]
pub struct Text {
    /// The text.
    #[rebuild(layout)]
    pub text: SmolStr,
    /// The font size of the text.
    #[rebuild(layout)]
    pub font_size: f32,
    /// The font family of the text.
    #[rebuild(layout)]
    pub font_family: FontFamily,
    /// The font weight of the text.
    #[rebuild(layout)]
    pub font_weight: FontWeight,
    /// The font stretch of the text.
    #[rebuild(layout)]
    pub font_stretch: FontStretch,
    /// The font.into of the text.
    #[rebuild(layout)]
    pub font_style: FontStyle,
    /// The color of the text.
    #[rebuild(layout)]
    pub color: Color,
    /// The horizontal alignment of the text.
    #[rebuild(layout)]
    pub align: TextAlign,
    /// The line height of the text.
    #[rebuild(layout)]
    pub line_height: f32,
    /// The text wrap of the text.
    #[rebuild(layout)]
    pub wrap: TextWrap,
}

impl Text {
    /// Creates a new text.
    pub fn new(text: impl Into<SmolStr>) -> Text {
        Text {
            text: text.into(),
            font_size: style(text::FONT_SIZE),
            font_family: style(text::FONT_FAMILY),
            font_weight: style(text::FONT_WEIGHT),
            font_stretch: style(text::FONT_STRETCH),
            font_style: style(text::FONT_STYLE),
            color: style(text::COLOR),
            align: style(text::ALIGN),
            line_height: style(text::LINE_HEIGHT),
            wrap: style(text::WRAP),
        }
    }

    fn set_attributes(&self, fonts: &mut Fonts, buffer: &mut TextBuffer) {
        buffer.set_wrap(fonts, self.wrap);
        buffer.set_align(self.align);
        buffer.set_text(
            fonts,
            &self.text,
            TextAttributes {
                family: self.font_family.clone(),
                stretch: self.font_stretch,
                weight: self.font_weight,
                style: self.font_style,
                color: self.color,
            },
        );
    }
}

impl<T> View<T> for Text {
    type State = TextBuffer;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let mut buffer = TextBuffer::new(cx.fonts(), self.font_size, self.line_height);
        self.set_attributes(cx.fonts(), &mut buffer);
        buffer
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
        self.set_attributes(cx.fonts(), state);
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        state.set_bounds(cx.fonts(), space.max);
        state.size()
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        let mesh = cx.rasterize_text(state, cx.rect());
        canvas.draw_pixel_perfect(mesh);
    }
}
