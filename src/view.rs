use crate::{Action, Event};

/// Trait restricting implementations of [`View`].
pub trait ViewMarker {}

/// Snapshot of the state of a retained UI.
pub trait View<C, T>: ViewMarker
where
    T: ?Sized,
{
    /// The element this view produces.
    type Element;

    /// The state of this view.
    type State;

    /// Create [`Self::Element`] and [`Self::State`].
    ///
    /// This is expected to be called only once per instance of [`View`].
    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State);

    /// Rebuild the UI, applying the differences between `self` and `old`.
    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    );

    /// Tear down the UI built by the [`View`].
    ///
    /// This is expected to be called only once per instance of [`View`].
    fn teardown(&mut self, element: Self::Element, state: Self::State, cx: &mut C);

    /// Handle an [`Event`].
    ///
    /// Returns whether the element has changed in a way that might invalidate the parent child
    /// relation as well as an [`Action`] to execute.
    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action;
}
