use std::{mem, sync::Mutex};

use glam::Vec2;
use ori_reactive::Scope;

use crate::{Node, Window, WindowId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RequestRedrawEvent;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowResizedEvent {
    pub size: Vec2,
}

impl WindowResizedEvent {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowClosedEvent {
    pub window: WindowId,
}

impl WindowClosedEvent {
    pub fn new(id: WindowId) -> Self {
        Self { window: id }
    }
}

pub struct OpenWindow {
    window: Window,
    ui: Mutex<Box<dyn FnMut(Scope) -> Node + Send + Sync>>,
}

impl OpenWindow {
    pub fn new(window: Window, ui: impl FnMut(Scope) -> Node + Send + Sync + 'static) -> Self {
        Self {
            window,
            ui: Mutex::new(Box::new(ui)),
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn take_ui(&self) -> Box<dyn FnMut(Scope) -> Node + Send + Sync> {
        mem::replace(&mut self.ui.lock().unwrap(), Box::new(|_| Node::empty()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CloseWindow {
    pub window: Option<WindowId>,
}

impl CloseWindow {
    pub const fn new() -> Self {
        Self { window: None }
    }

    pub fn window(window: WindowId) -> Self {
        Self {
            window: Some(window),
        }
    }
}
