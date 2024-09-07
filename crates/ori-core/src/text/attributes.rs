use cosmic_text::fontdb;
use smol_str::SmolStr;

use crate::style::Styled;

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

impl FontFamily {
    /// Convert the font family to a [`fontdb::Family`].
    pub fn as_fontdb(&self) -> fontdb::Family<'_> {
        match self {
            Self::Name(name) => fontdb::Family::Name(name),
            Self::Serif => fontdb::Family::Serif,
            Self::SansSerif => fontdb::Family::SansSerif,
            Self::Monospace => fontdb::Family::Monospace,
            Self::Cursive => fontdb::Family::Cursive,
            Self::Fantasy => fontdb::Family::Fantasy,
        }
    }
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

    /// Convert the font weight to a [`fontdb::Weight`].
    pub fn to_fontdb(self) -> fontdb::Weight {
        fontdb::Weight(self.0)
    }
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

impl FontStretch {
    /// Convert the font stretch to a [`fontdb::Stretch`].
    pub fn to_fontdb(self) -> fontdb::Stretch {
        match self {
            Self::UltraCondensed => fontdb::Stretch::UltraCondensed,
            Self::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
            Self::Condensed => fontdb::Stretch::Condensed,
            Self::SemiCondensed => fontdb::Stretch::SemiCondensed,
            Self::Normal => fontdb::Stretch::Normal,
            Self::SemiExpanded => fontdb::Stretch::SemiExpanded,
            Self::Expanded => fontdb::Stretch::Expanded,
            Self::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
            Self::UltraExpanded => fontdb::Stretch::UltraExpanded,
        }
    }
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

impl FontStyle {
    /// Convert the font style to a [`fontdb::Style`].
    pub fn to_fontdb(self) -> fontdb::Style {
        match self {
            Self::Normal => fontdb::Style::Normal,
            Self::Italic => fontdb::Style::Italic,
            Self::Oblique => fontdb::Style::Oblique,
        }
    }
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

    /// Convert the text align to a [`cosmic_text::Align`].
    pub fn to_cosmic_text(self) -> cosmic_text::Align {
        match self {
            TextAlign::Start => cosmic_text::Align::Left,
            TextAlign::Center => cosmic_text::Align::Center,
            TextAlign::End => cosmic_text::Align::Right,
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
}

impl TextWrap {
    /// Convert the text wrap to a [`cosmic_text::Wrap`].
    pub fn to_cosmic_text(self) -> cosmic_text::Wrap {
        match self {
            Self::None => cosmic_text::Wrap::None,
            Self::Word => cosmic_text::Wrap::Word,
        }
    }
}

/// Attributes of a section of text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextAttributes {
    /// The font family of the text.
    pub family: FontFamily,

    /// The font size of the text.
    pub stretch: FontStretch,

    /// The font weight of the text.
    pub weight: FontWeight,

    /// The font style of the text.
    pub style: FontStyle,
}

impl TextAttributes {
    /// Convert the text attributes to a [`cosmic_text::Attrs`].
    pub fn to_cosmic_text(&self) -> cosmic_text::Attrs<'_> {
        cosmic_text::Attrs {
            cache_key_flags: cosmic_text::CacheKeyFlags::empty(),
            color_opt: None,
            family: self.family.as_fontdb(),
            stretch: self.stretch.to_fontdb(),
            style: self.style.to_fontdb(),
            weight: self.weight.to_fontdb(),
            metadata: 0,
            metrics_opt: None,
        }
    }
}
