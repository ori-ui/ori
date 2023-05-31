use ori_reactive::{Scope, Signal};

use crate::{CloseWindow, Node, OpenWindow, Window, WindowId};

/// Extension trait for [`Scope`] that provides window related methods.
pub trait ScopeWindowExt {
    /// Returns a signal that with the current window.
    fn window(self) -> Signal<Window>;

    /// Open a new window.
    fn open_window(
        self,
        window: Window,
        ui: impl FnMut(Scope) -> Node + Send + Sync + 'static,
    ) -> WindowId;

    /// Close the current window.
    ///
    /// If you want to close a specific window, emit a [`CloseWindow`] event instead.
    fn close_window(self);
}

impl ScopeWindowExt for Scope {
    fn window(self) -> Signal<Window> {
        self.context::<Signal<Window>>()
    }

    fn open_window(
        self,
        window: Window,
        ui: impl FnMut(Scope) -> Node + Send + Sync + 'static,
    ) -> WindowId {
        let id = window.id();
        self.emit(OpenWindow::new(window, ui));
        id
    }

    fn close_window(self) {
        self.emit(CloseWindow::new());
    }
}
