use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use crate::{
    BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space, Update,
    ViewSequence, ViewState,
};

pub struct PodSequenceState<T, V: ViewSequence<T>> {
    content: V::State,
    view_state: Vec<ViewState>,
}

impl<T, V: ViewSequence<T>, S: SliceIndex<[ViewState]>> Index<S> for PodSequenceState<T, V> {
    type Output = S::Output;

    fn index(&self, index: S) -> &Self::Output {
        &self.view_state[index]
    }
}

impl<T, V: ViewSequence<T>, S: SliceIndex<[ViewState]>> IndexMut<S> for PodSequenceState<T, V> {
    fn index_mut(&mut self, index: S) -> &mut Self::Output {
        &mut self.view_state[index]
    }
}

pub struct PodSequence<T, V> {
    content: V,
    marker: PhantomData<fn() -> T>,
}

impl<T, V> PodSequence<T, V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            marker: PhantomData,
        }
    }
}

impl<T, V: ViewSequence<T>> ViewSequence<T> for PodSequence<T, V> {
    type State = PodSequenceState<T, V>;

    fn len(&self) -> usize {
        self.content.len()
    }

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        PodSequenceState {
            content: self.content.build(cx, data),
            view_state: vec![ViewState::default(); self.content.len()],
        }
    }

    fn rebuild(
        &mut self,
        index: usize,
        state: &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        (state.view_state).resize_with(self.content.len(), ViewState::default);

        state.view_state[index].update.remove(Update::TREE);

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state[index];

        (self.content).rebuild(index, &mut state.content, &mut new_cx, data, &old.content);

        cx.view_state.propagate(&mut state.view_state[index]);
    }

    fn event(
        &mut self,
        index: usize,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        let mut new_cx = cx.child();
        new_cx.transform *= state.view_state[index].transform;
        new_cx.view_state = &mut state.view_state[index];

        (self.content).event(index, &mut state.content, &mut new_cx, data, event);

        cx.view_state.propagate(&mut state.view_state[index]);
    }

    fn layout(
        &mut self,
        index: usize,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        state.view_state[index].update.remove(Update::LAYOUT);

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state[index];

        let size = (self.content).layout(index, &mut state.content, &mut new_cx, data, space);

        state.view_state[index].size = size;

        cx.view_state.propagate(&mut state.view_state[index]);

        size
    }

    fn draw(
        &mut self,
        index: usize,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        state.view_state[index].update.remove(Update::DRAW);

        let mut canvas = canvas.layer();
        canvas.transform *= state.view_state[index].transform;

        let mut new_cx = cx.layer();
        new_cx.view_state = &mut state.view_state[index];

        (self.content).draw(index, &mut state.content, &mut new_cx, data, &mut canvas);

        cx.view_state.propagate(&mut state.view_state[index]);
    }
}
