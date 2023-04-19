use std::{any::Any, fmt::Debug, sync::Arc};

use crate::Event;

pub trait EventSender: 'static {
    fn send_event(&self, event: Event);
}

#[derive(Clone)]
pub struct EventSink {
    sender: Arc<dyn EventSender>,
}

impl EventSink {
    pub fn new(sender: impl EventSender) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }

    pub fn send(&self, event: impl Any) {
        self.sender.send_event(Event::new(event));
    }
}

impl Debug for EventSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSink").finish()
    }
}
