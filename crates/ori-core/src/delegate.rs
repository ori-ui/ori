use std::any::Any;

use crate::{BaseCx, Event, ViewState};

/// A context for a [`Delegate`].
pub struct DelegateCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
}

impl<'a, 'b> DelegateCx<'a, 'b> {
    pub(crate) fn new(base: &'a mut BaseCx<'b>, view_state: &'a mut ViewState) -> Self {
        Self { base, view_state }
    }

    /// Send a command.
    pub fn cmd<T: Any>(&mut self, event: T) {
        self.base.cmd(Command::new(event));
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        self.view_state.request_rebuild();
    }

    /// Request a layout of the view tree.
    pub fn request_layout(&mut self) {
        self.view_state.request_layout();
    }

    /// Request a draw of the view tree.
    pub fn request_draw(&mut self) {
        self.view_state.request_draw();
    }
}

/// A command for a [`Delegate`].
#[derive(Clone, Debug)]
pub struct Command {
    event: Event,
}

impl Command {
    /// Create a new command.
    pub fn new<T: Any>(event: T) -> Self {
        Self {
            event: Event::new(event),
        }
    }

    /// Get the name of the event.
    pub fn event(&self) -> &Event {
        &self.event
    }
}

/// A delegate for handling events.
pub trait Delegate<T> {
    /// Handle an event, returning whether it was handled. If true,
    /// the event will not be send to the `view-tree`.
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event) -> bool;
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
