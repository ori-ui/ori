use std::any::Any;

use crate::{Action, AnyView, Event, NoElement, View, ViewMarker, ViewSeq};

/// A [`View`] that has [`NoElement`] and can therefore only produce side-effects.
///
/// Implemented for all [`View`]s with an element of [`NoElement`].
pub trait Effect<C, T>: View<C, T, Element = NoElement> {}

/// A sequence of [`Effect`]s.
///
/// Implemented for all [`ViewSeq`]s with an element of [`NoElement`].
pub trait EffectSeq<C, T>: ViewSeq<C, T, NoElement> {}

impl<C, T, V> Effect<C, T> for V where V: View<C, T, Element = NoElement> {}
impl<C, T, V> EffectSeq<C, T> for V where V: ViewSeq<C, T, NoElement> {}

/// Type erased [`Effect`](crate::Effect).
pub trait AnyEffect<C, T>: AnyView<C, T, NoElement> {}

impl<C, T, V> AnyEffect<C, T> for V where V: AnyView<C, T, NoElement> {}

impl<C, T> ViewMarker for Box<dyn AnyEffect<C, T>> {}
impl<C, T> View<C, T> for Box<dyn AnyEffect<C, T>> {
    type Element = NoElement;
    type State = Box<dyn Any>;

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        self.as_mut().any_build(cx, data)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.as_mut()
            .any_rebuild(element, state, cx, data, old.as_mut());
    }

    fn teardown(&mut self, element: Self::Element, state: Self::State, cx: &mut C, data: &mut T) {
        self.as_mut().any_teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        self.as_mut().any_event(element, state, cx, data, event)
    }
}
