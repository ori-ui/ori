mod rule;
mod theme;

pub use rule::*;

use std::{fmt::Display, fs, io, path::Path, str::FromStr};

use crate::{StyleAttribute, StyleSpec, StyleTree, StyleheetParseError};

/// An error that can occur when loading a style sheet.
#[derive(Debug)]
pub enum StyleLoadError {
    /// An error occurred while parsing the style sheet.
    Parse(StyleheetParseError),
    /// An error occurred while reading the style sheet.
    Io(io::Error),
}

impl From<StyleheetParseError> for StyleLoadError {
    fn from(error: StyleheetParseError) -> Self {
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

/// Includes a style sheet from a file.
#[macro_export]
macro_rules! include_stylesheet {
    ($($tt:tt)*) => {
        <$crate::Stylesheet as ::std::str::FromStr>::from_str(include_str!($($tt)*)).unwrap()
    };
}

/// A stylesheet is a list of rules.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stylesheet {
    pub rules: Vec<StylesheetRule>,
}

impl Stylesheet {
    /// Create a new stylesheet.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Loads the default day theme.
    pub fn day_theme() -> Self {
        Self::from_str(theme::DAY_THEME).unwrap()
    }

    /// Loads the default night theme.
    pub fn night_theme() -> Self {
        Self::from_str(theme::NIGHT_THEME).unwrap()
    }

    /// Add a rule to the stylesheet.
    pub fn add_rule(&mut self, rule: StylesheetRule) {
        self.rules.push(rule);
    }

    /// Extend the stylesheet with a list of rules.
    pub fn extend(&mut self, rules: impl IntoIterator<Item = StylesheetRule>) {
        self.rules.extend(rules);
    }

    /// Get an attribute from the style sheet.
    pub fn get_attribute(&self, tree: &StyleTree, name: &str) -> Option<StyleAttribute> {
        let (attribute, _) = self.get_attribute_specificity(tree, name)?;
        Some(attribute)
    }

    /// Get and attribute and its specificity from the style sheet.
    pub fn get_attribute_specificity(
        &self,
        tree: &StyleTree,
        name: &str,
    ) -> Option<(StyleAttribute, StyleSpec)> {
        let mut specificity = StyleSpec::default();
        let mut result = None;

        for rule in self.rules.iter() {
            let Some(selector) = rule.get_match(tree) else {
                continue;
            };

            let selector = &rule.selectors[selector];
            let spec = selector.spec();

            if spec < specificity {
                continue;
            }

            if let Some(attribute) = rule.get_attribute(name) {
                specificity = spec;
                result = Some((attribute, spec));
            }
        }

        result.map(|(attribute, _)| (attribute.clone(), specificity))
    }

    /// Loads a style sheet from a file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StyleLoadError> {
        let input = fs::read_to_string(path)?;
        Ok(Self::from_str(&input)?)
    }
}

impl IntoIterator for Stylesheet {
    type Item = StylesheetRule;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}

impl Display for Stylesheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rule in &self.rules {
            writeln!(f, "{}", rule)?;
        }

        Ok(())
    }
}
