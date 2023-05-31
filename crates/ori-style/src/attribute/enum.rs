use ori_graphics::TextAlign;

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

impl StyleAttributeEnum for TextAlign {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "left" | "start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "right" | "end" => Some(Self::End),
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
