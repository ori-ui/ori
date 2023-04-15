mod keyboard;
mod pointer;

pub use keyboard::*;
pub use pointer::*;

use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub struct Event {
    inner: Arc<dyn Any>,
    is_handled: AtomicBool,
}

impl Event {
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            inner: Arc::new(event),
            is_handled: AtomicBool::new(false),
        }
    }

    pub fn is_handled(&self) -> bool {
        self.is_handled.load(Ordering::Acquire)
    }

    pub fn handle(&self) {
        self.is_handled.store(true, Ordering::Release);
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.inner.downcast_ref()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
