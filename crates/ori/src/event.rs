use std::{
    any::Any,
    fmt,
    sync::atomic::{AtomicI64, Ordering},
};

/// An event in an application.
///
/// This is the primary way [`View`](crate::View)s communicate with each other, see
/// [`View::event`](crate::View::event) for more information.
pub struct Event {
    target: Option<Key>,
    item: Option<Box<dyn Any + Send>>,
    name: &'static str,
}

impl Event {
    /// Create a new [`Event`] with over an `item` and an optional `target`.
    pub fn new<T: Any + Send>(item: T, target: impl Into<Option<Key>>) -> Self {
        Self {
            target: target.into(),
            item: Some(Box::new(item)),
            name: std::any::type_name::<T>(),
        }
    }

    /// Get the target of `self`.
    pub fn target(&self) -> Option<Key> {
        self.target
    }

    /// Check if `id` is the target of `self`.
    pub fn is_target(&self, id: Key) -> bool {
        self.target() == Some(id)
    }

    /// Check if the item in `self` is an instance of `T`.
    pub fn is<T: Any + Send>(&self) -> bool {
        self.item.as_ref().is_some_and(|item| item.is::<T>())
    }

    /// Get the item in `self`.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn get<T: Any + Send>(&self) -> Option<&T> {
        self.item.as_ref().and_then(|item| item.downcast_ref())
    }

    /// Get the item in `self` mutably.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn get_mut<T: Any + Send>(&mut self) -> Option<&mut T> {
        self.item.as_mut().and_then(|item| item.downcast_mut())
    }

    /// Get the item in `self` if `key` is the target.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn get_targeted<T: Any + Send>(&self, key: Key) -> Option<&T> {
        self.get().filter(|_| self.is_target(key))
    }

    /// Get the item in `self` mutably if `key` is the target.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn get_mut_targeted<T: Any + Send>(&mut self, key: Key) -> Option<&mut T> {
        let is_target = self.is_target(key);
        self.get_mut().filter(|_| is_target)
    }

    /// Take the item out of `self`.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn take<T: Any + Send>(&mut self) -> Option<T> {
        match self.item.take()?.downcast() {
            Ok(item) => Some(*item),
            Err(item) => {
                self.item = Some(item);
                None
            }
        }
    }

    /// Take the item out of `self` if `key` is the target.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn take_targeted<T: Any + Send>(&mut self, key: Key) -> Option<T> {
        self.is_target(key).then(|| self.take()).flatten()
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("target", &self.target)
            .field("item", &self.name)
            .finish()
    }
}

/// Unique key for targeting [`Event`]s.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    id: i64,
}

impl Key {
    /// Create a [`Key`] from a string.
    pub const fn new(s: &str) -> Self {
        let mut hash = 14695981039346656037u64;
        let prime = 1099511628211u64;

        let mut i = 0;
        while i < s.len() {
            hash = hash.wrapping_mul(prime);
            hash ^= s.as_bytes()[i] as u64;
            i += 1;
        }

        Self {
            id: (hash as i64).abs(),
        }
    }

    /// Create a [`Key`] with a globally incremented id.
    pub fn next() -> Self {
        static NEXT_ID: AtomicI64 = AtomicI64::new(0);
        Self {
            id: NEXT_ID.fetch_sub(1, Ordering::SeqCst),
        }
    }

    /// Create a [`Key`] from a raw [`u64`].
    pub const fn from_u64(id: u64) -> Self {
        assert!(id <= i64::MAX as u64);

        Self { id: id as i64 }
    }
}
