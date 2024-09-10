use std::fmt::{self, Write};

use ori_macro::{example, Build, Styled};
use smol_str::SmolStr;

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::{Styled, Theme},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    view::View,
};

use smol_str;

pub use crate::format_text as text;

/// Create a formatted [`Text`].
///
/// This macro is slightly more efficient than using [`format!`] and [`Text::new`].
#[macro_export]
macro_rules! format_text {
    ($($tt:tt)*) => {
        $crate::views::Text::from(::std::format_args!($($tt)*))
    };
}

/// Create a new [`Text`].
pub fn text(text: impl Into<SmolStr>) -> Text {
    Text::new(text)
}

/// A view that displays text.
///
/// Can be styled using the [`TextStyle`].
#[example(name = "text", width = 400, height = 300)]
#[derive(Styled, Build)]
pub struct Text {
    /// The text.
    pub text: SmolStr,

    /// The font size of the text.
    #[styled(default = 16.0)]
    pub font_size: Styled<f32>,

    /// The font family of the text.
    #[styled(default)]
    pub font_family: Styled<FontFamily>,

    /// The font weight of the text.
    #[styled(default)]
    pub font_weight: Styled<FontWeight>,

    /// The font stretch of the text.
    #[styled(default)]
    pub font_stretch: Styled<FontStretch>,

    /// The font.into of the text.
    #[styled(default)]
    pub font_style: Styled<FontStyle>,

    /// The color of the text.
    #[styled(default -> Theme::CONTRAST or Color::BLACK)]
    pub color: Styled<Color>,

    /// The horizontal alignment of the text.
    #[styled(default)]
    pub align: Styled<TextAlign>,

    /// The line height of the text.
    #[styled(default = 1.2)]
    pub line_height: Styled<f32>,

    /// The text wrap of the text.
    #[styled(default)]
    pub wrap: Styled<TextWrap>,
}

impl Text {
    /// Create a new text.
    pub fn new(text: impl Into<SmolStr>) -> Self {
        Self {
            text: text.into(),
            font_size: TextStyle::FONT_SIZE.into(),
            font_family: TextStyle::FONT_FAMILY.into(),
            font_weight: TextStyle::FONT_WEIGHT.into(),
            font_stretch: TextStyle::FONT_STRETCH.into(),
            font_style: TextStyle::FONT_STYLE.into(),
            color: TextStyle::COLOR.into(),
            align: TextStyle::ALIGN.into(),
            line_height: TextStyle::LINE_HEIGHT.into(),
            wrap: TextStyle::WRAP.into(),
        }
    }

    fn set_attributes(&self, fonts: &mut Fonts, buffer: &mut TextBuffer, style: &TextStyle) {
        buffer.set_wrap(fonts, style.wrap);
        buffer.set_align(style.align);
        buffer.set_text(
            fonts,
            &self.text,
            TextAttributes {
                family: style.font_family.clone(),
                stretch: style.font_stretch,
                weight: style.font_weight,
                style: style.font_style,
            },
        );
    }
}

#[doc(hidden)]
pub struct TextState {
    style: TextStyle,
    buffer: TextBuffer,
}

impl<T> View<T> for Text {
    type State = TextState;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let style = TextStyle::styled(self, cx.styles());
        let mut buffer = TextBuffer::new(cx.fonts(), style.font_size, style.line_height);
        self.set_attributes(cx.fonts(), &mut buffer, &style);

        TextState { style, buffer }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        let style = TextStyle::styled(self, cx.styles());

        if style.font_size != state.style.font_size || style.line_height != state.style.line_height
        {
            (state.buffer).set_metrics(cx.fonts(), style.font_size, style.line_height);

            cx.layout();
        }

        if style.wrap != state.style.wrap {
            state.buffer.set_wrap(cx.fonts(), style.wrap);

            cx.draw();
        }

        if style.align != state.style.align {
            state.buffer.set_align(style.align);

            cx.draw();
        }

        if self.text != old.text
            || style.font_family != state.style.font_family
            || style.font_weight != state.style.font_weight
            || style.font_stretch != state.style.font_stretch
            || style.font_style != state.style.font_style
        {
            state.buffer.set_text(
                cx.fonts(),
                &self.text,
                TextAttributes {
                    family: style.font_family.clone(),
                    stretch: style.font_stretch,
                    weight: style.font_weight,
                    style: style.font_style,
                },
            );

            cx.layout();
        }

        if style.color != state.style.color {
            cx.draw();
        }

        state.style = style;
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
        if state.buffer.bounds() != space.max {
            state.buffer.set_bounds(cx.fonts(), space.max);
        }

        space.fit(state.buffer.size())
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        let offset = cx.rect().center() - state.buffer.rect().center();
        cx.text(&state.buffer, state.style.color, offset);
    }
}

impl From<fmt::Arguments<'_>> for Text {
    fn from(args: fmt::Arguments<'_>) -> Text {
        let mut w = smol_str::SmolStrBuilder::new();
        let _ = w.write_fmt(args);
        Text::new(w)
    }
}
