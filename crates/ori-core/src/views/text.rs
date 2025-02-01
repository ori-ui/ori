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
            font_size: Styled::style("text.font-size"),
            font_family: Styled::style("text.font-family"),
            font_weight: Styled::style("text.font-weight"),
            font_stretch: Styled::style("text.font-stretch"),
            font_style: Styled::style("text.font-style"),
            color: Styled::style("text.color"),
            align: Styled::style("text.align"),
            line_height: Styled::style("text.line-height"),
            wrap: Styled::style("text.wrap"),
        }
    }

    fn font_attributes(&self, style: &TextStyle) -> FontAttributes {
        FontAttributes {
            size: style.font_size,
            family: style.font_family.clone(),
            stretch: style.font_stretch,
            weight: style.font_weight,
            style: style.font_style,
            ligatures: true,
            color: style.color,
        }
    }
}

impl<T> View<T> for Text {
    type State = (TextStyle, Paragraph);

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let style = TextStyle::styled(self, cx.styles());

        let mut paragraph = Paragraph::new(style.line_height, style.align, style.wrap);
        paragraph.push_text(&self.text, self.font_attributes(&style));
        (style, paragraph)
    }

    fn rebuild(
        &mut self,
        (style, paragraph): &mut Self::State,
        cx: &mut RebuildCx,
        _data: &mut T,
        _old: &Self,
    ) {
        style.rebuild(self, cx);

        paragraph.line_height = style.line_height;
        paragraph.align = style.align;
        paragraph.wrap = style.wrap;

        paragraph.set_text(&self.text, self.font_attributes(style));
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
        (_, paragraph): &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        cx.fonts().measure(paragraph, space.max.width)
    }

    fn draw(&mut self, (_, paragraph): &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.paragraph(paragraph, cx.rect());
    }
}

impl From<fmt::Arguments<'_>> for Text {
    fn from(args: fmt::Arguments<'_>) -> Text {
        let mut w = smol_str::SmolStrBuilder::new();
        let _ = w.write_fmt(args);
        Text::new(w)
    }
}
