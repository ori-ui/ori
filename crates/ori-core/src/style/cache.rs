use nohash_hasher::IntMap;

use crate::{StyleAttribute, StyleSelectors, StyleSpecificity};

/// A hash of [`StyleSelectors`](StyleSelectors).
///
/// This is used to cache the style of a element.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StyleSelectorsHash {
    hash: u64,
}

impl StyleSelectorsHash {
    pub fn new(selectors: &StyleSelectors) -> Self {
        use std::hash::{Hash, Hasher};

        let mut hasher = seahash::SeaHasher::default();
        Hash::hash(&selectors, &mut hasher);

        Self {
            hash: hasher.finish(),
        }
    }
}

#[derive(Debug, Default)]
pub struct StyleCache {
    attributes: IntMap<u64, Option<(StyleAttribute, StyleSpecificity)>>,
}

impl Clone for StyleCache {
    fn clone(&self) -> Self {
        Self {
            attributes: self.attributes.clone(),
        }
    }
}

impl StyleCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        tracing::trace!("Clearing style cache");
        self.attributes.clear();
    }

    fn hash(hash: StyleSelectorsHash, key: &str) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = seahash::SeaHasher::default();
        Hash::hash(key, &mut hasher);

        hasher.finish() ^ hash.hash
    }

    pub fn insert(
        &mut self,
        hash: StyleSelectorsHash,
        attribute: StyleAttribute,
        specificity: StyleSpecificity,
    ) {
        let hash = Self::hash(hash, attribute.key());

        #[cfg(debug_assertions)]
        {
            if self.attributes.contains_key(&hash) {
                tracing::warn!(
                    "Overwriting style cache entry for {}, this might be a hash collision",
                    attribute.key()
                );
            }
        }

        self.attributes.insert(hash, Some((attribute, specificity)));
    }

    pub fn insert_none(&mut self, hash: StyleSelectorsHash, key: &str) {
        let hash = Self::hash(hash, &key);

        self.attributes.insert(hash, None);
    }

    pub fn get_attribute(
        &self,
        hash: StyleSelectorsHash,
        key: &str,
    ) -> Option<Option<(StyleAttribute, StyleSpecificity)>> {
        let hash = Self::hash(hash, key);

        match self.attributes.get(&hash) {
            Some(result) => Some(result.clone()),
            None => None,
        }
    }
}
