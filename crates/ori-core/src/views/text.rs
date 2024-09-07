use std::fmt::{self, Write};

use ori_macro::{example, Build, Styled};
use smol_str::SmolStr;

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::{key, Styled},
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
    #[styled(default -> "palette.contrast" or Color::BLACK)]
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
            font_size: key("text.font_size"),
            font_family: key("text.font_family"),
            font_weight: key("text.font_weight"),
            font_stretch: key("text.font_stretch"),
            font_style: key("text.font_style"),
            color: key("text.color"),
            align: key("text.align"),
            line_height: key("text.line_height"),
            wrap: key("text.wrap"),
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
        if self.font_size != old.font_size || self.line_height != old.line_height {
            (state.buffer).set_metrics(cx.fonts(), state.style.font_size, state.style.line_height);

            cx.layout();
        }

        if self.wrap != old.wrap {
            state.buffer.set_wrap(cx.fonts(), state.style.wrap);

            cx.draw();
        }

        if self.align != old.align {
            state.buffer.set_align(state.style.align);

            cx.draw();
        }

        if self.text != old.text
            || self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style
        {
            state.buffer.set_text(
                cx.fonts(),
                &self.text,
                TextAttributes {
                    family: state.style.font_family.clone(),
                    stretch: state.style.font_stretch,
                    weight: state.style.font_weight,
                    style: state.style.font_style,
                },
            );

            cx.layout();
        }

        if self.color != old.color {
            cx.draw();
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
