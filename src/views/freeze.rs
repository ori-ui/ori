use crate::{Action, Event, Mut, View, ViewMarker};

/// [`View`] that doesn't rebuild when state changes.
pub fn freeze<V>(build: impl FnOnce() -> V) -> Freeze<impl FnOnce() -> V> {
    Freeze::new(build)
}

/// [`View`] that doesn't rebuild when state changes.
#[must_use]
pub struct Freeze<F> {
    build: F,
}

impl<F> Freeze<F> {
    /// Crate a new [`Freeze`].
    pub fn new<V>(build: F) -> Self
    where
        F: FnOnce() -> V,
    {
        Self { build }
    }
}

impl<F> ViewMarker for Freeze<F> {}
impl<C, T, F, V> View<C, T> for Freeze<F>
where
    V: View<C, T>,
    F: FnOnce() -> V,
{
    type Element = V::Element;
    type State = V::State;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let view = (self.build)();
        view.build(cx, data)
    }

    fn rebuild(
        self,
        _element: Mut<C, Self::Element>,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
    ) {
    }

    fn event(
        element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::event(element, state, cx, data, event)
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        V::teardown(element, state, cx);
    }
}
