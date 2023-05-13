use dashmap::DashMap;
use smol_str::SmolStr;

use crate::{StyleAttribute, StyleSelectors, StyleSpecificity};

/// A hash of [`StyleSelectors`](StyleSelectors).
///
/// This is used to cache the style of a node.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StyleSelectorsHash {
    hash: u64,
}

impl StyleSelectorsHash {
    pub fn new(selectors: &StyleSelectors) -> Self {
        use std::hash::{Hash, Hasher};

        let mut hasher = fxhash::FxHasher::default();
        Hash::hash(&selectors, &mut hasher);

        Self {
            hash: hasher.finish(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StyleCache {
    attributes: DashMap<
        (StyleSelectorsHash, SmolStr),
        Option<(StyleAttribute, StyleSpecificity)>,
        fxhash::FxBuildHasher,
    >,
}

impl StyleCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) {
        self.attributes.clear();
    }

    pub fn insert(
        &self,
        hash: StyleSelectorsHash,
        attribute: StyleAttribute,
        specificity: StyleSpecificity,
    ) {
        self.attributes.insert(
            (hash, attribute.key().clone()),
            Some((attribute, specificity)),
        );
    }

    pub fn insert_none(&self, hash: StyleSelectorsHash, key: SmolStr) {
        self.attributes.insert((hash, key), None);
    }

    pub fn get_attribute(
        &self,
        hash: StyleSelectorsHash,
        key: &str,
    ) -> Option<Option<(StyleAttribute, StyleSpecificity)>> {
        match self.attributes.get(&(hash, key.into())) {
            Some(result) => Some(result.clone()),
            None => None,
        }
    }
}
