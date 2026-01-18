use std::any::Any;

use crate::{Action, AnyView, BaseElement, Event, Mut, View, ViewMarker};

/// Marker view for types implementing [`Build`].
pub trait BuildMarker {}

/// Helper trait for implementing the builder pattern for [`View`]s.
pub trait Build<C, T>: BuildMarker
where
    C: BaseElement,
{
    /// Build the [`View`] of this builder.
    fn build(&mut self) -> impl AnyView<C, T, C::Element> + 'static;
}

impl<V> ViewMarker for V where V: BuildMarker {}

impl<C, T, B> View<C, T> for B
where
    C: BaseElement,
    B: Build<C, T>,
{
    type Element = C::Element;
    type State = (
        Box<dyn AnyView<C, T, C::Element>>,
        Box<dyn Any>,
    );

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let mut view = self.build();
        let (element, state) = view.any_build(cx, data);

        let view = Box::new(view);
        let state = Box::new(state);
        (element, (view, state))
    }

    fn rebuild(
        &mut self,
        element: Mut<C, Self::Element>,
        (view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        _old: &mut Self,
    ) {
        let mut new_view = self.build();
        new_view.any_rebuild(element, state, cx, data, view.as_mut());
        *view = Box::new(new_view);
    }

    fn event(
        &mut self,
        element: Mut<C, Self::Element>,
        (view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        view.as_mut().any_event(element, state, cx, data, event)
    }

    fn teardown(&mut self, element: Self::Element, (mut view, state): Self::State, cx: &mut C) {
        view.as_mut().any_teardown(element, state, cx);
    }
}
