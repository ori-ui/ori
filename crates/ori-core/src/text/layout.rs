use fontdue::layout::{HorizontalAlign, VerticalAlign, WrapStyle};

use crate::{canvas::Color, layout::Size};

use super::{FontFamily, FontQuery, FontStretch, FontStyle, FontWeight};

/// Alignment of a section of text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextAlign {
    /// Align text at the start.
    #[default]
    Start,
    /// Align text in the center.
    Center,
    /// Align text at the end.
    End,
}

#[allow(non_upper_case_globals, missing_docs)]
impl TextAlign {
    pub const Left: Self = Self::Start;
    pub const Top: Self = Self::Start;
    pub const Middle: Self = Self::Center;
    pub const Right: Self = Self::End;
    pub const Bottom: Self = Self::End;
}

impl TextAlign {
    pub(crate) fn to_horizontal(self) -> HorizontalAlign {
        match self {
            Self::Start => HorizontalAlign::Left,
            Self::Center => HorizontalAlign::Center,
            Self::End => HorizontalAlign::Right,
        }
    }

    pub(crate) fn to_vertical(self) -> VerticalAlign {
        match self {
            TextAlign::Start => VerticalAlign::Bottom,
            TextAlign::Center => VerticalAlign::Middle,
            TextAlign::End => VerticalAlign::Top,
        }
    }
}

/// Wrapping of a section of text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextWrap {
    /// Do not wrap text.
    None,
    /// Wrap text at the word boundary.
    #[default]
    Word,
    /// Wrap text at the character boundary.
    Letter,
}

impl TextWrap {
    pub(crate) fn to_fontdue(self) -> WrapStyle {
        match self {
            Self::None => WrapStyle::Word,
            Self::Word => WrapStyle::Word,
            Self::Letter => WrapStyle::Letter,
        }
    }
}

/// A section of text.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextSection<'a> {
    /// The text.
    pub text: &'a str,
    /// The font size.
    pub font_size: f32,
    /// The font family.
    pub font_family: FontFamily,
    /// The font weight.
    pub font_weight: FontWeight,
    /// The font stretch.
    pub font_stretch: FontStretch,
    /// The font style.
    pub font_style: FontStyle,
    /// The text color.
    pub color: Color,
    /// The vertical alignment.
    pub v_align: TextAlign,
    /// The horizontal alignment.
    pub h_align: TextAlign,
    /// The line height.
    pub line_height: f32,
    /// The wrapping.
    pub wrap: TextWrap,
    /// The bounding rectangle.
    pub bounds: Size,
}

impl<'a> Default for TextSection<'a> {
    fn default() -> Self {
        Self {
            text: Default::default(),
            font_size: 16.0,
            font_family: FontFamily::default(),
            font_weight: FontWeight::default(),
            font_stretch: FontStretch::default(),
            font_style: FontStyle::default(),
            color: Color::default(),
            v_align: TextAlign::default(),
            h_align: TextAlign::default(),
            line_height: 1.0,
            wrap: TextWrap::default(),
            bounds: Size::UNBOUNDED,
        }
    }
}

impl<'a> TextSection<'a> {
    /// Get the font query for this section.
    pub fn font_query(&self) -> FontQuery {
        FontQuery {
            family: self.font_family.clone(),
            weight: self.font_weight,
            stretch: self.font_stretch,
            style: self.font_style,
        }
    }
}
