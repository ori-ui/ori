use std::{any::Any, marker::PhantomData};

use crate::{Action, Event, ProviderContext, View, ViewMarker};

/// [`View`] that provides a context to a contents view, see [`using`] for how to use contexts.
pub fn provide<U, C, T, V>(
    initial: impl FnMut(&mut T) -> U,
    contents: V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: ProviderContext,
    V: View<C, T>,
{
    Provide::new(initial, contents)
}

/// [`View`] that provides a context to a contents view, see [`Using`] for how to use contexts.
pub struct Provide<F, U, V> {
    contents: V,
    initial:  F,
    marker:   PhantomData<fn() -> U>,
}

impl<F, U, V> Provide<F, U, V> {
    /// Create a [`Provide`].
    pub fn new<T>(initial: F, contents: V) -> Self
    where
        F: FnMut(&mut T) -> U,
    {
        Self {
            contents,
            initial,
            marker: PhantomData,
        }
    }
}

impl<F, U, V> ViewMarker for Provide<F, U, V> {}
impl<F, U, C, T, V> View<C, T> for Provide<F, U, V>
where
    F: FnMut(&mut T) -> U,
    U: Any,
    C: ProviderContext,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = (Option<Box<U>>, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let context = (self.initial)(data);

        cx.push_context(Box::new(context));
        let (element, state) = self.contents.build(cx, data);
        let context = cx.pop_context();

        (element, (context, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (context, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        if let Some(context) = context.take() {
            cx.push_context(context);
        }

        self.contents.rebuild(
            element,
            state,
            cx,
            data,
            &mut old.contents,
        );

        *context = cx.pop_context();
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (context, state): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        if let Some(context) = context {
            cx.push_context(context);
        }

        self.contents.teardown(element, state, cx, data);

        cx.pop_context::<U>();
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (context, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        if let Some(context) = context.take() {
            cx.push_context(context);
        }

        let action = self.contents.event(element, state, cx, data, event);

        *context = cx.pop_context();

        action
    }
}

/// [`View`] that uses context provided by [`provide`].
pub fn using<U, C, T, V>(
    build: impl FnOnce(&mut T, &U) -> V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: ProviderContext,
    V: View<C, T>,
{
    try_using(move |data, context| {
        let context = context.expect(
            "`using` expects context to be provided, try providing it with `provide` or use `using_or_default` or `try_using` instead",
        );

        build(data, context)
    })
}

/// [`View`] that uses context provided by [`provide`].
pub fn using_or_default<U, C, T, V>(
    build: impl FnOnce(&mut T, &U) -> V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any + Default,
    C: ProviderContext,
    V: View<C, T>,
{
    try_using(move |data, context| match context {
        Some(context) => build(data, context),
        None => build(data, &Default::default()),
    })
}

/// [`View`] that uses context provided by [`provide`].
pub fn try_using<U, C, T, V>(
    build: impl FnOnce(&mut T, Option<&U>) -> V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: ProviderContext,
    V: View<C, T>,
{
    Using::new(build)
}

/// [`View`] that uses context provided by [`Provide`].
pub struct Using<F, U> {
    build:  Option<F>,
    marker: PhantomData<fn(&U)>,
}

impl<F, U> Using<F, U> {
    /// Create a [`Using`].
    pub fn new(build: F) -> Self {
        Self {
            build:  Some(build),
            marker: PhantomData,
        }
    }
}

impl<F, U> ViewMarker for Using<F, U> {}
impl<F, U, C, T, V> View<C, T> for Using<F, U>
where
    F: FnOnce(&mut T, Option<&U>) -> V,
    U: Any,
    C: ProviderContext,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = (V, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let context = cx.get_context::<U>();

        let build = self.build.take().expect("build should only be called once");
        let mut view = build(data, context);
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
        let context = cx.get_context::<U>();

        if let Some(build) = self.build.take() {
            let mut new_view = build(data, context);
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
