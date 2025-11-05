use std::marker::PhantomData;

use crate::{Action, Event, View};

pub fn focus<F, U, V, T>(content: V, focus: F) -> Focus<F, U, V>
where
    F: FnMut(&mut T, &mut Lens<U>),
{
    Focus::new(content, focus)
}

pub type Lens<'a, T> = dyn FnMut(&mut T) + 'a;

#[must_use]
pub struct Focus<F, U, V> {
    content: V,
    focus: F,
    marker: PhantomData<fn(&U)>,
}

impl<F, U, V> Focus<F, U, V> {
    pub fn new<T>(content: V, focus: F) -> Self
    where
        F: FnMut(&mut T, &mut Lens<U>),
    {
        Self {
            content,
            focus,
            marker: PhantomData,
        }
    }
}

impl<C, T, U, V, F> View<C, T> for Focus<F, U, V>
where
    F: FnMut(&mut T, &mut Lens<U>),
    V: View<C, U>,
{
    type Element = V::Element;
    type State = V::State;

    fn build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let mut state = None;

        (self.focus)(data, &mut |data| {
            state = Some(self.content.build(cx, data));
        });

        state.expect("focus calls lens")
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        let mut called = false;

        (self.focus)(data, &mut |data| {
            self.content.rebuild(
                element,
                state,
                cx,
                data,
                &mut old.content,
            );

            called = true;
        });

        assert!(called, "focus must call lens");
    }

    fn teardown(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let mut called = false;

        (self.focus)(data, &mut |data| {
            self.content.teardown(element, state, cx, data);
            called = true;
        });

        assert!(called, "focus must call lens");
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let mut action = None;

        (self.focus)(data, &mut |data| {
            action = Some(self.content.event(element, state, cx, data, event));
        });

        action.expect("focus calls lens")
    }
}
