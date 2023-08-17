use crate::{BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space, View};

/// A sequence of views.
pub trait ViewSequence<T> {
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

impl<T, V: View<T>> ViewSequence<T> for Vec<V> {
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

impl<T> ViewSequence<T> for () {
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
        impl<T, $($name: View<T>),* > ViewSequence<T> for ($($name,)*) {
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
