use glam::Vec2;

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
