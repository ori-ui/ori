use ori_graphics::{FontStretch, FontStyle, FontWeight, TextAlign, TextWrap};

/// A trait for types that can be represented as a string enum, for a
/// [`StyleAttributeValue`](crate::StyleAttributeValue).
pub trait StyleAttributeEnum: Sized {
    /// Convert the given string into `Self`.
    fn from_str(s: &str) -> Option<Self>;
    /// Convert `Self` into a string.
    fn to_str(&self) -> &str;
}

impl StyleAttributeEnum for bool {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        if *self {
            "true"
        } else {
            "false"
        }
    }
}

impl StyleAttributeEnum for FontWeight {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "thin" => Some(Self::THIN),
            "extra-light" => Some(Self::EXTRA_LIGHT),
            "light" => Some(Self::LIGHT),
            "normal" => Some(Self::NORMAL),
            "medium" => Some(Self::MEDIUM),
            "semi-bold" => Some(Self::SEMI_BOLD),
            "bold" => Some(Self::BOLD),
            "extra-bold" => Some(Self::EXTRA_BOLD),
            "black" => Some(Self::BLACK),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match *self {
            Self::THIN => "thin",
            Self::EXTRA_LIGHT => "extra-light",
            Self::LIGHT => "light",
            Self::NORMAL => "normal",
            Self::MEDIUM => "medium",
            Self::SEMI_BOLD => "semi-bold",
            Self::BOLD => "bold",
            Self::EXTRA_BOLD => "extra-bold",
            Self::BLACK => "black",
            _ => "normal",
        }
    }
}

impl StyleAttributeEnum for FontStretch {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "ultra-condensed" => Some(Self::UltraCondensed),
            "extra-condensed" => Some(Self::ExtraCondensed),
            "condensed" => Some(Self::Condensed),
            "semi-condensed" => Some(Self::SemiCondensed),
            "normal" => Some(Self::Normal),
            "semi-expanded" => Some(Self::SemiExpanded),
            "expanded" => Some(Self::Expanded),
            "extra-expanded" => Some(Self::ExtraExpanded),
            "ultra-expanded" => Some(Self::UltraExpanded),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::UltraCondensed => "ultra-condensed",
            Self::ExtraCondensed => "extra-condensed",
            Self::Condensed => "condensed",
            Self::SemiCondensed => "semi-condensed",
            Self::Normal => "normal",
            Self::SemiExpanded => "semi-expanded",
            Self::Expanded => "expanded",
            Self::ExtraExpanded => "extra-expanded",
            Self::UltraExpanded => "ultra-expanded",
        }
    }
}

impl StyleAttributeEnum for FontStyle {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "normal" => Some(Self::Normal),
            "italic" => Some(Self::Italic),
            "oblique" => Some(Self::Oblique),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::Normal => "normal",
            Self::Italic => "italic",
            Self::Oblique => "oblique",
        }
    }
}

impl StyleAttributeEnum for TextAlign {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "left" | "top" | "start" => Some(Self::Start),
            "center" | "middle" => Some(Self::Center),
            "right" | "bottom" | "end" => Some(Self::End),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

impl StyleAttributeEnum for TextWrap {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "word" => Some(Self::Word),
            "letter" => Some(Self::Letter),
            _ => None,
        }
    }

    fn to_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Word => "word",
            Self::Letter => "letter",
        }
    }
}
