use std::{any::Any, mem};

use crate::{Action, Element, Event, Mut, Super, View, ViewMarker};

/// A type erased [`View`].
///
/// Note that this generic over `E` which should be treated as a [`Super`] element of the elements
/// supported by this implementation.
pub trait AnyView<C, T, E>
where
    T: ?Sized,
    E: Element<C>,
{
    /// Get `self` as `&mut dyn Any`.
    ///
    /// This unfortunately is still necessary, even after the stabilization of casting `dyn` trait
    /// to super trait, due to lifetime issues.
    fn as_mut_any(&mut self) -> &mut dyn Any;

    /// Build in a type erased manner, see [`View::build`] for more details.
    fn any_build(&mut self, cx: &mut C, data: &mut T) -> (E, Box<dyn Any>);

    /// Rebuild in a type erased manner, see [`View::rebuild`] for more details.
    fn any_rebuild(
        &mut self,
        element: E::Mut<'_>,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        old: &mut dyn AnyView<C, T, E>,
    );

    /// Handle event in a type erased manner, see [`View::event`] for more details.
    fn any_event(
        &mut self,
        element: E::Mut<'_>,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action;

    /// Tear down in a type erased manner, see [`View::teardown`] for more details.
    fn any_teardown(&mut self, element: E, state: Box<dyn Any>, cx: &mut C);
}

impl<C, T, E, V> AnyView<C, T, E> for V
where
    T: ?Sized,
    E: Super<C, V::Element>,
    V: View<C, T> + Any,
    V::State: Any,
{
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn any_build(&mut self, cx: &mut C, data: &mut T) -> (E, Box<dyn Any>) {
        let (element, state) = self.build(cx, data);

        (E::upcast(cx, element), Box::new(state))
    }

    fn any_rebuild(
        &mut self,
        element: E::Mut<'_>,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        old: &mut dyn AnyView<C, T, E>,
    ) {
        match old.as_mut_any().downcast_mut::<V>() {
            Some(old) => E::downcast_with(element, |element| {
                if let Some(state) = state.downcast_mut() {
                    self.rebuild(element, state, cx, data, old);
                }
            }),

            None => {
                let (new_element, new_state) = self.build(cx, data);

                old.any_teardown(
                    E::replace(cx, element, new_element),
                    mem::replace(state, Box::new(new_state)),
                    cx,
                );
            }
        }
    }

    fn any_event(
        &mut self,
        element: E::Mut<'_>,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        E::downcast_with(element, |element| {
            if let Some(state) = state.downcast_mut() {
                self.event(element, state, cx, data, event)
            } else {
                Action::new()
            }
        })
    }

    fn any_teardown(&mut self, element: E, state: Box<dyn Any>, cx: &mut C) {
        if let Ok(state) = state.downcast() {
            self.teardown(element.downcast(), *state, cx);
        }
    }
}

impl<C, T, E> ViewMarker for Box<dyn AnyView<C, T, E>> where T: ?Sized {}
impl<C, T, E> View<C, T> for Box<dyn AnyView<C, T, E>>
where
    T: ?Sized,
    E: Element<C>,
{
    type Element = E;
    type State = Box<dyn Any>;

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        self.as_mut().any_build(cx, data)
    }

    fn rebuild(
        &mut self,
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.as_mut()
            .any_rebuild(element, state, cx, data, old.as_mut());
    }

    fn event(
        &mut self,
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        self.as_mut().any_event(element, state, cx, data, event)
    }

    fn teardown(&mut self, element: Self::Element, state: Self::State, cx: &mut C) {
        self.as_mut().any_teardown(element, state, cx);
    }
}
