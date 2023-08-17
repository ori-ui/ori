use crate::{Event, ViewState};

/// A context for a [`Delegate`].
pub struct DelegateCx<'a> {
    pub(crate) view_state: &'a mut ViewState,
}

impl<'a> DelegateCx<'a> {
    pub(crate) fn new(view_state: &'a mut ViewState) -> Self {
        Self { view_state }
    }

    pub fn request_rebuild(&mut self) {
        self.view_state.request_rebuild();
    }

    pub fn request_layout(&mut self) {
        self.view_state.request_layout();
    }

    pub fn request_draw(&mut self) {
        self.view_state.request_draw();
    }
}

/// A delegate for handling events.
pub trait Delegate<T> {
    /// Handle an event.
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event);
}

impl<T, F> Delegate<T> for F
where
    F: FnMut(&mut DelegateCx, &mut T, &Event),
{
    fn event(&mut self, cx: &mut DelegateCx, data: &mut T, event: &Event) {
        self(cx, data, event)
    }
}
