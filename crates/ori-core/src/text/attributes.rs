use std::hash::{Hash, Hasher};

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
        Styled::value(FontFamily::from(name))
    }
}

impl From<String> for Styled<FontFamily> {
    fn from(name: String) -> Self {
        Styled::value(FontFamily::from(name))
    }
}

impl From<SmolStr> for Styled<FontFamily> {
    fn from(name: SmolStr) -> Self {
        Styled::value(FontFamily::from(name))
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

impl From<u16> for FontWeight {
    fn from(weight: u16) -> Self {
        Self(weight)
    }
}

impl From<&str> for FontWeight {
    fn from(weight: &str) -> Self {
        match weight {
            "thin" => Self::THIN,
            "extra-light" => Self::EXTRA_LIGHT,
            "light" => Self::LIGHT,
            "normal" => Self::NORMAL,
            "medium" => Self::MEDIUM,
            "semi-bold" => Self::SEMI_BOLD,
            "bold" => Self::BOLD,
            "extra-bold" => Self::EXTRA_BOLD,
            "black" => Self::BLACK,
            _ => Self::NORMAL,
        }
    }
}

impl From<String> for FontWeight {
    fn from(weight: String) -> Self {
        Self::from(weight.as_str())
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

impl From<&str> for FontStretch {
    fn from(stretch: &str) -> Self {
        match stretch {
            "ultra-condensed" => Self::UltraCondensed,
            "extra-condensed" => Self::ExtraCondensed,
            "condensed" => Self::Condensed,
            "semi-condensed" => Self::SemiCondensed,
            "normal" => Self::Normal,
            "semi-expanded" => Self::SemiExpanded,
            "expanded" => Self::Expanded,
            "extra-expanded" => Self::ExtraExpanded,
            "ultra-expanded" => Self::UltraExpanded,
            _ => Self::Normal,
        }
    }
}

impl From<String> for FontStretch {
    fn from(stretch: String) -> Self {
        Self::from(stretch.as_str())
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

impl From<&str> for FontStyle {
    fn from(style: &str) -> Self {
        match style {
            "normal" => Self::Normal,
            "italic" => Self::Italic,
            "oblique" => Self::Oblique,
            _ => Self::Normal,
        }
    }
}

impl From<String> for FontStyle {
    fn from(style: String) -> Self {
        Self::from(style.as_str())
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
}

impl From<&str> for TextAlign {
    fn from(align: &str) -> Self {
        match align {
            "start" => Self::Start,
            "center" => Self::Center,
            "end" => Self::End,
            "left" => Self::Start,
            "top" => Self::Start,
            "middle" => Self::Center,
            "right" => Self::End,
            "bottom" => Self::End,
            _ => Self::Start,
        }
    }
}

impl From<String> for TextAlign {
    fn from(align: String) -> Self {
        Self::from(align.as_str())
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

impl From<&str> for TextWrap {
    fn from(wrap: &str) -> Self {
        match wrap {
            "none" => Self::None,
            "word" => Self::Word,
            _ => Self::Word,
        }
    }
}

impl From<String> for TextWrap {
    fn from(wrap: String) -> Self {
        Self::from(wrap.as_str())
    }
}

/// Attributes of a section of text.
#[derive(Clone, Debug, PartialEq)]
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

    /// Whether to use ligatures.
    pub ligatures: bool,

    /// The color of the font.
    pub color: Color,
}

impl Default for FontAttributes {
    fn default() -> Self {
        Self {
            size: 16.0,
            family: FontFamily::SansSerif,
            stretch: FontStretch::Normal,
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            ligatures: true,
            color: Color::BLACK,
        }
    }
}

impl Eq for FontAttributes {}

impl Hash for FontAttributes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.to_bits().hash(state);
        self.family.hash(state);
        self.stretch.hash(state);
        self.weight.hash(state);
        self.style.hash(state);
        self.ligatures.hash(state);
        self.color.hash(state);
    }
}
