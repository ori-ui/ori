use std::{collections::HashMap, hash::Hash};

use crate::{Action, Element, Event, NoElement, Sub, View};

/// A sequence of [`View`]s.
pub trait ViewSeq<C, T, E>
where
    E: Element<C>,
{
    /// State of the sequence.
    type State;

    /// Build [`Self::Elements`] and [`Self::State`], see [`View::build`] for more information.
    fn seq_build(self, elements: &mut impl Elements<C, E>, cx: &mut C, data: &mut T)
    -> Self::State;

    /// Rebuild the sequence, see [`View::rebuild`] for more information.
    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    );

    /// Handle an event for the sequence, see [`View::event`] for more information.
    ///
    /// Returns a list of indices of elements that have change in a way that might invalidate the
    /// parent child relation.
    fn seq_event(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action;

    /// Tear down the sequence, see [`View::teardown`] for more information.
    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C);
}

/// A iterator of elements, see [`ViewSeq`] for more details.
pub trait Elements<C, E>
where
    E: Element<C>,
{
    /// Get the next [`Element`].
    fn next(&mut self, cx: &mut C) -> Option<E::Mut<'_>>;

    /// Insert an [`Element`] at the current position.
    fn insert(&mut self, cx: &mut C, element: E);

    /// Remove the next [`Element`].
    fn remove(&mut self, cx: &mut C) -> Option<E>;

    /// Swap the next [`Element`] and another at `current + offset`.
    fn swap(&mut self, cx: &mut C, offset: usize);
}

/// There are no [`Elements`].
#[derive(Debug)]
pub struct NoElements;

impl<C> Elements<C, NoElement> for NoElements {
    fn next(&mut self, _cx: &mut C) -> Option<<NoElement as Element<C>>::Mut<'_>> {
        Some(())
    }

    fn insert(&mut self, _cx: &mut C, _element: NoElement) {}

    fn remove(&mut self, _cx: &mut C) -> Option<NoElement> {
        Some(NoElement)
    }

    fn swap(&mut self, _cx: &mut C, _offset: usize) {}
}

impl<C, T, E, V> ViewSeq<C, T, E> for V
where
    E: Element<C>,
    V: View<C, T>,
    V::Element: Sub<C, E>,
{
    type State = V::State;

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let (element, state) = self.build(cx, data);
        let element = V::Element::upcast(cx, element);
        elements.insert(cx, element);
        state
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        if let Some(element) = elements.next(cx) {
            V::Element::downcast_mut(cx, element, |cx, element| {
                self.rebuild(element, state, cx, data);
            });
        }
    }

    fn seq_event(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        if event.is_taken() {
            return Action::new();
        }

        if let Some(element) = elements.next(cx) {
            V::Element::downcast_mut(cx, element, |cx, element| {
                V::event(element, state, cx, data, event)
            })
            .unwrap_or(Action::new())
        } else {
            Action::new()
        }
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C) {
        if let Some(element) = elements.remove(cx)
            && let Some(element) = V::Element::downcast(cx, element)
        {
            V::teardown(element, state, cx);
        }
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for Option<V>
where
    E: Element<C>,
    V: ViewSeq<C, T, E>,
{
    type State = Option<V::State>;

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        self.map(|seq| seq.seq_build(elements, cx, data))
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        match self {
            Some(contents) => match state {
                None => {
                    let new_state = contents.seq_build(elements, cx, data);
                    *state = Some(new_state);
                }

                Some(state) => {
                    contents.seq_rebuild(elements, state, cx, data);
                }
            },

            None => {
                if let Some(state) = state.take() {
                    V::seq_teardown(elements, state, cx);
                }
            }
        }
    }

    fn seq_event(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        match state {
            Some(state) => V::seq_event(elements, state, cx, data, event),
            _ => Action::new(),
        }
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C) {
        if let Some(state) = state {
            V::seq_teardown(elements, state, cx);
        }
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for Vec<V>
where
    E: Element<C>,
    V: ViewSeq<C, T, E>,
{
    type State = Vec<V::State>;

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let mut states = Vec::with_capacity(self.len());

        for view in self {
            let state = view.seq_build(elements, cx, data);
            states.push(state);
        }

        states
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        states: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let len = self.len();

        for (i, view) in self.into_iter().enumerate() {
            match states.get_mut(i) {
                Some(state) => {
                    view.seq_rebuild(elements, state, cx, data);
                }

                None => {
                    let state = view.seq_build(elements, cx, data);
                    states.push(state);
                }
            }
        }

        if len < states.len() {
            for state in states.drain(len..) {
                V::seq_teardown(elements, state, cx);
            }
        }
    }

    fn seq_event(
        elements: &mut impl Elements<C, E>,
        states: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let mut action = Action::new();

        for state in states.iter_mut() {
            action |= V::seq_event(elements, state, cx, data, event);
        }

        action
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, states: Self::State, cx: &mut C) {
        for state in states {
            V::seq_teardown(elements, state, cx);
        }
    }
}

/// Create new [`Keyed`].
pub fn keyed<K, V>(pairs: impl IntoIterator<Item = (K, V)>) -> Keyed<K, V> {
    Keyed::new(pairs)
}

/// [`ViewSeq`] that orders contents to match a list of keys.
pub struct Keyed<K, V> {
    pairs: Vec<(K, V)>,
}

impl<K, V> Keyed<K, V> {
    /// Create new [`Keyed`].
    pub fn new(pairs: impl IntoIterator<Item = (K, V)>) -> Self {
        Self {
            pairs: pairs.into_iter().collect(),
        }
    }
}

impl<K, V> FromIterator<(K, V)> for Keyed<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        Self::new(iter)
    }
}

#[doc(hidden)]
pub struct KeyedState<K, S> {
    states:  Vec<S>,
    keys:    Vec<K>,
    indices: HashMap<K, usize>,
}

impl<C, T, E, K, V> ViewSeq<C, T, E> for Keyed<K, V>
where
    E: Element<C>,
    K: Clone + Hash + Eq,
    V: View<C, T>,
    V::Element: Sub<C, E>,
{
    type State = KeyedState<K, V::State>;

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let mut states = Vec::with_capacity(self.pairs.len());
        let mut keys = Vec::with_capacity(self.pairs.len());
        let mut indices = HashMap::with_capacity(self.pairs.len());

        for (i, (key, view)) in self.pairs.into_iter().enumerate() {
            let state = view.seq_build(elements, cx, data);

            states.push(state);
            keys.push(key.clone());
            indices.insert(key.clone(), i);
        }

        KeyedState {
            states,
            keys,
            indices,
        }
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let new_len = self.pairs.len();
        let mut offset = 0;

        for (i, (key, view)) in self.pairs.into_iter().enumerate() {
            let Some(index) = state.indices.get_mut(&key) else {
                let view_state = view.seq_build(elements, cx, data);

                state.states.insert(i, view_state);
                state.keys.insert(i, key.clone());
                state.indices.insert(key.clone(), i);

                offset += 1;

                continue;
            };

            *index += offset;
            let j = *index;

            if j != i {
                let other_key = state.keys[i].clone();

                elements.swap(cx, j - i);
                state.states.swap(i, j);
                state.keys.swap(i, j);

                state.indices.insert(key.clone(), i);
                state.indices.insert(other_key, j);
            }

            view.seq_rebuild(elements, &mut state.states[i], cx, data);
        }

        if state.keys.len() == new_len {
            return;
        }

        for (key, child_state) in state
            .keys
            .drain(new_len..)
            .zip(state.states.drain(new_len..))
        {
            state.indices.remove(&key);
            V::seq_teardown(elements, child_state, cx);
        }
    }

    fn seq_event(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let mut action = Action::new();

        for state in state.states.iter_mut() {
            action |= V::seq_event(elements, state, cx, data, event);
        }

        action
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C) {
        for state in state.states {
            V::seq_teardown(elements, state, cx);
        }
    }
}

macro_rules! impl_tuple {
    ($($name:ident, $index:tt);*) => {
        #[allow(unused)]
        impl<C, T, E, $($name),*> ViewSeq<C, T, E> for ($($name,)*)
        where
            E: Element<C>,
            $($name: ViewSeq<C, T, E>,)*
        {
            type State = ($($name::State,)*);

            fn seq_build(
                self,
                elements: &mut impl Elements<C, E>,
                cx: &mut C,
                data: &mut T,
            ) -> Self::State {
                #[allow(unused)]
                ($(self.$index.seq_build(elements, cx, data),)*)
            }

            fn seq_rebuild(
                self,
                elements: &mut impl Elements<C, E>,
                state: &mut Self::State,
                cx: &mut C,
                data: &mut T,
            ) {
                $({
                    self.$index.seq_rebuild(
                        elements,
                        &mut state.$index,
                        cx,
                        data,
                    );
                })*
            }

            fn seq_event(
                elements: &mut impl Elements<C, E>,
                state: &mut Self::State,
                cx: &mut C,
                data: &mut T,
                event: &mut Event,
            ) -> Action {
                let mut action = Action::new();

                $({
                    action |= $name::seq_event(
                        elements,
                        &mut state.$index,
                        cx,
                        data,
                        event,
                    );
                })*

                action
            }

            fn seq_teardown(
                elements: &mut impl Elements<C, E>,
                state: Self::State,
                cx: &mut C,
            ) {
                $({
                    $name::seq_teardown(
                        elements,
                        state.$index,
                        cx,
                    );
                })*
            }
        }
    };
}

impl_tuple!();
impl_tuple!(S0, 0);
impl_tuple!(S0, 0; S1, 1);
impl_tuple!(S0, 0; S1, 1; S2, 2);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4; S5, 5);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4; S5, 5; S6, 6);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4; S5, 5; S6, 6; S7, 7);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4; S5, 5; S6, 6; S7, 7; S8, 8);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4; S5, 5; S6, 6; S7, 7; S8, 8; S9, 9);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4; S5, 5; S6, 6; S7, 7; S8, 8; S9, 9; S10, 10);
impl_tuple!(S0, 0; S1, 1; S2, 2; S3, 3; S4, 4; S5, 5; S6, 6; S7, 7; S8, 8; S9, 9; S10, 10; S11, 11);
