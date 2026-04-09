use std::{
    collections::HashMap,
    hash::{BuildHasherDefault, Hash},
};

use seahash::SeaHasher;

use crate::{Action, Element, Elements, Is, Message, NodeId, Tracker, View, ViewSeq};

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
    indices: HashMap<K, usize, BuildHasherDefault<SeaHasher>>,
}

impl<C, T, E, K, V> ViewSeq<C, T, E> for Keyed<K, V>
where
    C: Tracker,
    E: Element,
    K: Clone + Hash + Eq,
    V: View<C, T>,
    V::Element: Is<C, E>,
{
    type State = KeyedState<K, (V::State, NodeId)>;

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let mut states = Vec::with_capacity(self.pairs.len());
        let mut keys = Vec::with_capacity(self.pairs.len());
        let mut indices = HashMap::with_capacity_and_hasher(self.pairs.len(), Default::default());

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
                state.indices.insert(other_key, j - offset);
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

    fn seq_message(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        let mut action = Action::new();

        for state in state.states.iter_mut() {
            action |= V::seq_message(elements, state, cx, data, message);
        }

        action
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C) {
        for state in state.states {
            V::seq_teardown(elements, state, cx);
        }
    }
}
