use std::{
    any::{Any, TypeId},
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

mod sink;
mod task;

pub use sink::*;
pub use task::*;

/// An event, see [`EventSink`] for more information.
#[derive(Clone)]
pub struct Event {
    inner: Arc<dyn Any + Send + Sync>,
    is_handled: Arc<AtomicBool>,
    type_name: &'static str,
}

impl Event {
    /// Creates a new event.
    pub fn new<T: Any + Send + Sync>(event: T) -> Self {
        Self {
            inner: Arc::new(event),
            is_handled: Arc::new(AtomicBool::new(false)),
            type_name: std::any::type_name::<T>(),
        }
    }

    /// Returns `true` if the event has been handled.
    pub fn is_handled(&self) -> bool {
        self.is_handled.load(Ordering::Acquire)
    }

    /// Marks the event as handled.
    pub fn handle(&self) {
        self.is_handled.store(true, Ordering::Release);
    }

    /// Returns the type name of the event.
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Returns the type id of the event.
    pub fn type_id(&self) -> TypeId {
        self.inner.as_ref().type_id()
    }

    /// Returns `true` if the event is of type `T`.
    pub fn is<T: Any>(&self) -> bool {
        self.inner.as_ref().is::<T>()
    }

    /// Tries to downcast the event to type `T`.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.inner.downcast_ref()
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("is_handled", &self.is_handled())
            .field("type_name", &self.type_name)
            .finish()
    }
}
