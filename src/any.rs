use std::{any::Any, mem};

use crate::{Action, Element, Event, Mut, Super, View, ViewMarker};

/// Type erased [`View`].
pub trait AnyView<C, T, E>
where
    E: Element<C>,
{
    /// Build type erased state.
    fn build(self: Box<Self>, cx: &mut C, data: &mut T) -> (E, AnyState<C, T, E>);

    /// Rebuild type erased state.
    fn rebuild(
        self: Box<Self>,
        element: Mut<C, E>,
        state: &mut AnyState<C, T, E>,
        cx: &mut C,
        data: &mut T,
    );
}

impl<C, T, E, V> AnyView<C, T, E> for V
where
    E: Super<C, V::Element>,
    V: View<C, T>,
    V::State: 'static,
{
    fn build(self: Box<Self>, cx: &mut C, data: &mut T) -> (E, AnyState<C, T, E>) {
        let (element, state) = V::build(*self, cx, data);
        let element = E::upcast(cx, element);
        let state = AnyState {
            state:    Box::new(state),
            event:    AnyState::<C, T, E>::event::<V>,
            teardown: AnyState::<C, T, E>::teardown::<V>,
        };

        (element, state)
    }

    fn rebuild(
        self: Box<Self>,
        element: Mut<C, E>,
        state: &mut AnyState<C, T, E>,
        cx: &mut C,
        data: &mut T,
    ) {
        match state.state.downcast_mut() {
            Some(view_state) => E::downcast_with(element, |element| {
                V::rebuild(*self, element, view_state, cx, data);
            }),

            None => {
                let (new_element, new_state) = V::build(*self, cx, data);
                let old_element = E::replace(cx, element, new_element);
                let old_state = mem::replace(&mut state.state, Box::new(new_state));
                (state.teardown)(old_element, old_state, cx);

                state.event = AnyState::<C, T, E>::event::<V>;
                state.teardown = AnyState::<C, T, E>::teardown::<V>;
            }
        }
    }
}

/// Type erased state, see [`AnyView`].
#[allow(clippy::type_complexity)]
pub struct AnyState<C, T, E>
where
    E: Element<C>,
{
    state:    Box<dyn Any>,
    event:    fn(Mut<C, E>, &mut dyn Any, &mut C, &mut T, &mut Event) -> Action,
    teardown: fn(E, Box<dyn Any>, &mut C),
}

impl<C, T, E> AnyState<C, T, E>
where
    E: Element<C>,
{
    fn event<V>(
        element: Mut<C, E>,
        state: &mut dyn Any,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action
    where
        E: Super<C, V::Element>,
        V: View<C, T>,
        V::State: 'static,
    {
        if let Some(state) = state.downcast_mut() {
            E::downcast_with(element, |element| {
                V::event(element, state, cx, data, event)
            })
        } else {
            Action::new()
        }
    }

    fn teardown<V>(element: E, state: Box<dyn Any>, cx: &mut C)
    where
        E: Super<C, V::Element>,
        V: View<C, T>,
        V::State: 'static,
    {
        let element = E::downcast(element);

        if let Ok(state) = state.downcast() {
            V::teardown(element, *state, cx);
        }
    }
}

impl<C, T, E> ViewMarker for Box<dyn AnyView<C, T, E>> {}
impl<C, T, E> View<C, T> for Box<dyn AnyView<C, T, E>>
where
    E: Element<C>,
{
    type Element = E;
    type State = AnyState<C, T, E>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        AnyView::build(self, cx, data)
    }

    fn rebuild(
        self,
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        AnyView::rebuild(self, element, state, cx, data)
    }

    fn event(
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        (state.event)(
            element,
            state.state.as_mut(),
            cx,
            data,
            event,
        )
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        (state.teardown)(element, state.state, cx)
    }
}
