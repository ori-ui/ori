//! Delegates for handling commands and events for the entire application.

use crate::{event::Event, view::DelegateCx};

/// A delegate for handling events.
/// ```
pub trait Delegate<T> {
    /// Called when the application starts.
    fn init(&mut self, _cx: &mut DelegateCx, _data: &mut T) {}

    /// Handle an event, returning whether it was handled.
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event);

    /// Called when the event loop is idle.
    fn idle(&mut self, _cx: &mut DelegateCx, _data: &mut T) {}
}
