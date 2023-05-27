use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use glam::Vec2;
use ori_graphics::ImageData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId {
    id: u64,
}

impl WindowId {
    pub const fn main() -> Self {
        Self { id: 0 }
    }

    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RequestRedrawEvent;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowResizeEvent {
    pub size: Vec2,
}

impl WindowResizeEvent {
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SetWindowTitleEvent {
    pub title: String,
    pub window: Option<WindowId>,
}

impl SetWindowTitleEvent {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            window: None,
        }
    }

    pub fn window(mut self, id: WindowId) -> Self {
        self.window = Some(id);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SetWindowIconEvent {
    pub icon: Option<ImageData>,
    pub window: Option<WindowId>,
}

impl SetWindowIconEvent {
    pub const fn new(icon: ImageData) -> Self {
        Self {
            icon: Some(icon),
            window: None,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Self {
        Self {
            icon: Some(ImageData::load(path)),
            window: None,
        }
    }

    pub const fn none() -> Self {
        Self {
            icon: None,
            window: None,
        }
    }

    pub fn window(mut self, id: WindowId) -> Self {
        self.window = Some(id);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CloseWindowEvent {
    pub window: Option<WindowId>,
}

impl CloseWindowEvent {
    pub const fn new() -> Self {
        Self { window: None }
    }

    pub fn window(mut self, id: WindowId) -> Self {
        self.window = Some(id);
        self
    }
}
