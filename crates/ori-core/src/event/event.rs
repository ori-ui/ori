use std::{any::Any, cell::Cell};

use crate::command::Command;

/// An event that can be sent to a view.
#[derive(Debug)]
pub struct Event {
    event: Box<dyn Any>,
    handled: Cell<bool>,
    propagate: bool,
    name: &'static str,
}

impl Event {
    /// Create a new event from a command.
    pub fn from_command(command: Command) -> Self {
        Self {
            event: command.command,
            handled: Cell::new(false),
            propagate: true,
            name: command.name,
        }
    }

    /// Create a new event.
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            event: Box::new(event),
            handled: Cell::new(false),
            propagate: true,
            name: std::any::type_name::<T>(),
        }
    }

    /// Create a new event with a name.
    pub fn new_with_name<T: Any>(event: T, name: &'static str) -> Self {
        Self {
            event: Box::new(event),
            handled: Cell::new(false),
            propagate: true,
            name,
        }
    }

    /// Create a new non-propagating event.
    pub fn new_non_propagating<T: Any>(event: T) -> Self {
        Self {
            event: Box::new(event),
            handled: Cell::new(false),
            propagate: false,
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
        if self.is::<T>() {
            self.event.downcast_ref::<T>()
        } else {
            None
        }
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

    /// Returns whether the event should propagate.
    pub fn should_propagate(&self) -> bool {
        self.propagate
    }
}

impl From<Command> for Event {
    fn from(command: Command) -> Self {
        Self::from_command(command)
    }
}
