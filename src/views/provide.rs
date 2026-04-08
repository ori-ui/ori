use std::{any::Any, marker::PhantomData};

use crate::{Action, Message, Mut, Provider, View, ViewMarker};

/// [`View`] that provides a `resource` to a [`View`], see [`using`] for how to use contexts.
pub fn provide<C, T, U, V>(state: U, contents: V) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: Provider,
    V: View<C, T>,
{
    Provide::new(state, contents)
}

/// [`View`] that uses `resource` provided by [`provide`].
pub fn using<C, T, U, V>(build: impl FnOnce(&T, &U) -> V) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: Provider,
    V: View<C, T>,
{
    try_using(move |data, resource| {
        let resource = resource.expect(
            "`using` expects resource to be provided, try providing it with `provide` or use `using_or_default` or `try_using` instead",
        );

        build(data, resource)
    })
}

/// [`View`] that uses `resource` provided by [`provide`].
pub fn using_or_default<C, T, U, V>(
    build: impl FnOnce(&T, &U) -> V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any + Default,
    C: Provider,
    V: View<C, T>,
{
    try_using(move |data, resource| match resource {
        Some(resource) => build(data, resource),
        None => build(data, &Default::default()),
    })
}

/// [`View`] that uses `resource` provided by [`provide`].
pub fn try_using<C, T, U, V>(
    build: impl FnOnce(&T, Option<&U>) -> V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: Provider,
    V: View<C, T>,
{
    Using::new(build)
}

/// [`View`] that provides a `resource` to a [`View`], see [`Using`] for how to use contexts.
#[must_use]
pub struct Provide<U, V> {
    state:    U,
    contents: V,
}

impl<U, V> Provide<U, V> {
    /// Create a [`Provide`].
    pub fn new(state: U, contents: V) -> Self {
        Self { state, contents }
    }
}

impl<U, V> ViewMarker for Provide<U, V> {}
impl<C, T, U, V> View<C, T> for Provide<U, V>
where
    C: Provider,
    U: Any,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = (Option<Box<U>>, V::State);

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let context = self.state;

        cx.push(Box::new(context));
        let (element, state) = self.contents.build(cx, data);
        let context = cx.pop();

        (element, (context, state))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (context, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        match context {
            Some(context) => **context = self.state,
            None => *context = Some(Box::new(self.state)),
        }

        if let Some(context) = context.take() {
            cx.push(context);
        }

        self.contents.rebuild(element, state, cx, data);

        *context = cx.pop();
    }

    fn message(
        element: Mut<'_, Self::Element>,
        (context, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(context) = context.take() {
            cx.push(context);
        }

        let action = V::message(element, state, cx, data, message);

        *context = cx.pop();

        action
    }

    fn teardown(element: Self::Element, (context, state): Self::State, cx: &mut C) {
        if let Some(context) = context {
            cx.push(context);
        }

        V::teardown(element, state, cx);

        cx.pop::<U>();
    }
}

/// [`View`] that uses `resource` provided by [`Provide`].
#[must_use]
pub struct Using<F, U> {
    build:  F,
    marker: PhantomData<fn(&U)>,
}

impl<F, U> Using<F, U> {
    /// Create a [`Using`].
    pub fn new(build: F) -> Self {
        Self {
            build,
            marker: PhantomData,
        }
    }
}

impl<F, U> ViewMarker for Using<F, U> {}
impl<F, C, T, U, V> View<C, T> for Using<F, U>
where
    F: FnOnce(&T, Option<&U>) -> V,
    C: Provider,
    U: Any,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = V::State;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let resource = cx.get::<U>();

        let view = (self.build)(data, resource);
        view.build(cx, data)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let resource = cx.get::<U>();
        let view = (self.build)(data, resource);
        view.rebuild(element, state, cx, data);
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        V::message(element, state, cx, data, message)
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        V::teardown(element, state, cx);
    }
}
