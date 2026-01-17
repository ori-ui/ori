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
    target: Option<ViewId>,
    item:   Option<Box<dyn Any + Send>>,
    name:   &'static str,
}

impl Event {
    /// Create a new [`Event`] with over an `item` and an optional `target`.
    pub fn new<T: Any + Send>(item: T, target: impl Into<Option<ViewId>>) -> Self {
        Self {
            target: target.into(),
            item:   Some(Box::new(item)),
            name:   std::any::type_name::<T>(),
        }
    }

    /// Get the target of `self`.
    pub fn target(&self) -> Option<ViewId> {
        self.target
    }

    /// Check if `id` is the target of `self`.
    pub fn is_target(&self, id: ViewId) -> bool {
        self.target() == Some(id)
    }

    /// Check if `self` is taken, in which case, propagation should stop.
    pub fn is_taken(&self) -> bool {
        self.item.is_none()
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
    pub fn get_targeted<T: Any + Send>(&self, id: ViewId) -> Option<&T> {
        self.get().filter(|_| self.is_target(id))
    }

    /// Get the item in `self` mutably if `key` is the target.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn get_mut_targeted<T: Any + Send>(&mut self, id: ViewId) -> Option<&mut T> {
        let is_target = self.is_target(id);
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
    pub fn take_targeted<T: Any + Send>(&mut self, id: ViewId) -> Option<T> {
        self.is_target(id).then(|| self.take()).flatten()
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
pub struct ViewId {
    data: i64,
}

impl ViewId {
    /// Create a [`ViewId`] from a string.
    pub const fn new(s: &str) -> Self {
        let mut hash = 14695981039346656037u64;
        let prime = 1099511628211u64;

        let mut i = 0;
        while i < s.len() {
            hash = hash.wrapping_mul(prime);
            hash ^= s.as_bytes()[i] as u64;
            i += 1;
        }

        Self::from_u64((hash as i64).wrapping_abs() as u64)
    }

    /// Create a [`ViewId`] with a globally incremented id.
    pub fn next() -> Self {
        static NEXT_ID: AtomicI64 = AtomicI64::new(0);

        Self {
            data: NEXT_ID.fetch_sub(1, Ordering::SeqCst),
        }
    }

    /// Create a [`ViewId`] from a raw [`u64`].
    #[track_caller]
    pub const fn from_u64(data: u64) -> Self {
        assert!(data <= i64::MAX as u64);

        Self { data: data as i64 }
    }
}
