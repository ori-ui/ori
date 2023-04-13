use std::{fmt::Display, fs, io, path::Path, str::FromStr};

use crate::{Attribute, Attributes, FromAttribute, StyleSelectors, Transition};

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
    pub fn get_attributes(&self, selector: &StyleSelectors) -> Attributes {
        let mut attributes = Attributes::new();

        for rule in self.rules.iter() {
            if selector.select(&rule.selector) {
                attributes.extend(rule.attributes.clone());
            }
        }

        attributes
    }

    /// Gets the value of an attribute that matches the given selector.
    pub fn get_attribute(&self, selector: &StyleSelectors, name: &str) -> Option<&Attribute> {
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
    pub fn get_value<T: FromAttribute>(&self, selector: &StyleSelectors, name: &str) -> Option<T> {
        for rule in self.rules.iter().rev() {
            if selector.select(&rule.selector) {
                if let Some(value) = rule.get_value(name) {
                    return Some(value);
                }
            }
        }

        None
    }

    /// Gets the value of an attribute that matches the given selector.
    pub fn get_value_and_transition<T: FromAttribute>(
        &self,
        selector: &StyleSelectors,
        name: &str,
    ) -> Option<(T, Option<Transition>)> {
        for rule in self.rules.iter().rev() {
            if selector.select(&rule.selector) {
                if let Some(value) = rule.get_value_and_transition(name) {
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
    pub attributes: Attributes,
}

impl StyleRule {
    /// Creates a new style rule from a [`Selector`].
    pub fn new(selector: StyleSelectors) -> Self {
        Self {
            selector,
            attributes: Attributes::new(),
        }
    }

    /// Adds an [`Attribute`] to the rule.
    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.add(attribute);
    }

    /// Adds a list of [`Attribute`]s to the rule.
    pub fn add_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes.extend(attributes);
    }

    /// Gets the value of an attribute.
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }

    /// Gets the value of an attribute.
    pub fn get_value<T: FromAttribute>(&self, name: &str) -> Option<T> {
        self.attributes.get_value(name)
    }

    /// Gets the value of an attribute.
    pub fn get_value_and_transition<T: FromAttribute>(
        &self,
        name: &str,
    ) -> Option<(T, Option<Transition>)> {
        self.attributes.get_value_and_transition(name)
    }
}
