use crate::{Action, Event, Super, View};

/// A sequence of [`View`]s.
pub trait ViewSeq<C, T, E> {
    /// State of the sequence.
    type SeqState;

    /// Build elements and [`Self::SeqState`], see [`View::build`] for more information.
    fn seq_build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Vec<E>, Self::SeqState);

    /// Rebuild the sequence, see [`View::rebuild`] for more information.
    fn seq_rebuild(
        &mut self,
        elements: &mut Vec<E>,
        state: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) -> bool;

    /// Tear down the sequence, see [`View::teardown`] for more information.
    fn seq_teardown(
        &mut self,
        elements: Vec<E>,
        state: Self::SeqState,
        cx: &mut C,
        data: &mut T,
    );

    /// Handle an event for the sequence, see [`View::event`] for more information.
    fn seq_event(
        &mut self,
        elements: &mut [E],
        state: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> (bool, Action);
}

impl<C, T, E, V> ViewSeq<C, T, E> for Option<V>
where
    V: View<C, T>,
    E: Super<C, V::Element>,
{
    type SeqState = Option<V::State>;

    fn seq_build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Vec<E>, Self::SeqState) {
        match self {
            Some(content) => {
                let (child, state) = content.build(cx, data);
                (vec![E::upcast(cx, child)], Some(state))
            }

            None => (Vec::new(), None),
        }
    }

    fn seq_rebuild(
        &mut self,
        elements: &mut Vec<E>,
        state: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) -> bool {
        match (self, old) {
            (None, None) => false,

            (None, Some(old)) => {
                let element = elements.pop().unwrap().downcast();
                let state = state.take().unwrap();
                old.teardown(element, state, cx, data);

                true
            }

            (Some(content), None) => {
                let (child, new_state) = content.build(cx, data);
                elements.push(E::upcast(cx, child));
                *state = Some(new_state);

                true
            }

            (Some(content), Some(old)) => {
                elements[0].downcast_with(|element| {
                    let state = state.as_mut().unwrap();
                    content.rebuild(element, state, cx, data, old)
                })
            }
        }
    }

    fn seq_teardown(
        &mut self,
        mut elements: Vec<E>,
        state: Self::SeqState,
        cx: &mut C,
        data: &mut T,
    ) {
        if let Some(content) = self {
            let element = elements.pop().unwrap().downcast();
            let state = state.unwrap();
            content.teardown(element, state, cx, data);
        }
    }

    fn seq_event(
        &mut self,
        elements: &mut [E],
        state: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> (bool, Action) {
        match self {
            Some(content) => elements[0].downcast_with(|element| {
                let state = state.as_mut().unwrap();
                content.event(element, state, cx, data, event)
            }),

            None => (false, Action::new()),
        }
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for Vec<V>
where
    V: View<C, T>,
    E: Super<C, V::Element>,
{
    type SeqState = Vec<V::State>;

    fn seq_build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Vec<E>, Self::SeqState) {
        let mut elements = Vec::with_capacity(self.len());
        let mut states = Vec::with_capacity(self.len());

        for view in self {
            let (element, state) = view.build(cx, data);
            elements.push(E::upcast(cx, element));
            states.push(state);
        }

        (elements, states)
    }

    fn seq_rebuild(
        &mut self,
        elements: &mut Vec<E>,
        states: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) -> bool {
        let mut elements_changed = false;

        if self.len() < old.len() {
            for ((old, element), state) in old
                .iter_mut()
                .skip(self.len())
                .zip(elements.drain(self.len()..))
                .zip(states.drain(self.len()..))
            {
                old.teardown(element.downcast(), state, cx, data);
            }

            elements.truncate(self.len());
            states.truncate(self.len());

            elements_changed = true;
        }

        for (i, view) in self.iter_mut().enumerate() {
            if let Some(old) = old.get_mut(i) {
                elements[i].downcast_with(|element| {
                    elements_changed |=
                        view.rebuild(element, &mut states[i], cx, data, old);
                });
            } else {
                let (element, state) = view.build(cx, data);
                elements.push(E::upcast(cx, element));
                states.push(state);

                elements_changed = true;
            }
        }

        elements_changed
    }

    fn seq_teardown(
        &mut self,
        elements: Vec<E>,
        states: Self::SeqState,
        cx: &mut C,
        data: &mut T,
    ) {
        for ((view, element), state) in
            self.iter_mut().zip(elements).zip(states)
        {
            view.teardown(element.downcast(), state, cx, data);
        }
    }

    fn seq_event(
        &mut self,
        elements: &mut [E],
        states: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> (bool, Action) {
        let mut changed = false;
        let mut action = Action::new();

        for (i, view) in self.iter_mut().enumerate() {
            elements[i].downcast_with(|element| {
                let (element_changed, element_action) =
                    view.event(element, &mut states[i], cx, data, event);

                changed |= element_changed;
                action |= element_action;
            });
        }

        (changed, action)
    }
}

macro_rules! impl_tuple {
    ($($name:ident, $index:tt);*) => {
        #[allow(unused)]
        impl<C, T, E, $($name),*> ViewSeq<C, T, E> for ($($name,)*)
        where
            $($name: View<C, T>,)*
            $(E: Super<C, $name::Element>,)*
        {
            type SeqState = ($($name::State,)*);

            fn seq_build(
                &mut self,
                cx: &mut C,
                data: &mut T,
            ) -> (Vec<E>, Self::SeqState) {
                let mut elements = Vec::with_capacity(0$(.max($index + 1))*);

                let state = ($({
                    let (element, state) = self.$index.build(cx, data);
                    elements.push(E::upcast(cx, element));
                    state
                },)*);

                (elements, state)
            }

            fn seq_rebuild(
                &mut self,
                elements: &mut Vec<E>,
                state: &mut Self::SeqState,
                cx: &mut C,
                data: &mut T,
                old: &mut Self,
            ) -> bool {
                let mut changed = false;

                $({
                    elements[$index].downcast_with(|element| {
                        changed |= self.$index.rebuild(
                            element,
                            &mut state.$index,
                            cx,
                            data,
                            &mut old.$index,
                        );
                    });
                })*

                changed
            }

            fn seq_teardown(
                &mut self,
                mut elements: Vec<E>,
                state: Self::SeqState,
                cx: &mut C,
                data: &mut T,
            ) {
                $({
                    let element = elements.remove(0);
                    self.$index.teardown(
                        element.downcast(),
                        state.$index,
                        cx,
                        data,
                    );
                })*
            }

            fn seq_event(
                &mut self,
                elements: &mut [E],
                state: &mut Self::SeqState,
                cx: &mut C,
                data: &mut T,
                event: &mut Event,
            ) -> (bool, Action) {
                let mut changed = false;
                let mut action = Action::new();

                $({
                    elements[$index].downcast_with(|element| {
                        let (element_changed, element_action) = self.$index.event(
                            element,
                            &mut state.$index,
                            cx,
                            data,
                            event,
                        );

                        changed |= element_changed;
                        action |= element_action;
                    });
                })*

                (changed, action)
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
