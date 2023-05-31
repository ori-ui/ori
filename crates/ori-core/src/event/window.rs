use std::{mem, sync::Mutex};

use glam::Vec2;
use ori_reactive::Scope;

use crate::{Node, Window, WindowId};

/// An event that requests a redraw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RequestRedrawEvent;

/// An event that's emitted when the window is resized.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowResizedEvent {
    pub size: Vec2,
}

impl WindowResizedEvent {
    /// Create a new window resized event.
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}

/// An event that's emitted when the window is closed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowClosedEvent {
    /// The id of the window that was closed.
    pub window: WindowId,
}

impl WindowClosedEvent {
    /// Create a new window closed event.
    pub fn new(id: WindowId) -> Self {
        Self { window: id }
    }
}

/// An event that opens a new window, when emitted.
pub struct OpenWindow {
    window: Window,
    ui: Mutex<Box<dyn FnMut(Scope) -> Node + Send + Sync>>,
}

impl OpenWindow {
    /// Create a new open window event.
    pub fn new(window: Window, ui: impl FnMut(Scope) -> Node + Send + Sync + 'static) -> Self {
        Self {
            window,
            ui: Mutex::new(Box::new(ui)),
        }
    }

    /// Get the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Takes the ui function, replacing it with an empty one.
    pub fn take_ui(&self) -> Box<dyn FnMut(Scope) -> Node + Send + Sync> {
        mem::replace(&mut self.ui.lock().unwrap(), Box::new(|_| Node::empty()))
    }
}

/// An event that closes a window, when emitted.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CloseWindow {
    /// The id of the window to close.
    pub window: Option<WindowId>,
}

impl CloseWindow {
    /// Create a new close window event, that will close the current window.
    pub const fn new() -> Self {
        Self { window: None }
    }

    /// Create a new close window event, that will close the given `window`.
    pub fn window(window: WindowId) -> Self {
        Self {
            window: Some(window),
        }
    }
}
