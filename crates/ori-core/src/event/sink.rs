use std::{any::Any, fmt::Debug};

use crate::{Event, Lock, Lockable, SendSync, Sendable, Shared};

/// An event sender, that can send events to the application.
///
/// This is usually implemented by the application shell and
/// should not be implemented by the user.
pub trait EventEmitter: Sendable + 'static {
    fn send_event(&mut self, event: Event);
}

impl EventEmitter for () {
    fn send_event(&mut self, _: Event) {}
}

/// An event sink, that can send events to the application.
#[derive(Clone)]
pub struct EventSink {
    emitter: Shared<Lock<dyn EventEmitter>>,
}

impl EventSink {
    /// Creates a dummy event sink, that does nothing.
    pub fn dummy() -> Self {
        Self::new(())
    }

    /// Creates a new event sink from an [`EventSender`].
    pub fn new(sender: impl EventEmitter) -> Self {
        Self {
            emitter: Shared::new(Lock::new(sender)),
        }
    }

    /// Sends an event to the application.
    pub fn emit(&self, event: impl Any + SendSync) {
        self.emitter.lock_mut().send_event(Event::new(event));
    }
}

impl Debug for EventSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSink").finish()
    }
}
