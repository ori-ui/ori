use std::ops::{Deref, DerefMut};

use ori_core::{
    context::BaseCx,
    event::Event,
    view::{any, AnyView},
    window::{Window, WindowId},
};

use crate::{AppRequest, UiBuilder};

/// The context passed to the [`AppDelegate`] trait.
pub struct DelegateCx<'a, 'b, T> {
    base: &'a mut BaseCx<'b>,
    requests: &'a mut Vec<AppRequest<T>>,
    rebuild: &'a mut bool,
}

impl<'b, T> Deref for DelegateCx<'_, 'b, T> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<T> DerefMut for DelegateCx<'_, '_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b, T> DelegateCx<'a, 'b, T> {
    pub(crate) fn new(
        base: &'a mut BaseCx<'b>,
        requests: &'a mut Vec<AppRequest<T>>,
        rebuild: &'a mut bool,
    ) -> Self {
        Self {
            base,
            requests,
            rebuild,
        }
    }

    /// Request a rebuild of the view tree.
    pub fn rebuild(&mut self) {
        *self.rebuild = true;
    }

    /// Quit the application.
    pub fn quit(&mut self) {
        self.requests.push(AppRequest::Quit);
    }

    /// Add a window to the application.
    pub fn open_window<V: AnyView<T> + 'static>(
        &mut self,
        window: Window,
        mut ui: impl FnMut(&mut T) -> V + 'static,
    ) {
        let builder: UiBuilder<T> = Box::new(move |data| any(ui(data)));
        (self.requests).push(AppRequest::OpenWindow(window, builder));
    }

    /// Close a window.
    pub fn close_window(&mut self, window_id: WindowId) {
        self.requests.push(AppRequest::CloseWindow(window_id));
    }
}

/// A delegate for handling events in an application.
pub trait AppDelegate<T> {
    /// Called when the application is initialized.
    fn init(&mut self, cx: &mut DelegateCx<T>, data: &mut T) {
        let _ = (cx, data);
    }

    /// Called when the application is idle.
    fn idle(&mut self, cx: &mut DelegateCx<T>, data: &mut T) {
        let _ = (cx, data);
    }

    /// Handle an event.
    fn event(&mut self, cx: &mut DelegateCx<T>, data: &mut T, event: &Event) -> bool;
}
