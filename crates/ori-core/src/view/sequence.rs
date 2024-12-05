use std::ops::{Deref, DerefMut};

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event, FocusTarget},
    layout::{Size, Space},
};

use super::{Pod, View, ViewState};

/// A sequence of views.
#[allow(clippy::len_without_is_empty)]
pub trait ViewSeq<T> {
    /// The state of the sequence.
    type State;

    /// The length of the sequence.
    fn len(&self) -> usize;

    /// Build the sequence state.
    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> (Self::State, Vec<ViewState>);

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
    ) -> bool;

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
    fn draw_nth(&mut self, n: usize, state: &mut Self::State, cx: &mut DrawCx, data: &mut T);
}

impl<T, V: View<T>> ViewSeq<T> for Vec<V> {
    type State = Vec<V::State>;

    fn len(&self) -> usize {
        self.len()
    }

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> (Self::State, Vec<ViewState>) {
        let mut states = Vec::with_capacity(self.len());
        let mut view_states = Vec::with_capacity(self.len());

        for view in self.iter_mut() {
            let (state, view_state) = Pod::<V>::build_with(cx, |cx| view.build(cx, data));
            view_states.push(view_state);
            states.push(state);
        }

        (states, view_states)
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
    ) -> bool {
        self[n].event(&mut state[n], cx, data, event)
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

    fn draw_nth(&mut self, n: usize, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self[n].draw(&mut state[n], cx, data);
    }
}

impl<T> ViewSeq<T> for () {
    type State = ();

    fn len(&self) -> usize {
        0
    }

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> (Self::State, Vec<ViewState>) {
        ((), Vec::new())
    }

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
    ) -> bool {
        false
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

    fn draw_nth(&mut self, _n: usize, _state: &mut Self::State, _cx: &mut DrawCx, _data: &mut T) {}
}

macro_rules! impl_tuple {
    ($($name:ident)* ; $($index:tt)*) => {
        impl<T, $($name: View<T>),* > ViewSeq<T> for ($($name,)*) {
            type State = ($($name::State,)*);

            fn len(&self) -> usize {
                0$(.max($index + 1))*
            }

            fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> (Self::State, Vec<ViewState>) {
                let mut view_states = Vec::with_capacity(self.len());

                let state = ($({
                    let (state, view_state) = Pod::<$name>::build_with(cx, |cx| {
                        self.$index.build(cx, data)
                    });

                    view_states.push(view_state);
                    state
                },)*);

                (state, view_states)
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
            ) -> bool {
                match n {
                    $($index => self.$index.event(&mut state.$index, cx, data, event),)*
                    _ => false,
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
            ) {
                match n {
                    $($index => self.$index.draw(&mut state.$index, cx, data),)*
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

impl<V> PodSeq<V> {
    /// The length of the sequence.
    pub fn len<T>(&self) -> usize
    where
        V: ViewSeq<T>,
    {
        self.views.len()
    }

    /// Whether the sequence is empty.
    pub fn is_empty<T>(&self) -> bool
    where
        V: ViewSeq<T>,
    {
        self.views.len() == 0
    }

    /// Build the sequence state.
    pub fn build<T>(&mut self, cx: &mut BuildCx, data: &mut T) -> SeqState<T, V>
    where
        V: ViewSeq<T>,
    {
        let (content, view_state) = self.views.build(cx, data);

        SeqState {
            content,
            view_state,
        }
    }

    /// Rebuild the sequence state.
    pub fn rebuild<T>(
        &mut self,
        state: &mut SeqState<T, V>,
        cx: &mut BuildCx,
        data: &mut T,
        old: &Self,
    ) where
        V: ViewSeq<T>,
    {
        (state.view_state).resize_with(self.views.len(), ViewState::default);

        (self.views).rebuild(&mut state.content, cx, data, &old.views);
    }

    /// Rebuild the nth view.
    pub fn rebuild_nth<T>(
        &mut self,
        n: usize,
        state: &mut SeqState<T, V>,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) where
        V: ViewSeq<T>,
    {
        Pod::<V>::rebuild_with(&mut state.view_state[n], cx, |cx| {
            (self.views).rebuild_nth(n, &mut state.content, cx, data, &old.views);
        });
    }

    /// Handle an event.
    pub fn event<T>(
        &mut self,
        state: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool
    where
        V: ViewSeq<T>,
    {
        match event {
            Event::FocusNext => {
                Self::switch_focus(self, state, cx, data, event, 0..self.len(), true)
            }
            Event::FocusPrev => {
                Self::switch_focus(self, state, cx, data, event, (0..self.len()).rev(), false)
            }
            Event::FocusGiven(FocusTarget::Next) => {
                Self::give_focus(self, state, cx, data, event, 0..self.len())
            }
            Event::FocusGiven(FocusTarget::Prev) => {
                Self::give_focus(self, state, cx, data, event, (0..self.len()).rev())
            }
            _ => {
                let mut handled = false;

                for i in 0..self.len() {
                    if handled {
                        cx.view_state.propagate(&mut state[i]);
                        continue;
                    }

                    handled |= self.event_nth(i, state, cx, data, event);
                }

                handled
            }
        }
    }

    fn switch_focus<T>(
        &mut self,
        content: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
        order: impl IntoIterator<Item = usize>,
        forward: bool,
    ) -> bool
    where
        V: ViewSeq<T>,
    {
        enum State {
            TakingFocus,
            GivingFocus,
            Propagating,
        }

        let mut state = State::TakingFocus;

        for i in order {
            match state {
                State::TakingFocus => {
                    if !content[i].has_focused() {
                        cx.view_state.propagate(&mut content[i]);
                        continue;
                    }

                    let handled = self.event_nth(i, content, cx, data, event);

                    match handled && !content[i].has_focused() {
                        true => state = State::GivingFocus,
                        false => state = State::Propagating,
                    }
                }
                State::GivingFocus => {
                    let event = match forward {
                        true => Event::FocusGiven(FocusTarget::Next),
                        false => Event::FocusGiven(FocusTarget::Prev),
                    };

                    let handled = self.event_nth(i, content, cx, data, &event);

                    if handled && content[i].has_focused() {
                        state = State::Propagating;
                    }
                }
                State::Propagating => {
                    cx.view_state.propagate(&mut content[i]);
                }
            }
        }

        matches!(state, State::GivingFocus | State::Propagating)
    }

    fn give_focus<T>(
        &mut self,
        state: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
        content: impl IntoIterator<Item = usize>,
    ) -> bool
    where
        V: ViewSeq<T>,
    {
        let mut handled = false;

        for i in content {
            if handled {
                cx.view_state.propagate(&mut state[i]);
                continue;
            }

            handled |= self.event_nth(i, state, cx, data, event);
        }

        handled
    }

    /// Handle an event for the nth view.
    ///
    /// This is only useful if you need to handle events in a specific way, otherwise use [`Self::event`].
    pub fn event_nth<T>(
        &mut self,
        n: usize,
        state: &mut SeqState<T, V>,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool
    where
        V: ViewSeq<T>,
    {
        Pod::<V>::event_with(&mut state.view_state[n], cx, event, |cx, event| {
            (self.views).event_nth(n, &mut state.content, cx, data, event)
        })
    }

    /// Layout the nth view.
    pub fn layout_nth<T>(
        &mut self,
        n: usize,
        state: &mut SeqState<T, V>,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size
    where
        V: ViewSeq<T>,
    {
        Pod::<V>::layout_with(&mut state.view_state[n], cx, |cx| {
            (self.views).layout_nth(n, &mut state.content, cx, data, space)
        })
    }

    /// Draw the nth view.
    pub fn draw_nth<T>(
        &mut self,
        n: usize,
        state: &mut SeqState<T, V>,
        cx: &mut DrawCx,
        data: &mut T,
    ) where
        V: ViewSeq<T>,
    {
        Pod::<V>::draw_with(&mut state.view_state[n], cx, |cx| {
            if !cx.is_visible(cx.rect()) {
                return;
            }

            (self.views).draw_nth(n, &mut state.content, cx, data)
        });
    }
}
