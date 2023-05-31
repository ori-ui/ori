use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash, Hasher},
    mem::MaybeUninit,
};

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
        let mut hasher = seahash::SeaHasher::default();
        Hash::hash(&selectors, &mut hasher);

        Self {
            hash: hasher.finish(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct StyleCacheHasher {
    hash: MaybeUninit<u64>,
}

impl Default for StyleCacheHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleCacheHasher {
    pub const fn new() -> Self {
        Self {
            hash: MaybeUninit::uninit(),
        }
    }
}

impl Hasher for StyleCacheHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.hash = MaybeUninit::new(i);
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        unsafe { self.hash.assume_init() }
    }
}

impl BuildHasher for StyleCacheHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::new()
    }
}

/// A cache of style attributes.
#[derive(Debug, Default)]
pub struct StyleCache {
    attributes: HashMap<u64, Option<(StyleAttribute, StyleSpecificity)>, StyleCacheHasher>,
}

impl Clone for StyleCache {
    fn clone(&self) -> Self {
        Self {
            attributes: self.attributes.clone(),
        }
    }
}

impl StyleCache {
    /// Create a new style cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the style cache.
    pub fn clear(&mut self) {
        tracing::trace!("Clearing style cache");
        self.attributes.clear();
    }

    fn hash(hash: StyleSelectorsHash, key: &str) -> u64 {
        let mut hasher = seahash::SeaHasher::default();
        Hash::hash(key, &mut hasher);

        hasher.finish() ^ hash.hash
    }

    /// Insert a style attribute into the cache.
    pub fn insert(
        &mut self,
        hash: StyleSelectorsHash,
        attribute: StyleAttribute,
        specificity: StyleSpecificity,
    ) {
        let hash = Self::hash(hash, attribute.key());

        #[cfg(debug_assertions)]
        if self.attributes.contains_key(&hash) {
            tracing::warn!(
                "Overwriting style cache entry for {}, this might be a hash collision",
                attribute.key()
            );
        }

        self.attributes.insert(hash, Some((attribute, specificity)));
    }

    /// Insert None into the cache.
    pub fn insert_none(&mut self, hash: StyleSelectorsHash, key: &str) {
        let hash = Self::hash(hash, key);
        self.attributes.insert(hash, None);
    }

    /// Get a style attribute from the cache.
    pub fn get_attribute(
        &self,
        hash: StyleSelectorsHash,
        key: &str,
    ) -> Option<Option<(StyleAttribute, StyleSpecificity)>> {
        let hash = Self::hash(hash, key);
        self.attributes.get(&hash).cloned()
    }
}
