use ori_reactive::{Scope, Signal};

use crate::{Element, OpenWindow, Window, WindowId};

pub trait ScopeWindowExt {
    fn window(self) -> Signal<Window>;
    fn open_window(
        self,
        window: Window,
        ui: impl FnOnce(Scope) -> Element + Send + Sync + 'static,
    ) -> WindowId;
}

impl ScopeWindowExt for Scope {
    fn window(self) -> Signal<Window> {
        self.context::<Signal<Window>>()
    }

    fn open_window(
        self,
        window: Window,
        ui: impl FnOnce(Scope) -> Element + Send + Sync + 'static,
    ) -> WindowId {
        let id = window.id();
        self.emit(OpenWindow::new(window, ui));
        id
    }
}
