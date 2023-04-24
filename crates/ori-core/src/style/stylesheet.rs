use std::{fmt::Display, fs, io, path::Path, str::FromStr};

use crate::{StyleAttribute, StyleAttributes, StyleCache, StyleSelectors, StyleSpecificity};

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

#[macro_export]
macro_rules! include_stylesheet {
    ($($tt:tt)*) => {
        <$crate::Stylesheet as ::std::str::FromStr>::from_str(include_str!($($tt)*)).unwrap()
    };
}

macro_rules! theme {
    ($name:ident, $folder:literal => $($style:literal),* $(,)?) => {
        pub const $name: &str = concat!(
            $(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/style/", $folder, "/", $style))),*
        );
    };
}

theme!(
    DAY_THEME,
    "day" =>
    "default.css",
    "button.css",
    "checkbox.css",
    "knob.css",
    "scroll.css",
    "text-input.css",
    "text.css",
);

theme!(
    NIGHT_THEME,
    "night" =>
    "default.css",
    "button.css",
    "checkbox.css",
    "knob.css",
    "scroll.css",
    "text-input.css",
    "text.css",
);

/// A style sheet.
///
/// A sheet is a list of [`StyleRule`]s.
/// Rules are applied in the order they are defined.
#[derive(Clone, Debug, Default)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
    pub cache: StyleCache,
}

impl Stylesheet {
    /// Creates a new style sheet.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            cache: StyleCache::new(),
        }
    }

    pub fn day_theme() -> Self {
        Self::from_str(DAY_THEME).expect("Failed to parse day theme, this is a bug with ori")
    }

    pub fn night_theme() -> Self {
        Self::from_str(NIGHT_THEME).expect("Failed to parse night theme, this is a bug with ori")
    }

    /// Adds a [`StyleRule`] to the style sheet.
    pub fn add_rule(&mut self, rule: StyleRule) {
        self.cache.clear();
        self.rules.push(rule);
    }

    /// Extends the style sheet with the given rules.
    pub fn extend(&mut self, rules: impl IntoIterator<Item = StyleRule>) {
        self.cache.clear();
        self.rules.extend(rules);
    }

    pub fn get_attribute(&self, selectors: &StyleSelectors, name: &str) -> Option<StyleAttribute> {
        let (attribute, _) = self.get_attribute_specificity(selectors, name)?;
        Some(attribute.clone())
    }

    pub fn get_attribute_specificity(
        &self,
        selectors: &StyleSelectors,
        name: &str,
    ) -> Option<(StyleAttribute, StyleSpecificity)> {
        if let Some(result) = self.cache.get_attribute(selectors, name) {
            return Some(result);
        }

        let mut specificity = StyleSpecificity::default();
        let mut result = None;

        for rule in self.rules.iter() {
            if selectors.select(&rule.selector) {
                let s = rule.selector.specificity();

                if s < specificity {
                    continue;
                }

                if let Some(attribute) = rule.get_attribute(name) {
                    specificity = s;
                    result = Some((attribute, s));
                }
            }
        }

        let (attribute, specificity) = result?;
        self.cache.insert(selectors, attribute.clone(), specificity);
        Some((attribute.clone(), specificity))
    }

    /// Loads a style sheet from a file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StyleLoadError> {
        let input = fs::read_to_string(path)?;
        Ok(Self::from_str(&input)?)
    }
}

impl IntoIterator for Stylesheet {
    type Item = StyleRule;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rules.into_iter()
    }
}

/// A [`Stylesheet`] rule.
///
/// A rule is a selector and a list of attributes.
/// The attributes are applied to the elements that match the selector.
#[derive(Clone, Debug)]
pub struct StyleRule {
    pub selector: StyleSelectors,
    pub attributes: StyleAttributes,
}

impl StyleRule {
    /// Creates a new style rule from [`StyleSelectors`].
    pub fn new(selector: StyleSelectors) -> Self {
        Self {
            selector,
            attributes: StyleAttributes::new(),
        }
    }

    /// Adds an [`StyleAttribute`] to the rule.
    pub fn add_attribute(&mut self, attribute: StyleAttribute) {
        self.attributes.add(attribute);
    }

    /// Adds a list of [`StyleAttribute`]s to the rule.
    pub fn add_attributes(&mut self, attributes: Vec<StyleAttribute>) {
        self.attributes.extend(attributes);
    }

    /// Gets the value of an attribute.
    pub fn get_attribute(&self, name: &str) -> Option<&StyleAttribute> {
        self.attributes.get(name)
    }
}
