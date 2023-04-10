use std::{any::Any, sync::Arc};

use glam::Vec2;

pub struct Event {
    inner: Arc<dyn Any>,
}

impl Event {
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            inner: Arc::new(event),
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Primary,
    Secondary,
    Tertiary,
    Other(u16),
}

#[derive(Clone, Debug)]
pub struct PointerPress {
    pub position: Vec2,
    pub pressed: bool,
    pub button: PointerButton,
    pub modifiers: Modifiers,
}
