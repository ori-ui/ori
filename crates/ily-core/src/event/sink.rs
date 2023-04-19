use std::{any::Any, fmt::Debug};

use crate::{Event, Lock, Lockable, SendSync, Sendable, Shared};

/// An event sender, that can send events to the application.
///
/// This is usually implemented by the application shell and
/// should not be implemented by the user.
pub trait EventSender: Sendable + 'static {
    fn send_event(&mut self, event: Event);
}

/// An event sink, that can send events to the application.
#[derive(Clone)]
pub struct EventSink {
    sender: Shared<Lock<dyn EventSender>>,
}

impl EventSink {
    /// Creates a new event sink from an [`EventSender`].
    pub fn new(sender: impl EventSender) -> Self {
        Self {
            sender: Shared::new(Lock::new(sender)),
        }
    }

    /// Sends an event to the application.
    pub fn send(&self, event: impl Any + SendSync) {
        self.sender.lock_mut().send_event(Event::new(event));
    }
}

impl Debug for EventSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSink").finish()
    }
}
