use crate::{Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space, View};

pub trait ViewSequence<T> {
    type State;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn build(&self) -> Self::State;

    fn rebuild(&mut self, index: usize, cx: &mut RebuildCx, old: &Self, state: &mut Self::State);

    fn event(
        &mut self,
        index: usize,
        cx: &mut EventCx,
        state: &mut Self::State,
        data: &mut T,
        event: &Event,
    );

    fn layout(
        &mut self,
        index: usize,
        cx: &mut LayoutCx,
        state: &mut Self::State,
        space: Space,
    ) -> Size;

    fn draw(&mut self, index: usize, cx: &mut DrawCx, state: &mut Self::State, scene: &mut Canvas);
}

impl<T, V: View<T>> ViewSequence<T> for Vec<V> {
    type State = Vec<V::State>;

    fn len(&self) -> usize {
        self.len()
    }

    fn build(&self) -> Self::State {
        self.iter().map(|v| v.build()).collect()
    }

    fn rebuild(&mut self, index: usize, cx: &mut RebuildCx, old: &Self, state: &mut Self::State) {
        self[index].rebuild(cx, &old[index], &mut state[index]);
    }

    fn event(
        &mut self,
        index: usize,
        cx: &mut EventCx,
        state: &mut Self::State,
        data: &mut T,
        event: &Event,
    ) {
        self[index].event(cx, &mut state[index], data, event);
    }

    fn layout(
        &mut self,
        index: usize,
        cx: &mut LayoutCx,
        state: &mut Self::State,
        space: Space,
    ) -> Size {
        self[index].layout(cx, &mut state[index], space)
    }

    fn draw(
        &mut self,
        index: usize,
        cx: &mut DrawCx,
        state: &mut Self::State,
        canvas: &mut Canvas,
    ) {
        self[index].draw(cx, &mut state[index], canvas);
    }
}

impl<T> ViewSequence<T> for () {
    type State = ();

    fn len(&self) -> usize {
        0
    }

    fn build(&self) -> Self::State {}

    fn rebuild(
        &mut self,
        _index: usize,
        _cx: &mut RebuildCx,
        _old: &Self,
        _state: &mut Self::State,
    ) {
    }

    fn event(
        &mut self,
        _index: usize,
        _cx: &mut EventCx,
        _state: &mut Self::State,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(
        &mut self,
        _index: usize,
        _cx: &mut LayoutCx,
        _state: &mut Self::State,
        space: Space,
    ) -> Size {
        space.min
    }

    fn draw(
        &mut self,
        _index: usize,
        _cx: &mut DrawCx,
        _state: &mut Self::State,
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

            fn build(&self) -> Self::State {
                ($(self.$index.build(),)*)
            }

            fn rebuild(&mut self, index: usize, cx: &mut RebuildCx, old: &Self, state: &mut Self::State) {
                match index {
                    $($index => self.$index.rebuild(cx, &old.$index, &mut state.$index),)*
                    _ => {},
                }
            }

            fn event(
                &mut self,
                index: usize,
                cx: &mut EventCx,
                state: &mut Self::State,
                data: &mut T,
                event: &Event,
            ) {
                match index {
                    $($index => self.$index.event(cx, &mut state.$index, data, event),)*
                    _ => {},
                }
            }

            fn layout(
                &mut self,
                index: usize,
                cx: &mut LayoutCx,
                state: &mut Self::State,
                space: Space,
            ) -> Size {
                match index {
                    $($index => self.$index.layout(cx, &mut state.$index, space),)*
                    _ => Size::ZERO,
                }
            }

            fn draw(
                &mut self,
                index: usize,
                cx: &mut DrawCx,
                state: &mut Self::State,
                canvas: &mut Canvas,
            ) {
                match index {
                    $($index => self.$index.draw(cx, &mut state.$index, canvas),)*
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
