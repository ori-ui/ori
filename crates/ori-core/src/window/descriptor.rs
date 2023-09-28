use crate::image::Image;

use super::WindowId;

/// A descriptor for a window.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowDescriptor {
    /// The unique identifier of the window.
    pub id: WindowId,
    /// The title of the window.
    pub title: String,
    /// The icon of the window.
    pub icon: Option<Image>,
    /// The width of the window.
    pub width: u32,
    /// The height of the window.
    pub height: u32,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// Whether the window is decorated.
    pub decorated: bool,
    /// Whether the window is transparent.
    pub transparent: bool,
    /// Whether the window is maximized.
    pub maximized: bool,
    /// Whether the window is visible.
    pub visible: bool,
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            id: WindowId::new(),
            title: String::from("Ori App"),
            icon: None,
            width: 800,
            height: 600,
            resizable: true,
            decorated: true,
            transparent: false,
            maximized: false,
            visible: true,
        }
    }
}

impl WindowDescriptor {
    /// Create a new window descriptor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the unique identifier of the window.
    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the icon of the window.
    pub fn icon(mut self, icon: impl Into<Option<Image>>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Set the size of the window.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the width of the window.
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the height of the window.
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set whether the window is decorated.
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    /// Set whether the window is transparent.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Set whether the window is maximized.
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Set whether the window is visible.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}
