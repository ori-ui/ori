use crate::window::WindowId;

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
