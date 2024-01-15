//! Delegates for handling commands and events for the entire application.

use crate::{event::Event, view::DelegateCx};

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
