use crate::{Action, Event, Super, View};

/// A sequence of [`View`]s.
pub trait ViewSeq<C, T, E> {
    /// The elements this view produces.
    type Elements: ElementSeq<E>;
    /// State of the sequence.
    type States;

    /// Build [`Self::Elements`] and [`Self::States`], see [`View::build`] for more information.
    fn seq_build(&mut self, cx: &mut C, data: &mut T) -> (Self::Elements, Self::States);

    /// Rebuild the sequence, see [`View::rebuild`] for more information.
    fn seq_rebuild(
        &mut self,
        elements: &mut Self::Elements,
        states: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    );

    /// Tear down the sequence, see [`View::teardown`] for more information.
    fn seq_teardown(
        &mut self,
        elements: Self::Elements,
        states: Self::States,
        cx: &mut C,
        data: &mut T,
    );

    /// Handle an event for the sequence, see [`View::event`] for more information.
    ///
    /// Returns a list of indices of elements that have change in a way that might invalidate the
    /// parent child relation.
    fn seq_event(
        &mut self,
        elements: &mut Self::Elements,
        states: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action;
}

/// A sequence of elements, see [`ViewSeq`] for more details.
pub trait ElementSeq<E> {
    /// Create an [`Iterator`] over all the elements in the sequence.
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a E>
    where
        E: 'a;

    /// Create an [`Iterator`] over all the elements in the sequence.
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E>
    where
        E: 'a;

    /// Compute the number of elements in the sequence.
    fn count(&self) -> usize {
        self.iter().count()
    }
}

#[doc(hidden)]
pub struct One<E>(pub E);

impl<E> ElementSeq<E> for One<E> {
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a E>
    where
        E: 'a,
    {
        std::iter::once(&self.0)
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E>
    where
        E: 'a,
    {
        std::iter::once(&mut self.0)
    }

    fn count(&self) -> usize {
        1
    }
}

impl<T, U> ElementSeq<T> for Option<U>
where
    U: ElementSeq<T>,
{
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.iter().flat_map(ElementSeq::iter)
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        self.iter_mut().flat_map(ElementSeq::iter_mut)
    }

    fn count(&self) -> usize {
        self.as_ref().map_or(0, |e| e.count())
    }
}

impl<T, U, const SIZE: usize> ElementSeq<T> for [U; SIZE]
where
    U: ElementSeq<T>,
{
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.as_slice().iter().flat_map(ElementSeq::iter)
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        self.as_mut_slice()
            .iter_mut()
            .flat_map(ElementSeq::iter_mut)
    }

    fn count(&self) -> usize {
        self.as_slice().iter().map(ElementSeq::count).sum()
    }
}

impl<T, U> ElementSeq<T> for Vec<U>
where
    U: ElementSeq<T>,
{
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.as_slice().iter().flat_map(ElementSeq::iter)
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        self.as_mut_slice()
            .iter_mut()
            .flat_map(ElementSeq::iter_mut)
    }

    fn count(&self) -> usize {
        self.as_slice().iter().map(ElementSeq::count).sum()
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for V
where
    V: View<C, T>,
    E: Super<C, V::Element>,
{
    type Elements = One<E>;
    type States = V::State;

    fn seq_build(&mut self, cx: &mut C, data: &mut T) -> (Self::Elements, Self::States) {
        let (element, state) = self.build(cx, data);
        let elements = One(E::upcast(cx, element));

        (elements, state)
    }

    fn seq_rebuild(
        &mut self,
        One(element): &mut Self::Elements,
        state: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        element.downcast_with(|element| self.rebuild(element, state, cx, data, old));
    }

    fn seq_teardown(
        &mut self,
        One(element): Self::Elements,
        state: Self::States,
        cx: &mut C,
        data: &mut T,
    ) {
        self.teardown(element.downcast(), state, cx, data);
    }

    fn seq_event(
        &mut self,
        One(elements): &mut Self::Elements,
        state: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        elements.downcast_with(|element| self.event(element, state, cx, data, event))
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for Option<V>
where
    V: ViewSeq<C, T, E>,
{
    type Elements = Option<V::Elements>;
    type States = Option<V::States>;

    fn seq_build(&mut self, cx: &mut C, data: &mut T) -> (Self::Elements, Self::States) {
        match self {
            Some(contents) => {
                let (children, states) = contents.seq_build(cx, data);
                (Some(children), Some(states))
            }

            None => (None, None),
        }
    }

    fn seq_rebuild(
        &mut self,
        elements: &mut Self::Elements,
        states: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        match (self, old) {
            (None, None) => {}

            (None, Some(old)) => {
                let elements = elements.take().unwrap();
                let states = states.take().unwrap();

                old.seq_teardown(elements, states, cx, data);
            }

            (Some(contents), None) => {
                let (new_elements, new_states) = contents.seq_build(cx, data);
                *elements = Some(new_elements);
                *states = Some(new_states);
            }

            (Some(contents), Some(old)) => {
                let elements = elements.as_mut().unwrap();
                let states = states.as_mut().unwrap();

                contents.seq_rebuild(elements, states, cx, data, old);
            }
        }
    }

    fn seq_teardown(
        &mut self,
        elements: Self::Elements,
        state: Self::States,
        cx: &mut C,
        data: &mut T,
    ) {
        if let Some(contents) = self {
            let element = elements.unwrap();
            let state = state.unwrap();
            contents.seq_teardown(element, state, cx, data);
        }
    }

    fn seq_event(
        &mut self,
        elements: &mut Self::Elements,
        state: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        match self {
            Some(contents) => {
                let elements = elements.as_mut().unwrap();
                let state = state.as_mut().unwrap();
                contents.seq_event(elements, state, cx, data, event)
            }

            None => Action::new(),
        }
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for Vec<V>
where
    V: ViewSeq<C, T, E>,
{
    type Elements = Vec<V::Elements>;
    type States = Vec<V::States>;

    fn seq_build(&mut self, cx: &mut C, data: &mut T) -> (Self::Elements, Self::States) {
        let mut elements = Vec::with_capacity(self.len());
        let mut states = Vec::with_capacity(self.len());

        for view in self {
            let (element, state) = view.seq_build(cx, data);
            elements.push(element);
            states.push(state);
        }

        (elements, states)
    }

    fn seq_rebuild(
        &mut self,
        elements: &mut Self::Elements,
        states: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        let mut diff = Vec::new();

        if self.len() < old.len() {
            for ((old, element), state) in old
                .iter_mut()
                .skip(self.len())
                .zip(elements.drain(self.len()..))
                .zip(states.drain(self.len()..))
            {
                for _ in 0..element.count() {
                    diff.push(self.len());
                }

                old.seq_teardown(element, state, cx, data);
            }

            elements.truncate(self.len());
            states.truncate(self.len());
        }

        for (i, view) in self.iter_mut().enumerate() {
            if let Some(old) = old.get_mut(i) {
                view.seq_rebuild(
                    &mut elements[i],
                    &mut states[i],
                    cx,
                    data,
                    old,
                );
            } else {
                let (element, state) = view.seq_build(cx, data);

                elements.push(element);
                states.push(state);
            }
        }
    }

    fn seq_teardown(
        &mut self,
        elements: Self::Elements,
        states: Self::States,
        cx: &mut C,
        data: &mut T,
    ) {
        for ((view, elements), states) in self.iter_mut().zip(elements).zip(states) {
            view.seq_teardown(elements, states, cx, data);
        }
    }

    fn seq_event(
        &mut self,
        elements: &mut Self::Elements,
        states: &mut Self::States,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let mut action = Action::new();

        for (i, view) in self.iter_mut().enumerate() {
            action |= view.seq_event(
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
        impl<E, $($name,)*> ElementSeq<E> for ($($name,)*)
        where
            $($name: ElementSeq<E>,)*
        {
            fn iter<'a>(&'a self) -> impl Iterator<Item = &'a E>
            where
                E: 'a,
            {
                None.into_iter()$(.chain(self.$index.iter()))*
            }

            fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E>
            where
                E: 'a,
            {
                None.into_iter()$(.chain(self.$index.iter_mut()))*
            }

            fn count(&self) -> usize {
                0 $(+ self.$index.count())*
            }
        }

        #[allow(unused)]
        impl<C, T, E, $($name),*> ViewSeq<C, T, E> for ($($name,)*)
        where
            $($name: ViewSeq<C, T, E>,)*
        {
            type Elements = ($($name::Elements,)*);
            type States = ($($name::States,)*);

            fn seq_build(
                &mut self,
                cx: &mut C,
                data: &mut T,
            ) -> (Self::Elements, Self::States) {
                let pairs = ($(self.$index.seq_build(cx, data),)*);

                (($(pairs.$index.0,)*), ($(pairs.$index.1,)*))
            }

            fn seq_rebuild(
                &mut self,
                elements: &mut Self::Elements,
                state: &mut Self::States,
                cx: &mut C,
                data: &mut T,
                old: &mut Self,
            ) {
                $({
                    self.$index.seq_rebuild(
                        &mut elements.$index,
                        &mut state.$index,
                        cx,
                        data,
                        &mut old.$index,
                    );
                })*
            }

            fn seq_teardown(
                &mut self,
                elements: Self::Elements,
                state: Self::States,
                cx: &mut C,
                data: &mut T,
            ) {
                $({
                    self.$index.seq_teardown(
                        elements.$index,
                        state.$index,
                        cx,
                        data,
                    );
                })*
            }

            fn seq_event(
                &mut self,
                elements: &mut Self::Elements,
                state: &mut Self::States,
                cx: &mut C,
                data: &mut T,
                event: &mut Event,
            ) -> Action {
                let mut action = Action::new();

                $({
                    action |= self.$index.seq_event(
                        &mut elements.$index,
                        &mut state.$index,
                        cx,
                        data,
                        event,
                    );
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
