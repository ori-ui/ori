use crate::{layout::Size, window::WindowId};

/// Event emitted when a window wants to close.
///
/// After this event is emitted, if it wasn't handled, the window will be closed.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct WindowCloseRequested {
    /// The window that wants to close.
    pub window: WindowId,
}

/// Event emitted when a window is resized.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct WindowResized {
    /// The window that was resized.
    pub window: WindowId,

    /// The new width of the window.
    pub width: u32,

    /// The new height of the window.
    pub height: u32,
}

impl WindowResized {
    /// Get the new size of the window.
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }
}

/// Event emitted when a window is scaled.
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct WindowScaled {
    /// The window that was scaled.
    pub window: WindowId,

    /// The new scale factor of the window.
    pub scale_factor: f32,
}

/// Event emitted when a window is maximized.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct WindowMaximized {
    /// The window that was maximized.
    pub window: WindowId,

    /// Whether the window is maximized or not.
    pub maximized: bool,
}
