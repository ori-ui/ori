use std::{
    any::{Any, TypeId},
    cell::Cell,
};

use crate::command::Command;

/// An event that can be sent to a view.
#[derive(Debug)]
pub struct Event {
    // SAFETY: This is always the type ID of the type of the `Self::event`.
    type_id: TypeId,
    event: Box<dyn Any>,
    handled: Cell<bool>,
    propagate: bool,
    name: &'static str,
}

impl Event {
    /// Create a new event from a command.
    pub fn from_command(command: Command) -> Self {
        Self {
            type_id: command.command.as_ref().type_id(),
            event: command.command,
            handled: Cell::new(false),
            propagate: true,
            name: command.name,
        }
    }

    /// Create a new event.
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            event: Box::new(event),
            handled: Cell::new(false),
            propagate: true,
            name: std::any::type_name::<T>(),
        }
    }

    /// Create a new event with a name.
    pub fn new_with_name<T: Any>(event: T, name: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            event: Box::new(event),
            handled: Cell::new(false),
            propagate: true,
            name,
        }
    }

    /// Create a new non-propagating event.
    pub fn new_non_propagating<T: Any>(event: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
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
        self.type_id == TypeId::of::<T>()
    }

    /// Try to downcast the event to the given type.
    pub fn get<T: Any>(&self) -> Option<&T> {
        if self.is::<T>() {
            // SAFETY: We just checked that the type is correct.
            //
            // We need unsafe here because <dyn Any>::downcast_ref does a dynamic call to
            // check the type, which is slow... This function is called a lot, so we want
            // to avoid that.
            unsafe { Some(&*(self.event.as_ref() as *const _ as *const T)) }
        } else {
            None
        }
    }

    /// Try to downcast the event to the given type.
    pub fn take<T: Any>(self) -> Result<T, Event> {
        if self.is::<T>() {
            Ok(*self.event.downcast::<T>().unwrap())
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
