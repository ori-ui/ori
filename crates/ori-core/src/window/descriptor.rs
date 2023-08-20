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
            id: WindowId::next(),
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
