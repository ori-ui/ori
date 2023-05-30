use std::{
    hash::{Hash, Hasher},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum WindowIdInner {
    Main,
    Hash(u64),
    Increment(u64),
}

/// A opaque identifier for a window.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId {
    inner: WindowIdInner,
}

impl WindowId {
    const fn from_inner(inner: WindowIdInner) -> Self {
        Self { inner }
    }

    /// Returns the id of the main window.
    pub const fn main() -> Self {
        Self::from_inner(WindowIdInner::Main)
    }

    /// Returns a window id from a hash.
    ///
    /// The hash is computed using [`seahash`].
    pub fn from_hash(hash: &impl Hash) -> Self {
        let mut hasher = seahash::SeaHasher::new();

        hash.hash(&mut hasher);

        Self::from_inner(WindowIdInner::Hash(hasher.finish()))
    }

    /// Returns a window id from an incrementing counter.
    pub fn incremental() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        Self::from_inner(WindowIdInner::Increment(id))
    }
}
