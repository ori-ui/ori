use std::fmt::Display;

use ily_graphics::{Color, TextAlign};
use smallvec::SmallVec;

use crate::{StyleTransition, Unit};

/// A collection of [`StyleAttribute`]s.
#[derive(Clone, Debug, Default)]
pub struct StyleAttributes {
    attributes: SmallVec<[StyleAttribute; 8]>,
}

impl StyleAttributes {
    pub const fn new() -> Self {
        Self {
            attributes: SmallVec::new_const(),
        }
    }

    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    pub fn clear(&mut self) {
        self.attributes.clear();
    }

    pub fn add(&mut self, attribute: StyleAttribute) {
        self.attributes.push(attribute);
    }

    pub fn extend(&mut self, attributes: impl IntoIterator<Item = StyleAttribute>) {
        self.attributes.extend(attributes);
    }

    pub fn get(&self, name: &str) -> Option<&StyleAttribute> {
        for attribute in self.attributes.iter() {
            if attribute.key == name {
                return Some(&attribute);
            }
        }

        None
    }

    pub fn get_value<T: FromStyleAttribute>(&self, name: &str) -> Option<T> {
        for attribute in self.attributes.iter().rev() {
            if attribute.key != name {
                continue;
            }

            if let Some(value) = T::from_attribute(attribute.value.clone()) {
                return Some(value);
            } else {
                tracing::warn!(
                    "Invalid attribute value for attribute '{}': {:?}, expected '{}'.",
                    name,
                    attribute.value,
                    std::any::type_name::<T>(),
                );
            }
        }

        None
    }

    pub fn get_value_and_transition<T: FromStyleAttribute>(
        &self,
        name: &str,
    ) -> Option<(T, Option<StyleTransition>)> {
        for attribute in self.attributes.iter().rev() {
            if attribute.key != name {
                continue;
            }

            if let Some(value) = T::from_attribute(attribute.value.clone()) {
                return Some((value, attribute.transition));
            } else {
                tracing::warn!(
                    "Invalid attribute value for attribute '{}': {:?}, expected '{}'.",
                    name,
                    attribute.value,
                    std::any::type_name::<T>(),
                );
            }
        }

        None
    }
}

impl IntoIterator for StyleAttributes {
    type Item = StyleAttribute;
    type IntoIter = smallvec::IntoIter<[Self::Item; 8]>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.into_iter()
    }
}

impl<'a> IntoIterator for &'a StyleAttributes {
    type Item = &'a StyleAttribute;
    type IntoIter = std::slice::Iter<'a, StyleAttribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter()
    }
}

impl FromIterator<StyleAttribute> for StyleAttributes {
    fn from_iter<T: IntoIterator<Item = StyleAttribute>>(iter: T) -> Self {
        Self {
            attributes: iter.into_iter().collect(),
        }
    }
}

/// A [`Style`](super::Style) attribute.
///
/// An attribute is a name and a value.
#[derive(Clone, Debug)]
pub struct StyleAttribute {
    /// The attribute key.
    pub key: String,
    /// The attribute value.
    pub value: StyleAttributeValue,
    /// The transition to use when animating the attribute.
    pub transition: Option<StyleTransition>,
}

impl StyleAttribute {
    pub fn new(key: impl Into<String>, value: impl Into<StyleAttributeValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            transition: None,
        }
    }

    pub fn with_transition(
        key: impl Into<String>,
        value: impl Into<StyleAttributeValue>,
        transition: impl Into<StyleTransition>,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            transition: Some(transition.into()),
        }
    }
}

pub trait StyleAttributeBuilder {
    fn attribute(self, key: impl Into<String>) -> StyleAttribute;
}

impl<T: Into<StyleAttributeValue>> StyleAttributeBuilder for T {
    fn attribute(self, key: impl Into<String>) -> StyleAttribute {
        StyleAttribute::new(key, self)
    }
}

impl<T: Into<StyleAttributeValue>, U: Into<StyleTransition>> StyleAttributeBuilder for (T, U) {
    fn attribute(self, key: impl Into<String>) -> StyleAttribute {
        StyleAttribute::with_transition(key, self.0, self.1)
    }
}

/// An ease of use function to create an [`AttributeBuilder`] with a transition.
pub fn trans(
    value: impl Into<StyleAttributeValue>,
    transition: impl Into<StyleTransition>,
) -> impl StyleAttributeBuilder {
    (value, transition)
}

/// A [`Style`](super::Style) attribute value.
#[derive(Clone, Debug)]
pub enum StyleAttributeValue {
    /// A string value, eg. `red`.
    String(String),
    /// A length value, eg. `10px` or `10pt`.
    Unit(Unit),
    /// A color value, eg. `#ff0000`.
    Color(Color),
}

impl Display for StyleAttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => write!(f, "{}", value),
            Self::Unit(value) => write!(f, "{}", value),
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

impl From<f32> for StyleAttributeValue {
    fn from(value: f32) -> Self {
        Self::Unit(Unit::Px(value))
    }
}

impl From<Unit> for StyleAttributeValue {
    fn from(value: Unit) -> Self {
        Self::Unit(value)
    }
}

impl From<Color> for StyleAttributeValue {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

pub trait FromStyleAttribute: Sized {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self>;
}

impl<T: StyleAttributeEnum> FromStyleAttribute for T {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::String(value) => T::from_str(&value),
            _ => None,
        }
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

impl FromStyleAttribute for Unit {
    fn from_attribute(value: StyleAttributeValue) -> Option<Self> {
        match value {
            StyleAttributeValue::Unit(value) => Some(value),
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

pub trait StyleAttributeEnum: Sized {
    fn from_str(s: &str) -> Option<Self>;
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
}

impl Into<StyleAttributeValue> for TextAlign {
    fn into(self) -> StyleAttributeValue {
        match self {
            Self::Start => "start".into(),
            Self::Center => "center".into(),
            Self::End => "end".into(),
        }
    }
}
