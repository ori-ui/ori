use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use crate::{
    BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space, Update,
    ViewSequence, ViewState,
};

/// The state of a [`ContentSequence`].
pub struct ContentSequenceState<T, V: ViewSequence<T>> {
    content: V::State,
    view_state: Vec<ViewState>,
}

impl<T, V: ViewSequence<T>, S: SliceIndex<[ViewState]>> Index<S> for ContentSequenceState<T, V> {
    type Output = S::Output;

    fn index(&self, index: S) -> &Self::Output {
        &self.view_state[index]
    }
}

impl<T, V: ViewSequence<T>, S: SliceIndex<[ViewState]>> IndexMut<S> for ContentSequenceState<T, V> {
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        &mut self.view_state[index]
    }
}

/// Contents of a view, in a sequence.
///
/// This is useful for views that contain multiple pieces of content.
/// See [`ViewSequence`] for more information.
///
/// This is strictly necessary for any view that contains any content.
/// If you don't wrap your content in this, you're in strange waters my friend,
/// and I wish you the best of luck.
#[repr(transparent)]
pub struct ContentSequence<T, V> {
    views: V,
    marker: PhantomData<fn() -> T>,
}

impl<T, V> ContentSequence<T, V> {
    /// Create a new [`ContentSequence`].
    pub fn new(views: V) -> Self {
        Self {
            views,
            marker: PhantomData,
        }
    }
}

impl<T, V: ViewSequence<T>> ViewSequence<T> for ContentSequence<T, V> {
    type State = ContentSequenceState<T, V>;

    fn len(&self) -> usize {
        self.views.len()
    }

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        ContentSequenceState {
            content: self.views.build(cx, data),
            view_state: vec![ViewState::default(); self.views.len()],
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut BuildCx, data: &mut T, old: &Self) {
        (state.view_state).resize_with(self.views.len(), ViewState::default);

        (self.views).rebuild(&mut state.content, cx, data, &old.views);
    }

    fn rebuild_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state[n];
        (self.views).rebuild_nth(n, &mut state.content, &mut new_cx, data, &old.views);

        cx.view_state.propagate(&mut state.view_state[n]);
    }

    fn event_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        let mut new_cx = cx.child();
        new_cx.transform *= state.view_state[n].transform;
        new_cx.view_state = &mut state.view_state[n];

        (self.views).event_nth(n, &mut state.content, &mut new_cx, data, event);

        cx.view_state.propagate(&mut state.view_state[n]);
    }

    fn layout_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        state.view_state[n].update.remove(Update::LAYOUT);

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state[n];

        let size = (self.views).layout_nth(n, &mut state.content, &mut new_cx, data, space);

        state.view_state[n].size = size;

        cx.view_state.propagate(&mut state.view_state[n]);

        size
    }

    fn draw_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        state.view_state[n].update.remove(Update::DRAW);
        state.view_state[n].depth = canvas.depth;

        let mut canvas = canvas.layer();
        canvas.transform *= state.view_state[n].transform;

        let mut new_cx = cx.layer();
        new_cx.view_state = &mut state.view_state[n];

        (self.views).draw_nth(n, &mut state.content, &mut new_cx, data, &mut canvas);

        cx.view_state.propagate(&mut state.view_state[n]);
    }
}
