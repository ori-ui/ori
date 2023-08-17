use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

/// A key used to identify a style property.
pub struct Key<T> {
    name: &'static str,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            marker: PhantomData,
        }
    }
}

impl<T> Copy for Key<T> {}

impl<T> Debug for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key").field("name", &self.name).finish()
    }
}

impl<T> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T> Eq for Key<T> {}

impl<T> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<T> Key<T> {
    /// Create a new key with the given name.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            marker: PhantomData,
        }
    }

    /// Get the name of the key.
    pub const fn name(&self) -> &'static str {
        self.name
    }
}
