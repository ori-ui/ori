use std::ops::{Deref, DerefMut};

use ori_core::{
    context::BaseCx,
    event::Event,
    view::{any, AnyView},
    window::{WindowDescriptor, WindowId},
};

use crate::{AppRequest, UiBuilder};

/// The context passed to the [`Delegate`] trait.
pub struct DelegateCx<'a, 'b, T> {
    base: &'a mut BaseCx<'b>,
    requests: &'a mut Vec<AppRequest<T>>,
    rebuild: &'a mut bool,
}

impl<'a, 'b, T> Deref for DelegateCx<'a, 'b, T> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b, T> DerefMut for DelegateCx<'a, 'b, T> {
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
    pub fn request_rebuild(&mut self) {
        *self.rebuild = true;
    }

    /// Quit the application.
    pub fn quit(&mut self) {
        self.requests.push(AppRequest::Quit);
    }

    /// Add a window to the application.
    pub fn open_window<V: AnyView<T> + 'static>(
        &mut self,
        descriptor: WindowDescriptor,
        mut ui: impl FnMut(&mut T) -> V + 'static,
    ) {
        let builder: UiBuilder<T> = Box::new(move |data| any(ui(data)));
        (self.requests).push(AppRequest::OpenWindow(descriptor, builder));
    }

    /// Close a window.
    pub fn close_window(&mut self, window_id: WindowId) {
        self.requests.push(AppRequest::CloseWindow(window_id));
    }
}

/// A delegate for handling events in an application.
pub trait Delegate<T> {
    /// Handle an event.
    fn event(&mut self, cx: &mut DelegateCx<T>, data: &mut T, event: &Event) -> bool;
}
