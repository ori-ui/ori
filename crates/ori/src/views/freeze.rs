use crate::{Action, Context, Event, View};

/// [`View`] that doesn't rebuild when state changes.
pub fn freeze<C, T, F, V>(build: F) -> Freeze<F>
where
    C: Context,
    V: View<C, T>,
    F: FnOnce() -> V,
{
    Freeze::new(build)
}

/// [`View`] that doesn't rebuild when state changes.
#[must_use]
pub struct Freeze<F> {
    build: Option<F>,
}

impl<F> Freeze<F> {
    /// Crate a new [`Freeze`].
    pub fn new<C, T, V>(build: F) -> Self
    where
        C: Context,
        V: View<C, T>,
        F: FnOnce() -> V,
    {
        Self { build: Some(build) }
    }
}

impl<C, T, F, V> View<C, T> for Freeze<F>
where
    C: Context,
    V: View<C, T>,
    F: FnOnce() -> V,
{
    type Element = V::Element;
    type State = (V, V::State);

    fn build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let build = self.build.take().unwrap();
        let mut view = build();
        let (element, state) = view.build(cx, data);

        (element, (view, state))
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _old: &mut Self,
    ) {
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (mut view, state): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        view.teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        view.event(element, state, cx, data, event)
    }
}
