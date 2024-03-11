use ori_macro::Build;
use smol_str::SmolStr;

use crate::{
    canvas::{Canvas, Color},
    event::Event,
    layout::{Size, Space},
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
#[derive(Build)]
pub struct Text {
    /// The text.
    pub text: SmolStr,
    /// The font size of the text.
    pub font_size: f32,
    /// The font family of the text.
    pub font_family: FontFamily,
    /// The font weight of the text.
    pub font_weight: FontWeight,
    /// The font stretch of the text.
    pub font_stretch: FontStretch,
    /// The font.into of the text.
    pub font_style: FontStyle,
    /// The color of the text.
    pub color: Color,
    /// The horizontal alignment of the text.
    pub align: TextAlign,
    /// The line height of the text.
    pub line_height: f32,
    /// The text wrap of the text.
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
        if self.wrap != old.wrap {
            state.set_wrap(cx.fonts(), self.wrap);

            cx.request_layout();
        }

        if self.align != old.align {
            state.set_align(self.align);

            cx.request_layout();
        }

        if self.text != old.text
            || self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style
            || self.color != old.color
        {
            state.set_text(
                cx.fonts(),
                &self.text,
                TextAttributes {
                    family: self.font_family.clone(),
                    stretch: self.font_stretch,
                    weight: self.font_weight,
                    style: self.font_style,
                    color: self.color,
                },
            );

            cx.request_layout();
        }
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
        space.fit(state.size())
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        let offset = cx.rect().center() - state.rect().center();

        let mesh = cx.rasterize_text(state, cx.rect());
        canvas.translate(offset);
        canvas.draw_pixel_perfect(mesh);
    }
}
