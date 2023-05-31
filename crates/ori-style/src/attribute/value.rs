use std::fmt::Display;

use ori_graphics::Color;

use crate::Length;

/// A style attribute value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StyleAttributeValue {
    /// A string value, eg. `"hello"`.
    String(String),
    /// An enum value, eg. `red` or `space-between`.
    Enum(String),
    /// A length value, eg. `10px` or `10pt`.
    Length(Length),
    /// A color value, eg. `#ff0000`.
    Color(Color),
}

impl StyleAttributeValue {
    /// Check if the value is `none`.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::Enum(value) if value == "none")
    }
}

impl Display for StyleAttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => write!(f, "\"{}\"", value),
            Self::Enum(value) => write!(f, "{}", value),
            Self::Length(value) => write!(f, "{}", value),
            Self::Color(value) => write!(f, "{}", value),
        }
    }
}

impl From<String> for StyleAttributeValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for StyleAttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<Length> for StyleAttributeValue {
    fn from(value: Length) -> Self {
        Self::Length(value)
    }
}

impl From<Color> for StyleAttributeValue {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

macro_rules! num_impl {
    ($($t:ty),*) => {
        $(
            impl From<$t> for StyleAttributeValue {
                fn from(value: $t) -> Self {
                    Self::Length(Length::Px(value as f32))
                }
            }
        )*
    };
}

num_impl!(f32, f64, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
