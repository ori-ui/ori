//! Delegates for handling commands and events for the entire application.

use std::any::Any;

use crate::{event::Event, proxy::Proxy, view::BaseCx};

/// A context for a [`Delegate`].
pub struct DelegateCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
}

impl<'a, 'b> DelegateCx<'a, 'b> {
    pub(crate) fn new(base: &'a mut BaseCx<'b>) -> Self {
        Self { base }
    }

    /// Get a proxy for sending commands.
    pub fn proxy(&self) -> Proxy {
        self.base.proxy()
    }

    /// Send a command.
    pub fn cmd<T: Any + Send>(&mut self, event: T) {
        self.base.cmd(event);
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        self.base.request_rebuild();
    }
}

/// A delegate for handling events.
pub trait Delegate<T> {
    /// Handle an event, returning whether it was handled. If true,
    /// the event will not be send to the `view-tree`.
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event) -> bool;

    /// Called when the event loop is idle.
    fn idle(&mut self, _cx: &mut DelegateCx, _data: &mut T) {}
}

impl<T, F> Delegate<T> for F
where
    F: FnMut(&mut DelegateCx, &mut T, &Event) -> bool,
{
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event) -> bool {
        self(cx, data, event)
    }
}

impl<T> Delegate<T> for () {
    fn event(&mut self, _: &mut DelegateCx, _: &mut T, _: &Event) -> bool {
        false
    }
}
