mod attributes;
mod builder;
mod r#enum;
mod length;
mod value;

pub use attributes::*;
pub use builder::*;
pub use length::*;
pub use r#enum::*;
pub use value::*;

use std::{fmt::Display, sync::Arc};

use ori_graphics::{Color, FontFamily};
use smol_str::SmolStr;

use crate::StyleTransition;

/// A style attribute key.
pub type StyleAttributeKey = SmolStr;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct StyleAttributeInner {
    key: StyleAttributeKey,
    value: StyleAttributeValue,
    transition: Option<StyleTransition>,
}

/// A [`Style`](super::Style) attribute.
///
/// An attribute is a name and a value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleAttribute {
    inner: Arc<StyleAttributeInner>,
}

impl StyleAttribute {
    /// Create a new attribute.
    pub fn new(
        key: StyleAttributeKey,
        value: StyleAttributeValue,
        transition: Option<StyleTransition>,
    ) -> Self {
        Self {
            inner: Arc::new(StyleAttributeInner {
                key,
                value,
                transition,
            }),
        }
    }

    /// Get the key of the attribute.
    pub fn key(&self) -> &StyleAttributeKey {
        &self.inner.key
    }

    /// Get the value of the attribute.
    pub fn value(&self) -> &StyleAttributeValue {
        &self.inner.value
    }

    /// Get the transition of the attribute.
    pub fn transition(&self) -> Option<StyleTransition> {
        self.inner.transition
    }
}

impl Display for StyleAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {};", self.key(), self.value())
    }
}

impl<T: StyleAttributeEnum> From<T> for StyleAttributeValue {
    fn from(value: T) -> Self {
        Self::Enum(String::from(value.to_str()))
    }
}

/// A trait for types that can be converted from a [`StyleAttributeValue`].
pub trait FromStyleAttribute: Sized {
    /// Convert the given [`StyleAttributeValue`] into `Self`.
    fn from_attribute(value: StyleAttributeValue) -> Option<Self>;
}

impl<T: StyleAttributeEnum> FromStyleAttribute for T {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::Enum(value) => T::from_str(&value),
            _ => None,
        }
    }
}

impl<T: FromStyleAttribute> FromStyleAttribute for Option<T> {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        if value.is_none() {
            return Some(None);
        }

        T::from_attribute(value).map(Some)
    }
}

impl FromStyleAttribute for String {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::String(value) => Some(value),
            _ => None,
        }
    }
}

impl FromStyleAttribute for Length {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::Length(value) => Some(value),
            _ => None,
        }
    }
}

impl FromStyleAttribute for Color {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::Color(value) => Some(value),
            _ => None,
        }
    }
}

impl FromStyleAttribute for f32 {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::Length(value) => Some(value.as_f32()),
            _ => None,
        }
    }
}

impl FromStyleAttribute for FontFamily {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::Enum(value) => match value.as_str() {
                "sans-serif" => Some(FontFamily::SansSerif),
                "serif" => Some(FontFamily::Serif),
                "monospace" => Some(FontFamily::Monospace),
                "cursive" => Some(FontFamily::Cursive),
                "fantasy" => Some(FontFamily::Fantasy),
                _ => None,
            },
            StyleAttributeValue::String(value) => Some(FontFamily::Name(value)),
            _ => None,
        }
    }
}
