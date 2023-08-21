use std::{
    ops::{Index, IndexMut},
    slice::SliceIndex,
};

use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
};

use super::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View, ViewState};

/// A sequence of views.
pub trait ViewSeq<T> {
    /// The state of the sequence.
    type State;

    /// The length of the sequence.
    fn len(&self) -> usize;

    /// Whether the sequence is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Build the sequence state.
    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State;

    /// Rebuild the sequence state.
    fn rebuild(&mut self, state: &mut Self::State, cx: &mut BuildCx, data: &mut T, old: &Self);

    /// Rebuild the nth view.
    fn rebuild_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    );

    /// Handle an event for the nth view.
    fn event_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    );

    /// Layout the nth view.
    fn layout_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size;

    /// Draw the nth view.
    fn draw_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        scene: &mut Canvas,
    );
}

impl<T, V: View<T>> ViewSeq<T> for Vec<V> {
    type State = Vec<V::State>;

    fn len(&self) -> usize {
        self.len()
    }

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.iter_mut().map(|v| v.build(cx, data)).collect()
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut BuildCx, data: &mut T, _old: &Self) {
        if self.len() < state.len() {
            state.truncate(self.len());
        } else {
            for item in self.iter_mut().skip(state.len()) {
                state.push(item.build(cx, data));
            }
        }
    }

    fn rebuild_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        if let Some(old) = old.get(n) {
            self[n].rebuild(&mut state[n], cx, data, old);
        }
    }

    fn event_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        self[n].event(&mut state[n], cx, data, event);
    }

    fn layout_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self[n].layout(&mut state[n], cx, data, space)
    }

    fn draw_nth(
        &mut self,
        n: usize,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self[n].draw(&mut state[n], cx, data, canvas);
    }
}

impl<T> ViewSeq<T> for () {
    type State = ();

    fn len(&self) -> usize {
        0
    }

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {}

    fn rebuild(&mut self, _state: &mut Self::State, _cx: &mut BuildCx, _data: &mut T, _old: &Self) {
    }

    fn rebuild_nth(
        &mut self,
        _n: usize,
        _state: &mut Self::State,
        _cx: &mut RebuildCx,
        _data: &mut T,
        _old: &Self,
    ) {
    }

    fn event_nth(
        &mut self,
        _n: usize,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout_nth(
        &mut self,
        _n: usize,
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.min
    }

    fn draw_nth(
        &mut self,
        _n: usize,
        _state: &mut Self::State,
        _cx: &mut DrawCx,
        _data: &mut T,
        _canvas: &mut Canvas,
    ) {
    }
}

macro_rules! impl_tuple {
    ($($name:ident)* ; $($index:tt)*) => {
        impl<T, $($name: View<T>),* > ViewSeq<T> for ($($name,)*) {
            type State = ($($name::State,)*);

            fn len(&self) -> usize {
                0$(.max($index + 1))*
            }

            fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
                ($(self.$index.build(cx, data),)*)
            }

            fn rebuild(
                &mut self,
                _state: &mut Self::State,
                _cx: &mut BuildCx,
                _data: &mut T,
                _old: &Self,
            ) {
            }

            fn rebuild_nth(
                &mut self,
                n: usize,
                state: &mut Self::State,
                cx: &mut RebuildCx,
                data: &mut T,
                old: &Self,
            ) {
                match n {
                    $($index => self.$index.rebuild(&mut state.$index, cx, data, &old.$index),)*
                    _ => {},
                }
            }

            fn event_nth(
                &mut self,
                n: usize,
                state: &mut Self::State,
                cx: &mut EventCx,
                data: &mut T,
                event: &Event,
            ) {
                match n {
                    $($index => self.$index.event(&mut state.$index, cx, data, event),)*
                    _ => {},
                }
            }

            fn layout_nth(
                &mut self,
                n: usize,
                state: &mut Self::State,
                cx: &mut LayoutCx,
                data: &mut T,
                space: Space,
            ) -> Size {
                match n {
                    $($index => self.$index.layout(&mut state.$index, cx, data, space),)*
                    _ => Size::ZERO,
                }
            }

            fn draw_nth(
                &mut self,
                n: usize,
                state: &mut Self::State,
                cx: &mut DrawCx,
                data: &mut T,
                canvas: &mut Canvas,
            ) {
                match n {
                    $($index => self.$index.draw(&mut state.$index, cx, data, canvas),)*
                    _ => {},
                }
            }
        }
    };
}

// NOTE: this is pretty ugly, but it works
impl_tuple!(A; 0);
impl_tuple!(A B; 0 1);
impl_tuple!(A B C; 0 1 2);
impl_tuple!(A B C D; 0 1 2 3);
impl_tuple!(A B C D E; 0 1 2 3 4);
impl_tuple!(A B C D E F; 0 1 2 3 4 5);
impl_tuple!(A B C D E F G; 0 1 2 3 4 5 6);
impl_tuple!(A B C D E F G H; 0 1 2 3 4 5 6 7);
impl_tuple!(A B C D E F G H I; 0 1 2 3 4 5 6 7 8);
impl_tuple!(A B C D E F G H I J; 0 1 2 3 4 5 6 7 8 9);
impl_tuple!(A B C D E F G H I J K; 0 1 2 3 4 5 6 7 8 9 10);
impl_tuple!(A B C D E F G H I J K L; 0 1 2 3 4 5 6 7 8 9 10 11);

/// The state of a [`ContentSequence`].
pub struct SeqState<T, V: ViewSeq<T>> {
    content: V::State,
    view_state: Vec<ViewState>,
}

impl<T, V: ViewSeq<T>, S: SliceIndex<[ViewState]>> Index<S> for SeqState<T, V> {
    type Output = S::Output;

    fn index(&self, index: S) -> &Self::Output {
        &self.view_state[index]
    }
}

impl<T, V: ViewSeq<T>, S: SliceIndex<[ViewState]>> IndexMut<S> for SeqState<T, V> {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentSeq<V> {
    views: V,
}

impl<V> ContentSeq<V> {
    /// Create a new [`ContentSequence`].
    pub fn new(views: V) -> Self {
        Self { views }
    }
}

impl<V> From<V> for ContentSeq<V> {
    fn from(views: V) -> Self {
        Self::new(views)
    }
}

impl<T, V: ViewSeq<T>> ViewSeq<T> for ContentSeq<V> {
    type State = SeqState<T, V>;

    fn len(&self) -> usize {
        self.views.len()
    }

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        SeqState {
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
        state.view_state[n].prepare();

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state[n];
        (self.views).rebuild_nth(n, &mut state.content, &mut new_cx, data, &old.views);
        new_cx.update_cursor();

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
        state.view_state[n].prepare();

        let mut new_cx = cx.child();
        new_cx.transform *= state.view_state[n].transform;
        new_cx.view_state = &mut state.view_state[n];

        (self.views).event_nth(n, &mut state.content, &mut new_cx, data, event);
        new_cx.update_cursor();

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
        state.view_state[n].prepare_layout();

        let mut new_cx = cx.child();
        new_cx.view_state = &mut state.view_state[n];

        let size = (self.views).layout_nth(n, &mut state.content, &mut new_cx, data, space);
        new_cx.update_cursor();

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
        state.view_state[n].prepare_draw(canvas);

        let mut canvas = canvas.layer();
        canvas.transform *= state.view_state[n].transform;

        let mut new_cx = cx.layer();
        new_cx.view_state = &mut state.view_state[n];

        (self.views).draw_nth(n, &mut state.content, &mut new_cx, data, &mut canvas);
        new_cx.update_cursor();

        cx.view_state.propagate(&mut state.view_state[n]);
    }
}
