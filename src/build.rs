use crate::{Action, AnyState, AnyView, BaseElement, Event, Mut, View, ViewMarker};

/// Marker view for types implementing [`Build`].
pub trait BuildMarker {}

/// Helper trait for implementing the builder pattern for [`View`]s.
pub trait Build<C, T>: BuildMarker
where
    C: BaseElement,
{
    /// Build the [`View`] of this builder.
    fn build(self) -> impl AnyView<C, T, C::Element>;
}

impl<V> ViewMarker for V where V: BuildMarker {}

impl<C, T, B> View<C, T> for B
where
    C: BaseElement,
    B: Build<C, T>,
{
    type Element = C::Element;
    type State = AnyState<C, T, C::Element>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let view = self.build();
        AnyView::build(Box::new(view), cx, data)
    }

    fn rebuild(
        self,
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let view = self.build();
        AnyView::rebuild(Box::new(view), element, state, cx, data);
    }

    fn event(
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        Box::<dyn AnyView<C, T, C::Element>>::event(element, state, cx, data, event)
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        Box::<dyn AnyView<C, T, C::Element>>::teardown(element, state, cx);
    }
}
