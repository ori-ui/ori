use crate::{Action, Element, Message, Mut};

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

    /// Rebuild the UI to match the new view tree.
    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    );

    /// Handle a [`Message`].
    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action;

    /// Tear down the UI built by the [`View`].
    ///
    /// This is expected to be called only once per instance of [`View`].
    fn teardown(element: Self::Element, state: Self::State, cx: &mut C);
}
