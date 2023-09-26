use std::collections::LinkedList;

use crate::window::{WindowDescriptor, WindowId};

use super::UiBuilder;

/// Requests the [`Ui`] might make to the application shell.
pub enum UiRequest<T> {
    /// Render a window.
    Render(WindowId),
    /// Create a window.
    CreateWindow(WindowDescriptor, UiBuilder<T>),
    /// Remove a window.
    RemoveWindow(WindowId),
}

/// A list of [`UiRequest`]s.
pub type UiRequests<T> = LinkedList<UiRequest<T>>;
