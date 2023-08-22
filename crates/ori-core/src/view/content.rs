use std::ops::{Deref, DerefMut};

use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
};

use super::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View, ViewState};

/// The state of [`Content`].
pub struct State<T, V: View<T> + ?Sized> {
    content: V::State,
    view_state: ViewState,
}

impl<T, V: View<T> + ?Sized> State<T, V> {
    /// Set the state to `active`.
    pub fn with_active(mut self, active: bool) -> Self {
        self.view_state.active = active;
        self
    }
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
///
/// This is strictly necessary for any view that contains any content.
/// If you don't wrap your content in this, you're in strange waters my friend,
/// and I wish you the best of luck.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Content<V> {
    pub(crate) view: V,
}

impl<V> Content<V> {
    /// Create a new content view.
    pub const fn new(view: V) -> Self {
        Self { view }
    }
}

impl<V> From<V> for Content<V> {
    fn from(view: V) -> Self {
        Self::new(view)
    }
}

impl<V> Deref for Content<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<V> DerefMut for Content<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}

impl<T, V: View<T>> View<T> for Content<V> {
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        State {
            content: self.view.build(cx, data),
            view_state: ViewState::default(),
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        state.view_state.prepare();

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state;

        (self.view).rebuild(&mut state.content, &mut new_cx, data, &old.view);
        new_cx.update();

        cx.view_state.propagate(&mut state.view_state);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if event.is_handled() && !state.view_state.has_active() {
            return;
        }

        state.view_state.prepare();

        let mut new_cx = cx.child();
        new_cx.transform *= state.view_state.transform;
        new_cx.view_state = &mut state.view_state;

        (self.view).event(&mut state.content, &mut new_cx, data, event);
        new_cx.update();

        cx.view_state.propagate(&mut state.view_state);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        state.view_state.prepare_layout();

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state;

        let size = (self.view).layout(&mut state.content, &mut new_cx, data, space);
        new_cx.update();

        state.view_state.size = size;
        cx.view_state.propagate(&mut state.view_state);

        size
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        state.view_state.prepare_draw();

        // create the canvas layer
        let mut canvas = canvas.layer();
        canvas.transform *= state.view_state.transform;

        // create the draw context
        let mut new_cx = cx.layer();
        new_cx.view_state = &mut state.view_state;

        // draw the content
        (self.view).draw(&mut state.content, &mut new_cx, data, &mut canvas);
        new_cx.update();

        // propagate the view state
        cx.view_state.propagate(&mut state.view_state);
    }
}
