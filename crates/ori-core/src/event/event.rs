use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

/// An event that can be sent to a view.
#[derive(Clone, Debug)]
pub struct Event {
    event: Arc<dyn Any>,
    handled: Arc<AtomicBool>,
    name: &'static str,
}

impl Event {
    /// Create a new event.
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            event: Arc::new(event),
            handled: Arc::new(AtomicBool::new(false)),
            name: std::any::type_name::<T>(),
        }
    }

    /// Get the name of the event.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Try to downcast the event to the given type.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.event.as_ref().downcast_ref::<T>()
    }

    /// Returns whether the event has been handled.
    pub fn is_handled(&self) -> bool {
        self.handled.load(Ordering::Relaxed)
    }

    /// Mark the event as handled.
    pub fn handle(&self) {
        self.handled.store(true, Ordering::Relaxed);
    }
}
