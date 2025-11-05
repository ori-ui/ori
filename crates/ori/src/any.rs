use std::any::Any;

use crate::{Action, Event, Super, View};

pub trait AnyView<C, E, T> {
    fn as_mut_any(&mut self) -> &mut dyn Any;

    fn any_build(&mut self, cx: &mut C, data: &mut T) -> (E, Box<dyn Any>);

    fn any_rebuild(
        &mut self,
        element: &mut E,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        old: &mut dyn AnyView<C, E, T>,
    );

    fn any_teardown(
        &mut self,
        element: &mut E,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
    );

    fn any_event(
        &mut self,
        element: &mut E,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action;
}

impl<C, E, T, V> AnyView<C, E, T> for V
where
    V: View<C, T> + Any,
    E: Super<C, V::Element>,
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
        element: &mut E,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        old: &mut dyn AnyView<C, E, T>,
    ) {
        match old.as_mut_any().downcast_mut::<V>() {
            Some(old) => {
                element.downcast(|element| {
                    let state = state.downcast_mut().unwrap();
                    self.rebuild(element, state, cx, data, old);
                });
            }

            None => {
                old.any_teardown(element, state, cx, data);

                let (new_element, new_state) = self.build(cx, data);
                *element = E::upcast(cx, new_element);
                *state = Box::new(new_state);
            }
        }
    }

    fn any_teardown(
        &mut self,
        element: &mut E,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
    ) {
        element.downcast(|element| {
            let state = state.downcast_mut().unwrap();
            self.teardown(element, state, cx, data);
        });
    }

    fn any_event(
        &mut self,
        element: &mut E,
        state: &mut Box<dyn Any>,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        element.downcast(|element| {
            let state = state.downcast_mut().unwrap();
            self.event(element, state, cx, data, event)
        })
    }
}

impl<C, E, T> View<C, T> for dyn AnyView<C, E, T> {
    type Element = E;
    type State = Box<dyn Any>;

    fn build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        self.any_build(cx, data)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.any_rebuild(element, state, cx, data, old);
    }

    fn teardown(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.any_teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        self.any_event(element, state, cx, data, event)
    }
}
