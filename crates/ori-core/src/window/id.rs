use std::{
    hash::Hash,
    sync::atomic::{AtomicU64, Ordering},
};

/// A opaque identifier for a window.
///
/// They are created by the [`WindowId::new`] method, and should usually not be created manually.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId {
    id: u64,
}

impl Default for WindowId {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowId {
    /// Creates a new window id.
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        }
    }
}
