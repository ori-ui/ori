use crate::{Action, Event, View, ViewMarker};

/// [`View`] that is built from a callback.
pub fn builder<C, T, V, F>(build: F) -> Builder<F>
where
    F: FnOnce(&mut C, &mut T) -> V,
    V: View<C, T>,
{
    Builder::new(build)
}

/// [`View`] that is built from a callback.
pub struct Builder<F> {
    build: Option<F>,
}

impl<F> Builder<F> {
    /// Create a [`Builder`].
    pub fn new<C, T, V>(build: F) -> Self
    where
        F: FnOnce(&mut C, &mut T) -> V,
        V: View<C, T>,
    {
        Self { build: Some(build) }
    }
}

impl<F> ViewMarker for Builder<F> {}
impl<C, T, V, F> View<C, T> for Builder<F>
where
    F: FnOnce(&mut C, &mut T) -> V,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = (V, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let build = self.build.take().unwrap();
        let mut view = build(cx, data);
        let (element, state) = view.build(cx, data);

        (element, (view, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        _old: &mut Self,
    ) {
        let build = self.build.take().unwrap();
        let mut new_view = build(cx, data);
        new_view.rebuild(element, state, cx, data, view);
        *view = new_view;
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
