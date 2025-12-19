use std::sync::Arc;

use crate::{
    Action, AsyncContext, Effect, Event, IntoAction, Proxy, View, ViewMarker, views::effects,
};

/// [`View`] that is built from a callback.
pub fn build<C, T, V>(build: impl FnOnce(&mut T) -> V) -> impl View<C, T, Element = V::Element>
where
    V: View<C, T>,
{
    build_with_context(move |_, data| build(data))
}

/// [`View`] that is built from a callback with access to the context.
pub fn build_with_context<C, T, V>(
    build: impl FnOnce(&mut C, &mut T) -> V,
) -> impl View<C, T, Element = V::Element>
where
    V: View<C, T>,
{
    Builder::new(build)
}

/// [`Effect`] that runs a task with access to a [`Proxy`] when it's built.
pub fn task<C, T, A, I>(task: impl FnOnce(&mut T, Arc<dyn Proxy>) -> A) -> impl Effect<C, T>
where
    C: AsyncContext,
    A: IntoAction<I>,
{
    build_with_context(|cx: &mut C, data| {
        let proxy = cx.proxy().cloned();
        let action = task(data, proxy.clone());
        proxy.action(action.into_action());

        effects(())
    })
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
        let build = self.build.take().expect("build should only be called once");
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
        if let Some(build) = self.build.take() {
            let mut new_view = build(cx, data);
            new_view.rebuild(element, state, cx, data, view);
            *view = new_view;
        }
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
