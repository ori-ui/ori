use std::fmt::{self, Write};

use ori_macro::{example, Build};
use smol_str::SmolStr;

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
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

/// The style of a [`Text`].
#[derive(Clone, Rebuild)]
pub struct TextStyle {
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

    /// The font style of the text.
    #[rebuild(layout)]
    pub font_style: FontStyle,

    /// The color of the text.
    #[rebuild(draw)]
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

impl Style for TextStyle {
    fn builder() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| Self {
            font_size: 16.0,
            font_family: FontFamily::default(),
            font_weight: FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style: FontStyle::Normal,
            color: theme.contrast,
            align: TextAlign::Left,
            line_height: 1.2,
            wrap: TextWrap::None,
        })
    }
}

/// A view that displays text.
///
/// Can be styled using the [`TextStyle`].
#[example(name = "text", width = 400, height = 300)]
#[derive(Build, Rebuild)]
pub struct Text {
    /// The text.
    #[rebuild(layout)]
    pub text: SmolStr,

    /// The font size of the text.
    pub font_size: Option<f32>,

    /// The font family of the text.
    pub font_family: Option<FontFamily>,

    /// The font weight of the text.
    pub font_weight: Option<FontWeight>,

    /// The font stretch of the text.
    pub font_stretch: Option<FontStretch>,

    /// The font.into of the text.
    pub font_style: Option<FontStyle>,

    /// The color of the text.
    pub color: Option<Color>,

    /// The horizontal alignment of the text.
    pub align: Option<TextAlign>,

    /// The line height of the text.
    pub line_height: Option<f32>,

    /// The text wrap of the text.
    pub wrap: Option<TextWrap>,
}

impl Text {
    /// Create a new text.
    pub fn new(text: impl Into<SmolStr>) -> Self {
        Self {
            text: text.into(),
            font_size: None,
            font_family: None,
            font_weight: None,
            font_stretch: None,
            font_style: None,
            color: None,
            align: None,
            line_height: None,
            wrap: None,
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

impl Stylable for Text {
    type Style = TextStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        Self::Style {
            font_size: self.font_size.unwrap_or(style.font_size),
            font_family: (self.font_family.clone()).unwrap_or_else(|| style.font_family.clone()),
            font_weight: self.font_weight.unwrap_or(style.font_weight),
            font_stretch: self.font_stretch.unwrap_or(style.font_stretch),
            font_style: self.font_style.unwrap_or(style.font_style),
            color: self.color.unwrap_or(style.color),
            align: self.align.unwrap_or(style.align),
            line_height: self.line_height.unwrap_or(style.line_height),
            wrap: self.wrap.unwrap_or(style.wrap),
        }
    }
}

impl<T> View<T> for Text {
    type State = (TextStyle, Paragraph);

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let style = self.style(cx.style());

        let mut paragraph = Paragraph::new(style.line_height, style.align, style.wrap);
        paragraph.push_text(&self.text, self.font_attributes(&style));
        (style, paragraph)
    }

    fn rebuild(
        &mut self,
        (style, paragraph): &mut Self::State,
        cx: &mut RebuildCx,
        _data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);
        self.rebuild_style(cx, style);

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
