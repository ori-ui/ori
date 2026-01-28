use crate::{Action, AnyState, AnyView, Base, Message, Mut, View, ViewMarker};

/// Marker view for types implementing [`Build`].
pub trait BuildMarker {}

/// Helper trait for implementing the builder pattern for [`View`]s.
pub trait BuildView<C, T>: BuildMarker
where
    C: Base,
{
    /// Build the [`View`] of this builder.
    fn build(self) -> impl AnyView<C, T, C::Element>;
}

impl<V> ViewMarker for V where V: BuildMarker {}

impl<C, T, B> View<C, T> for B
where
    C: Base,
    B: BuildView<C, T>,
{
    type Element = C::Element;
    type State = AnyState<C, T, C::Element>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let view = self.build();
        AnyView::build(Box::new(view), cx, data)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let view = self.build();
        AnyView::rebuild(Box::new(view), element, state, cx, data);
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        Box::<dyn AnyView<C, T, C::Element>>::message(element, state, cx, data, message)
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        Box::<dyn AnyView<C, T, C::Element>>::teardown(element, state, cx);
    }
}
