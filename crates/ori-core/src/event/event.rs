use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::proxy::Command;

/// An event that can be sent to a view.
#[derive(Debug)]
pub struct Event {
    pub(crate) event: Box<dyn Any>,
    pub(crate) handled: Arc<AtomicBool>,
    pub(crate) name: &'static str,
}

impl Event {
    pub(crate) fn from_command(command: Command) -> Self {
        Self {
            event: command.command,
            handled: Arc::new(AtomicBool::new(false)),
            name: command.name,
        }
    }

    /// Create a new event.
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            event: Box::new(event),
            handled: Arc::new(AtomicBool::new(false)),
            name: std::any::type_name::<T>(),
        }
    }

    /// Get the name of the event.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Check whether the event is of the given type.
    pub fn is<T: Any>(&self) -> bool {
        self.event.is::<T>()
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
