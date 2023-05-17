mod cursor;
mod keyboard;
mod pointer;
mod sink;
mod task;
mod window;

pub use cursor::*;
pub use keyboard::*;
pub use pointer::*;
pub use sink::*;
pub use task::*;
pub use window::*;

use std::{
    any::{Any, TypeId},
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Clone)]
pub struct Event {
    inner: Arc<dyn Any + Send + Sync>,
    is_handled: Arc<AtomicBool>,
    type_name: &'static str,
}

impl Event {
    pub fn new<T: Any + Send + Sync>(event: T) -> Self {
        Self {
            inner: Arc::new(event),
            is_handled: Arc::new(AtomicBool::new(false)),
            type_name: std::any::type_name::<T>(),
        }
    }

    pub fn is_handled(&self) -> bool {
        self.is_handled.load(Ordering::Acquire)
    }

    pub fn handle(&self) {
        self.is_handled.store(true, Ordering::Release);
    }

    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn type_id(&self) -> TypeId {
        self.inner.as_ref().type_id()
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
            .field("type_name", &self.type_name)
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
