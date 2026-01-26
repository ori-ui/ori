use crate::{Action, Element, Event, Mut};

/// Trait restricting implementations of [`View`].
pub trait ViewMarker {}

/// Snapshot of the state of a retained UI.
pub trait View<C, T>: ViewMarker
where
    T: ?Sized,
{
    /// The element this view produces.
    type Element: Element;

    /// The state of this view.
    type State;

    /// Create [`Self::Element`] and [`Self::State`].
    ///
    /// This is expected to be called only once per instance of [`View`].
    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State);

    /// Rebuild the UI, applying the differences between `self` and `old`.
    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    );

    /// Handle an [`Event`].
    ///
    /// Returns whether the element has changed in a way that might invalidate the parent child
    /// relation as well as an [`Action`] to execute.
    fn event(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action;

    /// Tear down the UI built by the [`View`].
    ///
    /// This is expected to be called only once per instance of [`View`].
    fn teardown(element: Self::Element, state: Self::State, cx: &mut C);
}
