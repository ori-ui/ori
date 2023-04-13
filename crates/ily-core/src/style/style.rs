use std::{fmt::Display, fs, io, path::Path, str::FromStr};

use ily_graphics::Color;

use crate::{Length, StyleSelectors};

use super::parse::StyleParseError;

/// An error that can occur when loading a style sheet.
#[derive(Debug)]
pub enum StyleLoadError {
    /// An error occurred while parsing the style sheet.
    Parse(StyleParseError),
    /// An error occurred while reading the style sheet.
    Io(io::Error),
}

impl From<StyleParseError> for StyleLoadError {
    fn from(error: StyleParseError) -> Self {
        Self::Parse(error)
    }
}

impl From<io::Error> for StyleLoadError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl Display for StyleLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "Parse error: {}", error),
            Self::Io(error) => write!(f, "IO error: {}", error),
        }
    }
}

/// A style sheet.
///
/// A sheet is a list of [`StyleRule`]s.
/// Rules are applied in the order they are defined.
#[derive(Clone, Debug, Default)]
pub struct Style {
    pub rules: Vec<StyleRule>,
}

impl Style {
    /// Creates a new style sheet.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a [`StyleRule`] to the style sheet.
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }

    /// Extends the style sheet with the given rules.
    pub fn extend(&mut self, rules: impl IntoIterator<Item = StyleRule>) {
        self.rules.extend(rules);
    }

    /// Gets the attributes that match the given selector.
    pub fn get_attributes(&self, selector: &StyleSelectors) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        for rule in self.rules.iter() {
            if selector.select(&rule.selector) {
                attributes.extend(rule.attributes.clone());
            }
        }

        attributes
    }

    /// Gets the value of an attribute that matches the given selector.
    pub fn get_attribute(&self, selector: &StyleSelectors, name: &str) -> Option<&AttributeValue> {
        for rule in self.rules.iter().rev() {
            if selector.select(&rule.selector) {
                if let Some(value) = rule.get_attribute(name) {
                    return Some(value);
                }
            }
        }

        None
    }

    /// Gets the value of an attribute that matches the given selector.
    pub fn get_value<T>(&self, selector: &StyleSelectors, name: &str) -> Option<T>
    where
        Option<T>: From<AttributeValue>,
    {
        for rule in self.rules.iter().rev() {
            if selector.select(&rule.selector) {
                if let Some(value) = rule.get_value(name) {
                    return Some(value);
                }
            }
        }

        None
    }

    /// Loads a style sheet from a file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StyleLoadError> {
        let input = fs::read_to_string(path)?;
        Ok(Self::from_str(&input)?)
    }
}

impl IntoIterator for Style {
    type Item = StyleRule;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}

/// A [`Style`] rule.
///
/// A rule is a selector and a list of attributes.
/// The attributes are applied to the elements that match the selector.
#[derive(Clone, Debug)]
pub struct StyleRule {
    pub selector: StyleSelectors,
    pub attributes: Vec<Attribute>,
}

impl StyleRule {
    /// Creates a new style rule from a [`Selector`].
    pub fn new(selector: StyleSelectors) -> Self {
        Self {
            selector,
            attributes: Vec::new(),
        }
    }

    /// Adds an [`Attribute`] to the rule.
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    /// Adds a list of [`Attribute`]s to the rule.
    pub fn add_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes.extend(attributes);
    }

    /// Gets the value of an attribute.
    pub fn get_attribute(&self, name: &str) -> Option<&AttributeValue> {
        for attribute in self.attributes.iter() {
            if attribute.name == name {
                return Some(&attribute.value);
            }
        }

        None
    }

    /// Gets the value of an attribute.
    pub fn get_value<T>(&self, name: &str) -> Option<T>
    where
        Option<T>: From<AttributeValue>,
    {
        for attribute in self.attributes.iter() {
            if attribute.name != name {
                continue;
            }

            if let Some(value) = Option::<T>::from(attribute.value.clone()) {
                return Some(value);
            }
        }

        None
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
}

/// A [`Style`] attribute value.
#[derive(Clone, Debug)]
pub enum AttributeValue {
    /// A string value, eg. `red`.
    String(String),
    /// A length value, eg. `10px` or `10pt`.
    Length(Length),
    /// A color value, eg. `#ff0000`.
    Color(Color),
}

impl From<AttributeValue> for Option<String> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::String(value) => Some(value),
            _ => {
                tracing::warn!("attribute is not a string");

                None
            }
        }
    }
}

impl From<AttributeValue> for Option<Length> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::Length(value) => Some(value),
            _ => {
                tracing::warn!("attribute is not a length");

                None
            }
        }
    }
}

impl From<AttributeValue> for Option<Color> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::Color(value) => Some(value),
            _ => {
                tracing::warn!("attribute is not a color");

                None
            }
        }
    }
}
