//! Delegates for handling commands and events for the entire application.

use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use crate::{command::CommandProxy, event::Event, view::BaseCx};

/// A context for a [`Delegate`].
pub struct DelegateCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
}

impl<'a, 'b> DelegateCx<'a, 'b> {
    pub(crate) fn new(base: &'a mut BaseCx<'b>) -> Self {
        Self { base }
    }

    /// Get a proxy for sending commands.
    pub fn proxy(&self) -> CommandProxy {
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

impl<'a, 'b> Deref for DelegateCx<'a, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b> DerefMut for DelegateCx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

/// A delegate for handling events.
///
/// Delegate is implemented for functions like:
/// ```ignore
/// fn delegate<T>(cx: &mut DelegateCx, data: &mut T, event: &Event) {
///     // ...
/// }
/// ```
pub trait Delegate<T> {
    /// Called when the application starts.
    fn init(&mut self, _cx: &mut DelegateCx, _data: &mut T) {}

    /// Handle an event, returning whether it was handled.
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event);

    /// Called when the event loop is idle.
    fn idle(&mut self, _cx: &mut DelegateCx, _data: &mut T) {}
}

impl<T, F> Delegate<T> for F
where
    F: FnMut(&mut DelegateCx, &mut T, &Event),
{
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event) {
        self(cx, data, event)
    }
}
