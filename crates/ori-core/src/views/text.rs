use std::fmt::{self, Write};

use ori_macro::{example, Build, Styled};
use smol_str::SmolStr;

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    style::{Styled, Theme},
    text::{
        FontAttributes, FontFamily, FontStretch, FontStyle, FontWeight, Paragraph, TextAlign,
        TextWrap,
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
#[derive(Styled, Build, Rebuild)]
pub struct Text {
    /// The text.
    #[rebuild(layout)]
    pub text: SmolStr,

    /// The font size of the text.
    #[styled(default = 16.0)]
    #[rebuild(layout)]
    pub font_size: Styled<f32>,

    /// The font family of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_family: Styled<FontFamily>,

    /// The font weight of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_weight: Styled<FontWeight>,

    /// The font stretch of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_stretch: Styled<FontStretch>,

    /// The font.into of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_style: Styled<FontStyle>,

    /// The color of the text.
    #[styled(default -> Theme::CONTRAST or Color::BLACK)]
    #[rebuild(draw)]
    pub color: Styled<Color>,

    /// The horizontal alignment of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub align: Styled<TextAlign>,

    /// The line height of the text.
    #[styled(default = 1.2)]
    #[rebuild(layout)]
    pub line_height: Styled<f32>,

    /// The text wrap of the text.
    #[styled(default)]
    #[rebuild(layout)]
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

    fn font_attributes(&self, style: &TextStyle) -> FontAttributes {
        FontAttributes {
            size: style.font_size,
            family: style.font_family.clone(),
            stretch: style.font_stretch,
            weight: style.font_weight,
            style: style.font_style,
            color: style.color,
        }
    }
}

impl<T> View<T> for Text {
    type State = Paragraph;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let style = TextStyle::styled(self, cx.styles());

        let mut paragraph = Paragraph::new(style.line_height, style.align, style.wrap);
        paragraph.push_text(&self.text, self.font_attributes(&style));
        paragraph
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        let style = TextStyle::styled(self, cx.styles());

        state.line_height = style.line_height;
        state.align = style.align;
        state.wrap = style.wrap;

        state.set_text(&self.text, self.font_attributes(&style));
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) -> bool {
        false
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        cx.fonts().measure(state, space.max.width)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.text(state, cx.rect());
    }
}

impl From<fmt::Arguments<'_>> for Text {
    fn from(args: fmt::Arguments<'_>) -> Text {
        let mut w = smol_str::SmolStrBuilder::new();
        let _ = w.write_fmt(args);
        Text::new(w)
    }
}
