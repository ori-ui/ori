use ori_core::window::{WindowDescriptor, WindowId};

use crate::UiBuilder;

/// Requests that an application can make to the platform.
pub enum AppRequest<T> {
    /// Open a new window.
    OpenWindow(WindowDescriptor, UiBuilder<T>),
    /// Close a window.
    CloseWindow(WindowId),
    /// Quit the application.
    Quit,
}
