use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use glam::Vec2;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Primary,
    Secondary,
    Tertiary,
    Other(u16),
}

#[derive(Clone, Debug, Default)]
pub struct PointerEvent {
    pub position: Vec2,
    pub pressed: bool,
    pub button: Option<PointerButton>,
    pub modifiers: Modifiers,
}

impl PointerEvent {
    pub fn pressed(&self, button: PointerButton) -> bool {
        self.pressed && self.button == Some(button)
    }

    pub fn released(&self, button: PointerButton) -> bool {
        !self.pressed && self.button == Some(button)
    }
}
