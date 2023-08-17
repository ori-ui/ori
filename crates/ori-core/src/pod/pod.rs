use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    BoxedView, BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space, Update,
    View, ViewState,
};

/// A [`Content`] with a [`BoxedView`] as its content.
pub type AnyContent<T> = Content<T, BoxedView<T>>;

/// The state of a [`Content`].
pub struct ContentState<T, V: View<T>> {
    content: V::State,
    view_state: ViewState,
}

impl<T, V: View<T>> Deref for ContentState<T, V> {
    type Target = ViewState;

    fn deref(&self) -> &Self::Target {
        &self.view_state
    }
}

impl<T, V: View<T>> DerefMut for ContentState<T, V> {
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
pub struct Content<T, V> {
    view: V,
    marker: PhantomData<fn() -> T>,
}

impl<T, V> Content<T, V> {
    /// Create a new [`Content`].
    pub const fn new(view: V) -> Self {
        Self {
            view,
            marker: PhantomData,
        }
    }
}

impl<T, V: View<T>> View<T> for Content<T, V> {
    type State = ContentState<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        ContentState {
            content: self.view.build(cx, data),
            view_state: ViewState::default(),
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        state.view_state.update.remove(Update::TREE);

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state;

        (self.view).rebuild(&mut state.content, &mut new_cx, data, &old.view);

        cx.view_state.propagate(&mut state.view_state);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        let mut new_cx = cx.child();
        new_cx.transform *= state.view_state.transform;
        new_cx.view_state = &mut state.view_state;

        (self.view).event(&mut state.content, &mut new_cx, data, event);

        cx.view_state.propagate(&mut state.view_state);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        state.view_state.update.remove(Update::LAYOUT);

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state;

        let size = (self.view).layout(&mut state.content, &mut new_cx, data, space);
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
        state.view_state.update.remove(Update::DRAW);

        // create the canvas layer
        let mut canvas = canvas.layer();
        canvas.transform *= state.view_state.transform;

        // create the draw context
        let mut new_cx = cx.layer();
        new_cx.view_state = &mut state.view_state;

        // draw the content
        (self.view).draw(&mut state.content, &mut new_cx, data, &mut canvas);

        // propagate the view state
        cx.view_state.propagate(&mut state.view_state);
    }
}

impl<T, V: Debug> Debug for Content<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pod").field("content", &self.view).finish()
    }
}

impl<T, V: Clone> Clone for Content<T, V> {
    fn clone(&self) -> Self {
        Self {
            view: self.view.clone(),
            marker: PhantomData,
        }
    }
}

impl<T, V: Copy> Copy for Content<T, V> {}

impl<T, V: Default> Default for Content<T, V> {
    fn default() -> Self {
        Self::new(V::default())
    }
}

impl<T, V> Deref for Content<T, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<T, V> DerefMut for Content<T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view
    }
}
