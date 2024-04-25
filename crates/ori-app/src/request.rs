use ori_core::window::{Window, WindowId, WindowUpdate};

use crate::UiBuilder;

/// Requests that an application can make to the platform.
pub enum AppRequest<T> {
    /// Open a new window.
    OpenWindow(Window, UiBuilder<T>),

    /// Close a window.
    CloseWindow(WindowId),

    /// Drag a window.
    DragWindow(WindowId),

    /// Redraw a window.
    RequestRedraw(WindowId),

    /// Update a window.
    UpdateWindow(WindowId, WindowUpdate),

    /// Quit the application.
    Quit,
}
