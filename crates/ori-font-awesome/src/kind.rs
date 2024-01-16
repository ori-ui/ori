use ori_core::text::{FontFamily, FontWeight};

include!(concat!(env!("OUT_DIR"), "/icons.rs"));

impl From<&str> for IconCode {
    fn from(value: &str) -> Self {
        Self::from_name(value).unwrap_or(Self::Question)
    }
}

/// The font to use for icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconFont {
    /// The regular font.
    Regular,
    /// The solid font.
    Solid,
    /// The brand font.
    Brand,
}

impl IconFont {
    /// Get the font family for the icon font.
    pub fn family(self) -> FontFamily {
        let name = match self {
            Self::Regular | Self::Solid => "Font Awesome 6 Free",
            Self::Brand => "Font Awesome 6 Brands",
        };

        FontFamily::Name(String::from(name))
    }

    /// Get the font weight for the icon font.
    pub fn weight(self) -> FontWeight {
        match self {
            Self::Regular | Self::Brand => FontWeight::NORMAL,
            Self::Solid => FontWeight::BLACK,
        }
    }
}
