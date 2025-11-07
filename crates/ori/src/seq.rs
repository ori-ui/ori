use crate::{Action, Context, Event, Super, View};

/// A sequence of [`View`]s.
pub trait ViewSeq<C: Context, E, T> {
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
    );

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
    ) -> Action;
}

impl<C, T, V> ViewSeq<C, V::Element, T> for Vec<V>
where
    C: Context,
    V: View<C, T>,
{
    type SeqState = Vec<V::State>;

    fn seq_build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Vec<V::Element>, Self::SeqState) {
        let mut elements = Vec::with_capacity(self.len());
        let mut states = Vec::with_capacity(self.len());

        for view in self {
            let (element, state) = view.build(cx, data);
            elements.push(element);
            states.push(state);
        }

        (elements, states)
    }

    fn seq_rebuild(
        &mut self,
        elements: &mut Vec<V::Element>,
        states: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        if self.len() < old.len() {
            for ((old, element), state) in old
                .iter_mut()
                .skip(self.len())
                .zip(elements.drain(self.len()..))
                .zip(states.drain(self.len()..))
            {
                old.teardown(element, state, cx, data);
            }

            elements.truncate(self.len());
            states.truncate(self.len());
        }

        for (i, view) in self.iter_mut().enumerate() {
            if let Some(old) = old.get_mut(i) {
                view.rebuild(
                    &mut elements[i],
                    &mut states[i],
                    cx,
                    data,
                    old,
                );
            } else {
                let (element, state) = view.build(cx, data);
                elements.push(element);
                states.push(state);
            }
        }
    }

    fn seq_teardown(
        &mut self,
        elements: Vec<V::Element>,
        states: Self::SeqState,
        cx: &mut C,
        data: &mut T,
    ) {
        for ((view, element), state) in
            self.iter_mut().zip(elements).zip(states)
        {
            view.teardown(element, state, cx, data);
        }
    }

    fn seq_event(
        &mut self,
        elements: &mut [V::Element],
        states: &mut Self::SeqState,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let mut action = Action::none();

        for (i, view) in self.iter_mut().enumerate() {
            action |= view.event(
                &mut elements[i],
                &mut states[i],
                cx,
                data,
                event,
            );
        }

        action
    }
}

macro_rules! impl_tuple {
    ($($name:ident, $index:tt);*) => {
        #[allow(unused)]
        impl<C, E, T, $($name),*> ViewSeq<C, E, T> for ($($name,)*)
        where
            C: Context,
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
            ) {
                $({
                    elements[$index].downcast_with(|element| {
                        self.$index.rebuild(
                            element,
                            &mut state.$index,
                            cx,
                            data,
                            &mut old.$index,
                        );
                    });
                })*
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
            ) -> Action {
                let mut action = Action::none();

                $({
                    action |= elements[$index].downcast_with(|element| {
                        self.$index.event(
                            element,
                            &mut state.$index,
                            cx,
                            data,
                            event,
                        )
                    });
                })*

                action
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
