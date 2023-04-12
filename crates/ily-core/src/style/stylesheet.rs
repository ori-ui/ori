use std::fmt::Display;

use ily_graphics::Color;

use crate::Length;

/// A [`Style`] sheet.
///
/// A sheet is a list of [`StyleRule`]s.
/// Rules are applied in the order they are defined.
#[derive(Clone, Debug, Default)]
pub struct Style {
    pub rules: Vec<StyleRule>,
}

impl Style {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: StyleRule) {
        self.rules.push(rule);
    }

    pub fn get_attributes(&self, selector: &Selector) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        for rule in self.rules.iter() {
            if rule.selectors.contains(selector) {
                attributes.extend(rule.attributes.clone());
            }
        }

        attributes
    }

    pub fn get_attribute(&self, selector: &Selector, name: &str) -> Option<&AttributeValue> {
        for rule in self.rules.iter().rev() {
            if rule.selectors.contains(selector) {
                if let Some(value) = rule.get_attribute(name) {
                    return Some(value);
                }
            }
        }

        None
    }
}

/// A [`Style`] rule.
///
/// A rule is a selector and a list of attributes.
/// The attributes are applied to the elements that match the selector.
#[derive(Clone, Debug)]
pub struct StyleRule {
    pub selectors: Selector,
    pub attributes: Vec<Attribute>,
}

impl StyleRule {
    pub fn new(selectors: Selector) -> Self {
        Self {
            selectors,
            attributes: Vec::new(),
        }
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    pub fn add_attributes(&mut self, attributes: Vec<Attribute>) {
        self.attributes.extend(attributes);
    }

    pub fn get_attribute(&self, name: &str) -> Option<&AttributeValue> {
        for attribute in self.attributes.iter() {
            if attribute.name == name {
                return Some(&attribute.value);
            }
        }

        None
    }
}

/// A [`Style`] selector.
///
/// A selector is a list of classes and an optional element.
#[derive(Clone, Debug, Default)]
pub struct Selector {
    /// The element name.
    ///
    /// This is set by [`View::element`](crate::View::element).
    pub element: Option<String>,
    pub classes: Vec<String>,
}

impl Selector {
    /// Creates a new selector.
    pub fn new(element: Option<String>, classes: Vec<String>) -> Self {
        Self { element, classes }
    }

    /// Returns true if `other` is a subset of `self`.
    pub fn contains(&self, other: &Self) -> bool {
        // check if the element is the same
        //
        // if `other` doesn't have an element, then it's a wildcard
        if other.element.is_some() && self.element != other.element {
            return false;
        }

        // check `self` contains all the classes in `other`
        for class in other.classes.iter() {
            if !self.classes.contains(class) {
                return false;
            }
        }

        true
    }
}

impl Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(element) = &self.element {
            write!(f, "{}", element)?;
        }

        for (i, class) in self.classes.iter().enumerate() {
            if i == 0 && self.element.is_none() {
                write!(f, ".{}", class)?;
            } else {
                write!(f, " .{}", class)?;
            }
        }

        Ok(())
    }
}

/// A [`Style`] attribute.
///
/// An attribute is a name and a value.
#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

/// A [`Style`] attribute value.
#[derive(Clone, Debug)]
pub enum AttributeValue {
    String(String),
    Length(Length),
    Color(Color),
}

impl From<AttributeValue> for Option<String> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::String(value) => Some(value),
            _ => None,
        }
    }
}

impl From<AttributeValue> for Option<Length> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::Length(value) => Some(value),
            _ => None,
        }
    }
}

impl From<AttributeValue> for Option<Color> {
    fn from(value: AttributeValue) -> Self {
        match value {
            AttributeValue::Color(value) => Some(value),
            _ => None,
        }
    }
}
