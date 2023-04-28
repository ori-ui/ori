mod cursor;
mod keyboard;
mod pointer;
mod sink;
mod window;

pub use cursor::*;
pub use keyboard::*;
pub use pointer::*;
pub use sink::*;
pub use window::*;

use std::{
    any::Any,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::SendSync;

#[derive(Clone)]
pub struct Event {
    #[cfg(feature = "multithread")]
    inner: Arc<dyn Any + Send + Sync>,
    #[cfg(not(feature = "multithread"))]
    inner: Arc<dyn Any>,
    is_handled: Arc<AtomicBool>,
}

impl Event {
    pub fn new<T: Any + SendSync>(event: T) -> Self {
        Self {
            inner: Arc::new(event),
            is_handled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_handled(&self) -> bool {
        self.is_handled.load(Ordering::Acquire)
    }

    pub fn handle(&self) {
        self.is_handled.store(true, Ordering::Release);
    }

    pub fn is<T: Any>(&self) -> bool {
        self.inner.as_ref().is::<T>()
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.inner.downcast_ref()
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("is_handled", &self.is_handled())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
