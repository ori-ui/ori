use std::fmt::Display;

use ily_graphics::{Color, TextAlign};
use smallvec::SmallVec;

use crate::{Transition, Unit};

/// A collection of [`Attribute`]s.
#[derive(Clone, Debug, Default)]
pub struct Attributes {
    attributes: SmallVec<[Attribute; 8]>,
}

impl Attributes {
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

    pub fn add(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    pub fn extend(&mut self, attributes: impl IntoIterator<Item = Attribute>) {
        self.attributes.extend(attributes);
    }

    pub fn get(&self, name: &str) -> Option<&Attribute> {
        for attribute in self.attributes.iter() {
            if attribute.key == name {
                return Some(&attribute);
            }
        }

        None
    }

    pub fn get_value<T: FromAttribute>(&self, name: &str) -> Option<T> {
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

    pub fn get_value_and_transition<T: FromAttribute>(
        &self,
        name: &str,
    ) -> Option<(T, Option<Transition>)> {
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

impl IntoIterator for Attributes {
    type Item = Attribute;
    type IntoIter = smallvec::IntoIter<[Self::Item; 8]>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.into_iter()
    }
}

impl<'a> IntoIterator for &'a Attributes {
    type Item = &'a Attribute;
    type IntoIter = std::slice::Iter<'a, Attribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter()
    }
}

impl FromIterator<Attribute> for Attributes {
    fn from_iter<T: IntoIterator<Item = Attribute>>(iter: T) -> Self {
        Self {
            attributes: iter.into_iter().collect(),
        }
    }
}

/// A [`Style`](super::Style) attribute.
///
/// An attribute is a name and a value.
#[derive(Clone, Debug)]
pub struct Attribute {
    /// The attribute key.
    pub key: String,
    /// The attribute value.
    pub value: AttributeValue,
    /// The transition to use when animating the attribute.
    pub transition: Option<Transition>,
}

impl Attribute {
    pub fn new(key: impl Into<String>, value: impl Into<AttributeValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            transition: None,
        }
    }

    pub fn with_transition(
        key: impl Into<String>,
        value: impl Into<AttributeValue>,
        transition: impl Into<Transition>,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            transition: Some(transition.into()),
        }
    }
}

pub trait AttributeBuilder {
    fn attribute(self, key: impl Into<String>) -> Attribute;
}

impl<T: Into<AttributeValue>> AttributeBuilder for T {
    fn attribute(self, key: impl Into<String>) -> Attribute {
        Attribute::new(key, self)
    }
}

impl<T: Into<AttributeValue>, U: Into<Transition>> AttributeBuilder for (T, U) {
    fn attribute(self, key: impl Into<String>) -> Attribute {
        Attribute::with_transition(key, self.0, self.1)
    }
}

/// An ease of use function to create an [`AttributeBuilder`] with a transition.
pub fn trans(
    value: impl Into<AttributeValue>,
    transition: impl Into<Transition>,
) -> impl AttributeBuilder {
    (value, transition)
}

/// A [`Style`](super::Style) attribute value.
#[derive(Clone, Debug)]
pub enum AttributeValue {
    /// A string value, eg. `red`.
    String(String),
    /// A length value, eg. `10px` or `10pt`.
    Unit(Unit),
    /// A color value, eg. `#ff0000`.
    Color(Color),
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => write!(f, "{}", value),
            Self::Unit(value) => write!(f, "{}", value),
            Self::Color(value) => write!(f, "{}", value),
        }
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<f32> for AttributeValue {
    fn from(value: f32) -> Self {
        Self::Unit(Unit::Px(value))
    }
}

impl From<Unit> for AttributeValue {
    fn from(value: Unit) -> Self {
        Self::Unit(value)
    }
}

impl From<Color> for AttributeValue {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

pub trait FromAttribute: Sized {
    fn from_attribute(value: AttributeValue) -> Option<Self>;
}

impl<T: AttributeEnum> FromAttribute for T {
    fn from_attribute(value: AttributeValue) -> Option<Self> {
        match value {
            AttributeValue::String(value) => T::from_str(&value),
            _ => None,
        }
    }
}

impl FromAttribute for String {
    fn from_attribute(value: AttributeValue) -> Option<Self> {
        match value {
            AttributeValue::String(value) => Some(value),
            _ => None,
        }
    }
}

impl FromAttribute for Unit {
    fn from_attribute(value: AttributeValue) -> Option<Self> {
        match value {
            AttributeValue::Unit(value) => Some(value),
            _ => None,
        }
    }
}

impl FromAttribute for Color {
    fn from_attribute(value: AttributeValue) -> Option<Self> {
        match value {
            AttributeValue::Color(value) => Some(value),
            _ => None,
        }
    }
}

pub trait AttributeEnum: Sized {
    fn from_str(s: &str) -> Option<Self>;
}

impl AttributeEnum for TextAlign {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "left" | "start" => Some(Self::Start),
            "center" => Some(Self::Center),
            "right" | "end" => Some(Self::End),
            _ => None,
        }
    }
}

impl Into<AttributeValue> for TextAlign {
    fn into(self) -> AttributeValue {
        match self {
            Self::Start => "start".into(),
            Self::Center => "center".into(),
            Self::End => "end".into(),
        }
    }
}
