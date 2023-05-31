use std::{fmt::Display, sync::Arc};

use crate::{StyleAttribute, StyleSelector, StyleTree};

/// A stylesheet rule.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StylesheetRule {
    /// The selectors that match this rule.
    pub selectors: Arc<[StyleSelector]>,
    /// The attributes that are set by this rule.
    pub attributes: Arc<[StyleAttribute]>,
}

impl StylesheetRule {
    pub fn get_match(&self, selector: &StyleTree) -> Option<usize> {
        self.selectors.iter().position(|s| s.matches(selector))
    }

    pub fn get_attribute(&self, key: &str) -> Option<&StyleAttribute> {
        self.attributes.iter().rev().find(|a| a.key() == key)
    }
}

impl Display for StylesheetRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let selectors = self
            .selectors
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(f, "{} {{", selectors)?;

        for attribute in &*self.attributes {
            writeln!(f, "    {}: {};", attribute.key(), attribute.value())?;
        }

        writeln!(f, "}}")
    }
}
