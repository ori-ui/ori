use crate::{layout::Size, window::WindowId};

/// Event emitted when a window wants to close.
///
/// After this event is emitted, if it wasn't handled, the window will be closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CloseRequested {
    /// The window that wants to close.
    pub window: WindowId,
}

impl CloseRequested {
    /// Create a new close requested event.
    pub fn new(window: WindowId) -> Self {
        Self { window }
    }
}

/// Event emitted when a window is resized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
