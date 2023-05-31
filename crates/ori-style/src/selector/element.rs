use std::fmt::Display;

use smol_str::SmolStr;

use crate::{StyleClasses, StyleSpec, StyleTags};

/// A style element, (e.g. `div`).
pub type StyleElement = SmolStr;

/// A style selector for a single element, (e.g. `div`, `button:hover`, `image`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleElementSelector {
    /// The element name.
    pub element: Option<StyleElement>,
    /// The classes.
    pub classes: StyleClasses,
    /// The tags
    pub tags: StyleTags,
}

impl StyleElementSelector {
    /// Creates a new [`StyleElementSelector`].
    pub fn new(element: Option<StyleElement>, classes: StyleClasses, tags: StyleTags) -> Self {
        Self {
            element,
            classes,
            tags,
        }
    }

    /// Returns the specificity of the selector, see [`StyleSpec`] for more information.
    pub fn spec(&self) -> StyleSpec {
        StyleSpec {
            class: self.classes.len() as u16 + self.tags.len() as u16,
            tag: self.element.is_some() as u16,
        }
    }

    /// Checks if the selector matches an element in the tree.
    pub fn matches(&self, other: &Self) -> bool {
        if self.element.is_some() && self.element != other.element {
            return false;
        }

        self.classes.matches(&other.classes) && self.tags.matches(&other.tags)
    }
}

impl Display for StyleElementSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(element) = &self.element {
            write!(f, "{}", element)?;
        } else {
            write!(f, "*")?;
        }

        write!(f, "{}", self.classes)?;
        write!(f, "{}", self.tags)?;

        Ok(())
    }
}
