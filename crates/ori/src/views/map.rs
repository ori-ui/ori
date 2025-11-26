use std::marker::PhantomData;

use crate::{Action, Event, View, ViewMarker};

/// [`View`] that maps one type of data to another.
pub fn map<C, T, U, E>(
    contents: impl View<C, U, Element = E>,
    map: impl FnMut(&mut T, &mut dyn FnMut(&mut U)),
) -> impl View<C, T, Element = E> {
    Map::new(contents, map)
}

/// [`View`] that maps one type of data to another.
#[must_use]
pub struct Map<F, U, V> {
    contents: V,
    map:      F,
    marker:   PhantomData<fn(&U)>,
}

impl<F, U, V> Map<F, U, V> {
    /// Create a [`Map`].
    pub fn new<T>(contents: V, map: F) -> Self
    where
        F: FnMut(&mut T, &mut dyn FnMut(&mut U)),
    {
        Self {
            contents,
            map,
            marker: PhantomData,
        }
    }
}

impl<F, U, V> ViewMarker for Map<F, U, V> {}
impl<C, T, U, V, F> View<C, T> for Map<F, U, V>
where
    V: View<C, U>,
    F: FnMut(&mut T, &mut dyn FnMut(&mut U)),
{
    type Element = V::Element;
    type State = V::State;

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let mut state = None;

        (self.map)(data, &mut |data| {
            state = Some(self.contents.build(cx, data));
        });

        state.expect("map calls lens")
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

        (self.map)(data, &mut |data| {
            self.contents.rebuild(
                element,
                state,
                cx,
                data,
                &mut old.contents,
            );

            called = true;
        });
    }

    fn teardown(&mut self, element: Self::Element, state: Self::State, cx: &mut C, data: &mut T) {
        let mut called = false;

        let mut element = Some(element);
        let mut state = Some(state);

        (self.map)(data, &mut |data| {
            if let (Some(element), Some(state)) = (element.take(), state.take()) {
                self.contents.teardown(element, state, cx, data);
                called = true;
            }
        });
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

        (self.map)(data, &mut |data| {
            action = Some(self.contents.event(element, state, cx, data, event));
        });

        action.unwrap_or(Action::new())
    }
}
