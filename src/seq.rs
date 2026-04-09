use crate::{Action, Element, Is, Message, NodeId, Tracker, View};

/// A sequence of [`View`]s.
#[must_use = "views do nothing outside of the view tree"]
pub trait ViewSeq<C, T, E>
where
    E: Element,
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

    /// Handle a message for the sequence, see [`View::message`] for more information.
    fn seq_message(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action;

    /// Tear down the sequence, see [`View::teardown`] for more information.
    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C);
}

/// A iterator of elements, see [`ViewSeq`] for more details.
pub trait Elements<C, E>
where
    E: Element,
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

impl<C> Elements<C, ()> for () {
    fn next(&mut self, _cx: &mut C) -> Option<()> {
        Some(())
    }

    fn insert(&mut self, _cx: &mut C, _element: ()) {}

    fn remove(&mut self, _cx: &mut C) -> Option<()> {
        Some(())
    }

    fn swap(&mut self, _cx: &mut C, _offset: usize) {}
}

impl<C, T, E, V> ViewSeq<C, T, E> for V
where
    C: Tracker,
    E: Element,
    V: View<C, T>,
    V::Element: Is<C, E>,
{
    type State = (V::State, NodeId);

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let id = NodeId::next();

        cx.tree().insert(id);
        cx.tree().push(id);

        let (element, state) = self.build(cx, data);
        let element = V::Element::upcast(cx, element);
        elements.insert(cx, element);

        cx.tree().pop();

        (state, id)
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        (state, id): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        if let Some(element) = elements.next(cx)
            && let Ok(element) = V::Element::downcast_mut(element)
        {
            cx.tree().push(*id);

            self.rebuild(element, state, cx, data);

            cx.tree().pop();
        }
    }

    fn seq_message(
        elements: &mut impl Elements<C, E>,
        (state, id): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if message.is_taken() {
            return Action::new();
        }

        if let Some(element) = elements.next(cx)
            && let Ok(element) = V::Element::downcast_mut(element)
        {
            cx.tree().push(*id);

            let action = if let Some(id) = message.target()
                && !cx.tree().contains(id)
            {
                Action::new()
            } else {
                V::message(element, state, cx, data, message)
            };

            cx.tree().pop();

            action
        } else {
            Action::new()
        }
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, (state, id): Self::State, cx: &mut C) {
        if let Some(element) = elements.remove(cx)
            && let Ok(element) = V::Element::downcast(element)
        {
            cx.tree().push(id);

            V::teardown(element, state, cx);

            cx.tree().pop();
            cx.tree().remove(id);
        }
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for Option<V>
where
    E: Element,
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

    fn seq_message(
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        match state {
            Some(state) => V::seq_message(elements, state, cx, data, message),
            _ => Action::new(),
        }
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C) {
        if let Some(state) = state {
            V::seq_teardown(elements, state, cx);
        }
    }
}

impl<C, T, E, V, const N: usize> ViewSeq<C, T, E> for [V; N]
where
    E: Element,
    V: ViewSeq<C, T, E>,
{
    type State = [Option<V::State>; N];

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let mut states = [const { None }; N];

        for (i, view) in self.into_iter().enumerate() {
            let state = view.seq_build(elements, cx, data);
            states[i] = Some(state);
        }

        states
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        for (i, view) in self.into_iter().enumerate() {
            let state = state[i].as_mut().expect("all states are `Some`");
            view.seq_rebuild(elements, state, cx, data);
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

        for state in state.iter_mut() {
            let state = state.as_mut().expect("all states are `Some`");
            action |= V::seq_message(elements, state, cx, data, message);
        }

        action
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, state: Self::State, cx: &mut C) {
        for state in state.into_iter() {
            let state = state.expect("all states are `Some`");
            V::seq_teardown(elements, state, cx);
        }
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for Vec<V>
where
    E: Element,
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

    fn seq_message(
        elements: &mut impl Elements<C, E>,
        states: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        let mut action = Action::new();

        for state in states.iter_mut() {
            action |= V::seq_message(elements, state, cx, data, message);
        }

        action
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, states: Self::State, cx: &mut C) {
        for state in states {
            V::seq_teardown(elements, state, cx);
        }
    }
}

macro_rules! impl_tuple {
    ($($name:ident, $index:tt);*) => {
        #[allow(unused)]
        impl<C, T, E, $($name),*> ViewSeq<C, T, E> for ($($name,)*)
        where
            E: Element,
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

            fn seq_message(
                elements: &mut impl Elements<C, E>,
                state: &mut Self::State,
                cx: &mut C,
                data: &mut T,
                message: &mut Message,
            ) -> Action {
                let mut action = Action::new();

                $({
                    action |= $name::seq_message(
                        elements,
                        &mut state.$index,
                        cx,
                        data,
                        message,
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
