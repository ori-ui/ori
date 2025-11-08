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
    item: Option<Box<dyn Any + Send>>,
    name: &'static str,
}

impl Event {
    /// Create a new [`Event`] with over an `item` and an optional `target`.
    pub fn new<T: Any + Send>(
        item: T,
        target: impl Into<Option<ViewId>>,
    ) -> Self {
        Self {
            target: target.into(),
            item: Some(Box::new(item)),
            name: std::any::type_name::<T>(),
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

    /// Get the item in `self` if `id` is the target.
    ///
    /// Returns [`None`] if the item is not an instance of `T` or has been taken.
    pub fn get_targeted<T: Any + Send>(&self, id: ViewId) -> Option<&T> {
        self.get().filter(|_| self.is_target(id))
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

    /// Take the item out of `self` if `id` is the target.
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

/// Unique identifier for a [`View`](crate::View).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewId {
    id: i64,
}

impl Default for ViewId {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewId {
    /// Create a [`ViewId`] with a globally incremented id.
    pub fn new() -> Self {
        static NEXT_ID: AtomicI64 = AtomicI64::new(0);
        Self {
            id: NEXT_ID.fetch_sub(1, Ordering::SeqCst),
        }
    }

    /// Create a [`ViewId`] from a raw [`u64`].
    pub const fn from_u64(id: u64) -> Self {
        assert!(id <= i64::MAX as u64);

        Self { id: id as i64 }
    }
}
