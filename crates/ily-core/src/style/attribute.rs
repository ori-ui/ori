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
            if attribute.name == name {
                return Some(&attribute);
            }
        }

        None
    }

    pub fn get_value<T: FromAttribute>(&self, name: &str) -> Option<T> {
        for attribute in self.attributes.iter().rev() {
            if attribute.name != name {
                continue;
            }

            if let Some(value) = T::from_attribute(attribute.value.clone()) {
                return Some(value);
            }
        }

        None
    }

    pub fn get_value_and_transition<T: FromAttribute>(
        &self,
        name: &str,
    ) -> Option<(T, Option<Transition>)> {
        for attribute in self.attributes.iter().rev() {
            if attribute.name != name {
                continue;
            }

            if let Some(value) = T::from_attribute(attribute.value.clone()) {
                return Some((value, attribute.transition));
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

/// A [`Style`] attribute.
///
/// An attribute is a name and a value.
#[derive(Clone, Debug)]
pub struct Attribute {
    /// The attribute name.
    pub name: String,
    /// The attribute value.
    pub value: AttributeValue,
    /// The transition to use when animating the attribute.
    pub transition: Option<Transition>,
}

/// A [`Style`] attribute value.
#[derive(Clone, Debug)]
pub enum AttributeValue {
    /// A string value, eg. `red`.
    String(String),
    /// A length value, eg. `10px` or `10pt`.
    Length(Unit),
    /// A color value, eg. `#ff0000`.
    Color(Color),
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
            AttributeValue::Length(value) => Some(value),
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
