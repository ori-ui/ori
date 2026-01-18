use crate::{Action, Event, Mut, View, ViewMarker};

/// [`View`] that doesn't rebuild when state changes.
pub fn freeze<V>(build: impl FnOnce() -> V) -> Freeze<impl FnOnce() -> V> {
    Freeze::new(build)
}

/// [`View`] that doesn't rebuild when state changes.
#[must_use]
pub struct Freeze<F> {
    build: Option<F>,
}

impl<F> Freeze<F> {
    /// Crate a new [`Freeze`].
    pub fn new<V>(build: F) -> Self
    where
        F: FnOnce() -> V,
    {
        Self { build: Some(build) }
    }
}

impl<F> ViewMarker for Freeze<F> {}
impl<C, T, F, V> View<C, T> for Freeze<F>
where
    V: View<C, T>,
    F: FnOnce() -> V,
{
    type Element = V::Element;
    type State = (V, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let build = self.build.take().expect("build should only be called once");
        let mut view = build();
        let (element, state) = view.build(cx, data);

        (element, (view, state))
    }

    fn rebuild(
        &mut self,
        _element: Mut<C, Self::Element>,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _old: &mut Self,
    ) {
    }

    fn event(
        &mut self,
        element: Mut<C, Self::Element>,
        (view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        view.event(element, state, cx, data, event)
    }

    fn teardown(&mut self, element: Self::Element, (mut view, state): Self::State, cx: &mut C) {
        view.teardown(element, state, cx);
    }
}
