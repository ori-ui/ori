use fxhash::FxHashMap;
use smol_str::SmolStr;

use crate::{Lock, Lockable, Shared, StyleAttribute, StyleSelectors, StyleSpecificity};

type RuleAttributes = FxHashMap<SmolStr, (StyleAttribute, StyleSpecificity)>;

#[derive(Clone, Debug, Default)]
pub struct StyleCache {
    attributes: Shared<Lock<FxHashMap<StyleSelectors, RuleAttributes>>>,
}

impl StyleCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) {
        let mut attributes = self.attributes.lock_mut();
        attributes.clear();
    }

    pub fn insert(
        &self,
        selectors: &StyleSelectors,
        attribute: StyleAttribute,
        specificity: StyleSpecificity,
    ) {
        let mut attributes = self.attributes.lock_mut();
        let attributes = attributes.entry(selectors.clone()).or_default();
        attributes.insert(attribute.key.clone(), (attribute, specificity));
    }

    pub fn get_attribute(
        &self,
        selectors: &StyleSelectors,
        key: &str,
    ) -> Option<(StyleAttribute, StyleSpecificity)> {
        let attributes = self.attributes.lock_mut();
        let attributes = attributes.get(selectors)?;
        let attribute = attributes.get(key)?;
        Some(attribute.clone())
    }
}
