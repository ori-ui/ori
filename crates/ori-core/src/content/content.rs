use std::ops::{Deref, DerefMut};

use crate::{
    BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space, Update, View,
    ViewState,
};

/// The state of a [`ViewContent`].
pub struct State<T, V: View<T> + ?Sized> {
    content: V::State,
    view_state: ViewState,
}

impl<T, V: View<T> + ?Sized> Deref for State<T, V> {
    type Target = ViewState;

    fn deref(&self) -> &Self::Target {
        &self.view_state
    }
}

impl<T, V: View<T> + ?Sized> DerefMut for State<T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view_state
    }
}

/// Contents of a view.
pub trait ViewContent<T>: View<T> {
    /// Build the content state.
    fn build_content(&mut self, cx: &mut BuildCx, data: &mut T) -> State<T, Self> {
        State {
            content: self.build(cx, data),
            view_state: ViewState::default(),
        }
    }

    /// Rebuild the content state.
    fn rebuild_content(
        &mut self,
        state: &mut State<T, Self>,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state;

        View::rebuild(self, &mut state.content, &mut new_cx, data, old);

        cx.view_state.propagate(&mut state.view_state);
    }

    /// Handle an event.
    fn event_content(
        &mut self,
        state: &mut State<T, Self>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        let mut new_cx = cx.child();
        new_cx.transform *= state.view_state.transform;
        new_cx.view_state = &mut state.view_state;

        View::event(self, &mut state.content, &mut new_cx, data, event);

        cx.view_state.propagate(&mut state.view_state);
    }

    /// Layout the content.
    fn layout_content(
        &mut self,
        state: &mut State<T, Self>,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        state.view_state.update.remove(Update::LAYOUT);

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state;

        let size = View::layout(self, &mut state.content, &mut new_cx, data, space);
        state.view_state.size = size;

        cx.view_state.propagate(&mut state.view_state);

        size
    }

    /// Draw the content.
    fn draw_content(
        &mut self,
        state: &mut State<T, Self>,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        state.view_state.update.remove(Update::DRAW);
        state.view_state.depth = canvas.depth;

        // create the canvas layer
        let mut canvas = canvas.layer();
        canvas.transform *= state.view_state.transform;

        // create the draw context
        let mut new_cx = cx.layer();
        new_cx.view_state = &mut state.view_state;

        // draw the content
        View::draw(self, &mut state.content, &mut new_cx, data, &mut canvas);

        // propagate the view state
        cx.view_state.propagate(&mut state.view_state);
    }
}

impl<T, V: View<T>> ViewContent<T> for V {}
