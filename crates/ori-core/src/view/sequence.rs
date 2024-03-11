use std::{
    any::type_name,
    ops::{Deref, DerefMut},
    time::Instant,
};

use crate::{
    canvas::Canvas,
    debug::DebugTree,
    event::Event,
    layout::{Size, Space},
};

use super::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, View, ViewState};

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

    /// The debug name of the nth view.
    fn debug_name(&self, _n: usize) -> &'static str {
        type_name::<Self>()
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

    fn debug_name(&self, _n: usize) -> &'static str {
        type_name::<V>()
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

            fn debug_name(&self, n: usize) -> &'static str {
                match n {
                    $($index => type_name::<$name>(),)*
                    _ => "unknown",
                }
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
impl_tuple!(A B C D E F G H I J K L M; 0 1 2 3 4 5 6 7 8 9 10 11 12);
impl_tuple!(A B C D E F G H I J K L M N; 0 1 2 3 4 5 6 7 8 9 10 11 12 13);
impl_tuple!(A B C D E F G H I J K L M N O; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14);
impl_tuple!(A B C D E F G H I J K L M N O P; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15);
impl_tuple!(A B C D E F G H I J K L M N O P Q; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);
impl_tuple!(A B C D E F G H I J K L M N O P Q R; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S U; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S U V; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S U V W; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S U V W X; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22);
impl_tuple!(A B C D E F G H I J K L M N O P Q R S U V W X Z; 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23);

/// The state of a [`PodSeq`].
pub struct SeqState<T, V: ViewSeq<T>> {
    content: V::State,
    view_state: Vec<ViewState>,
}

impl<T, V: ViewSeq<T>> SeqState<T, V> {
    /// Whether any of the views in the sequence are active.
    pub fn has_active(&self) -> bool {
        self.view_state.iter().any(|state| state.has_active())
    }
}

impl<T, V: ViewSeq<T>> Deref for SeqState<T, V> {
    type Target = Vec<ViewState>;

    fn deref(&self) -> &Self::Target {
        &self.view_state
    }
}

impl<T, V: ViewSeq<T>> DerefMut for SeqState<T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.view_state
    }
}

/// Contents of a view, in a sequence.
///
/// This is useful for views that contain multiple pieces of content.
/// See [`ViewSeq`] for more information.
///
/// See [`Pod`] for more information on when to use this.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PodSeq<V> {
    views: V,
}

impl<V> PodSeq<V> {
    /// Create a new [`PodSeq`].
    pub fn new(views: V) -> Self {
        Self { views }
    }
}

impl<V> From<V> for PodSeq<V> {
    fn from(views: V) -> Self {
        Self::new(views)
    }
}

impl<V> Deref for PodSeq<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.views
    }
}

impl<V> DerefMut for PodSeq<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.views
    }
}

impl<T, V: ViewSeq<T>> ViewSeq<T> for PodSeq<V> {
    type State = SeqState<T, V>;

    fn len(&self) -> usize {
        self.views.len()
    }

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut view_state = Vec::with_capacity(self.len());

        for _ in 0..self.len() {
            view_state.push(ViewState::default());
        }

        SeqState {
            content: Pod::<V>::build(cx, |cx| self.views.build(cx, data)),
            view_state,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut BuildCx, data: &mut T, old: &Self) {
        if let Some(debug_tree) = cx.get_context_mut::<DebugTree>() {
            debug_tree.truncate(self.len());
        }

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
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(n);
            cx.insert_context(child_tree);

            let start = Instant::now();

            Pod::<V>::rebuild(&mut state.view_state[n], cx, |cx| {
                (self.views).rebuild_nth(n, &mut state.content, cx, data, &old.views);
            });

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_name(self.views.debug_name(n));
            child_tree.set_rebuild_time(start.elapsed());

            debug_tree.insert(n, child_tree);
            cx.insert_context(debug_tree);
        } else {
            Pod::<V>::rebuild(&mut state.view_state[n], cx, |cx| {
                (self.views).rebuild_nth(n, &mut state.content, cx, data, &old.views);
            });
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
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(n);
            cx.insert_context(child_tree);

            let start = Instant::now();

            Pod::<V>::event(&mut state.view_state[n], cx, event, |cx, event| {
                (self.views).event_nth(n, &mut state.content, cx, data, event);
            });

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_event_time(start.elapsed());

            debug_tree.insert(n, child_tree);
            cx.insert_context(debug_tree);
        } else {
            Pod::<V>::event(&mut state.view_state[n], cx, event, |cx, event| {
                (self.views).event_nth(n, &mut state.content, cx, data, event);
            });
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
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(n);
            cx.insert_context(child_tree);

            let start = Instant::now();

            let size = Pod::<V>::layout(&mut state.view_state[n], cx, |cx| {
                (self.views).layout_nth(n, &mut state.content, cx, data, space)
            });

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_space(space);
            child_tree.set_flex(state.view_state[n].flex, state.view_state[n].flex);
            child_tree.set_layout_time(start.elapsed());

            debug_tree.insert(n, child_tree);
            cx.insert_context(debug_tree);

            size
        } else {
            Pod::<V>::layout(&mut state.view_state[n], cx, |cx| {
                (self.views).layout_nth(n, &mut state.content, cx, data, space)
            })
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
        if let Some(mut debug_tree) = cx.remove_context::<DebugTree>() {
            let child_tree = debug_tree.remove_or_new(n);
            cx.insert_context(child_tree);

            let start = Instant::now();

            Pod::<V>::draw(&mut state.view_state[n], cx, canvas, |cx, canvas| {
                (self.views).draw_nth(n, &mut state.content, cx, data, canvas)
            });

            let mut child_tree = cx.remove_context::<DebugTree>().unwrap();
            child_tree.set_rect(state.view_state[n].rect());
            child_tree.set_transform(state.view_state[n].transform());
            child_tree.set_depth(canvas.depth);
            child_tree.set_draw_time(start.elapsed());

            debug_tree.insert(n, child_tree);
            cx.insert_context(debug_tree);
        } else {
            Pod::<V>::draw(&mut state.view_state[n], cx, canvas, |cx, canvas| {
                (self.views).draw_nth(n, &mut state.content, cx, data, canvas)
            });
        }
    }
}
