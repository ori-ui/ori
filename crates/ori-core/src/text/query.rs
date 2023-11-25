/// A font family.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontFamily {
    /// A font family by name, e.g. "Arial".
    Name(String),
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
    pub(crate) fn to_fontdb(&self) -> fontdb::Family<'_> {
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
        Self::Name(name.to_owned())
    }
}

impl From<String> for FontFamily {
    fn from(name: String) -> Self {
        Self::Name(name)
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

    pub(crate) fn to_fontdb(self) -> fontdb::Weight {
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
    pub(crate) fn to_fontdb(self) -> fontdb::Stretch {
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
    pub(crate) fn to_fontdb(self) -> fontdb::Style {
        match self {
            Self::Normal => fontdb::Style::Normal,
            Self::Italic => fontdb::Style::Italic,
            Self::Oblique => fontdb::Style::Oblique,
        }
    }
}

/// A query for a font.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct FontQuery {
    /// The font family.
    pub family: FontFamily,
    /// The font weight.
    pub weight: FontWeight,
    /// The font stretch.
    pub stretch: FontStretch,
    /// The font style.
    pub style: FontStyle,
}
