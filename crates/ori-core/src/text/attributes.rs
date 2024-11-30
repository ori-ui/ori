use smol_str::SmolStr;

use crate::{canvas::Color, style::Styled};

/// A font family, by default [`FontFamily::SansSerif`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// A font family by name, e.g. "Arial".
    Name(SmolStr),

    /// A serif font family.
    Serif,

    /// A sans-serif font family.
    #[default]
    SansSerif,

    /// A monospace font family.
    Monospace,

    /// A cursive font family.
    Cursive,

    /// A fantasy font family.
    Fantasy,
}

impl From<&str> for FontFamily {
    fn from(name: &str) -> Self {
        Self::Name(SmolStr::new(name))
    }
}

impl From<String> for FontFamily {
    fn from(name: String) -> Self {
        Self::Name(SmolStr::new(name))
    }
}

impl From<SmolStr> for FontFamily {
    fn from(name: SmolStr) -> Self {
        Self::Name(name)
    }
}

impl From<&str> for Styled<FontFamily> {
    fn from(name: &str) -> Self {
        Styled::Value(FontFamily::from(name))
    }
}

impl From<String> for Styled<FontFamily> {
    fn from(name: String) -> Self {
        Styled::Value(FontFamily::from(name))
    }
}

impl From<SmolStr> for Styled<FontFamily> {
    fn from(name: SmolStr) -> Self {
        Styled::Value(FontFamily::from(name))
    }
}

/// A font weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontWeight(pub u16);

impl FontWeight {
    /// Thin font weight (100), the thinnest possible.
    pub const THIN: Self = Self(100);
    /// Extra-light font weight (200).
    pub const EXTRA_LIGHT: Self = Self(200);
    /// Light font weight (300).
    pub const LIGHT: Self = Self(300);
    /// Normal font weight (400), the default.
    pub const NORMAL: Self = Self(400);
    /// Medium font weight (500).
    pub const MEDIUM: Self = Self(500);
    /// Semi-bold font weight (600).
    pub const SEMI_BOLD: Self = Self(600);
    /// Bold font weight (700).
    pub const BOLD: Self = Self(700);
    /// Extra-bold font weight (800).
    pub const EXTRA_BOLD: Self = Self(800);
    /// Black font weight (900), the boldest possible.
    pub const BLACK: Self = Self(900);
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// A font stretch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontStretch {
    /// Ultra-condensed font stretch.
    UltraCondensed,

    /// Extra-condensed font stretch.
    ExtraCondensed,

    /// Condensed font stretch.
    Condensed,

    /// Semi-condensed font stretch.
    SemiCondensed,

    /// Normal font stretch, the default.
    #[default]
    Normal,

    /// Semi-expanded font stretch.
    SemiExpanded,

    /// Expanded font stretch.
    Expanded,

    /// Extra-expanded font stretch.
    ExtraExpanded,

    /// Ultra-expanded font stretch.
    UltraExpanded,
}

/// A font style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontStyle {
    /// Normal font style, the default.
    #[default]
    Normal,

    /// Italic font style.
    Italic,

    /// Oblique font style.
    Oblique,
}

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

/// Wrapping of a section of text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextWrap {
    /// Do not wrap text.
    None,

    /// Wrap text at the word boundary.
    #[default]
    Word,
}

/// Attributes of a section of text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontAttributes {
    /// The font size of the font.
    pub size: f32,

    /// The font family of the font.
    pub family: FontFamily,

    /// The font stretch of the font.
    pub stretch: FontStretch,

    /// The font weight of the font.
    pub weight: FontWeight,

    /// The font style of the font.
    pub style: FontStyle,

    /// The color of the font.
    pub color: Color,
}
