use std::path::Path;

use glam::Vec2;
use ori_graphics::ImageData;

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
}

impl SetWindowTitleEvent {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SetWindowIconEvent {
    pub icon: Option<ImageData>,
}

impl SetWindowIconEvent {
    pub const fn new(icon: ImageData) -> Self {
        Self { icon: Some(icon) }
    }

    pub fn load(path: impl AsRef<Path>) -> Self {
        Self {
            icon: Some(ImageData::load(path)),
        }
    }

    pub const fn none() -> Self {
        Self { icon: None }
    }
}
