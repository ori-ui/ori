use std::{any::Any, cell::Cell};

use crate::command::Command;

/// An event that can be sent to a view.
#[derive(Debug)]
pub struct Event {
    event: Box<dyn Any>,
    handled: Cell<bool>,
    name: &'static str,
}

impl Event {
    pub(crate) fn from_command(command: Command) -> Self {
        Self {
            event: command.command,
            handled: Cell::new(false),
            name: command.name,
        }
    }

    /// Create a new event.
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            event: Box::new(event),
            handled: Cell::new(false),
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

    /// Try to downcast the event to the given type.
    pub fn take<T: Any>(self) -> Result<T, Event> {
        if self.is::<T>() {
            let event = self.event.downcast::<T>().unwrap();
            Ok(*event)
        } else {
            Err(self)
        }
    }

    /// Returns whether the event has been handled.
    pub fn is_handled(&self) -> bool {
        self.handled.get()
    }

    /// Set whether the event has been handled.
    pub fn set_handled(&self, handled: bool) {
        self.handled.set(handled);
    }

    /// Mark the event as handled.
    pub fn handle(&self) {
        self.set_handled(true);
    }
}
