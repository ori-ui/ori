use std::{
    any::Any,
    fmt::Debug,
    sync::{mpsc::Sender, Arc},
};

use parking_lot::Mutex;

use crate::Event;

/// An event sender, that can send events to the application.
///
/// This is usually implemented by the application shell and
/// usually won't be implemented by the user.
pub trait EventEmitter: Any + Send {
    fn send_event(&mut self, event: Event);
}

impl EventEmitter for () {
    fn send_event(&mut self, _: Event) {}
}

impl EventEmitter for Sender<Event> {
    fn send_event(&mut self, event: Event) {
        let _ = self.send(event);
    }
}

/// An event sink, that can send events to the application.
#[derive(Clone)]
pub struct EventSink {
    emitter: Arc<Mutex<dyn EventEmitter>>,
}

impl EventSink {
    /// Creates a dummy event sink, that does nothing.
    pub fn dummy() -> Self {
        Self::new(())
    }

    /// Creates a new event sink from an [`EventEmitter`].
    pub fn new(sender: impl EventEmitter) -> Self {
        Self {
            emitter: Arc::new(Mutex::new(sender)),
        }
    }

    /// Tries to downcast the [`EventEmitter`] to a specific type.
    ///
    /// **Note** this will lock the [`EventSink`] for the duration of the
    /// closure, meaning that using the event sink inside the closure will
    /// result in a deadlock.
    pub fn downcast_with<T: EventEmitter>(&self, f: impl FnOnce(&mut T)) -> Result<(), ()> {
        let emitter = &*self.emitter.lock();

        if emitter.type_id() == std::any::TypeId::of::<T>() {
            let mut emitter = unsafe { &mut *(emitter as *const dyn EventEmitter as *mut T) };
            f(&mut emitter);

            Ok(())
        } else {
            Err(())
        }
    }

    /// Sends an event to the application.
    pub fn emit(&self, event: impl Any + Send + Sync) {
        self.emitter.lock().send_event(Event::new(event));
    }
}

impl Debug for EventSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSink").finish()
    }
}
