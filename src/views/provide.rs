use std::{any::Any, marker::PhantomData};

use crate::{Action, Message, Mut, Provider, View, ViewMarker};

/// [`View`] that provides a `resource` to a [`View`], see [`using`] for how to use contexts.
#[must_use]
pub fn provide<U, C, T, V>(
    initial: impl FnOnce(&T) -> U,
    contents: V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: Provider,
    V: View<C, T>,
{
    Provide::new(initial, contents)
}

/// [`View`] that uses `resource` provided by [`provide`].
#[must_use]
pub fn using<U, C, T, V>(build: impl FnOnce(&T, &U) -> V) -> impl View<C, T, Element = V::Element>
where
    U: Any,
    C: Provider,
    V: View<C, T>,
{
    try_using(move |data, context| {
        let context = context.expect(
            "`using` expects context to be provided, try providing it with `provide` or use `using_or_default` or `try_using` instead",
        );

        build(data, context)
    })
}

/// [`View`] that uses `resource` provided by [`provide`].
#[must_use]
pub fn using_or_default<U, C, T, V>(
    build: impl FnOnce(&T, &U) -> V,
) -> impl View<C, T, Element = V::Element>
where
    U: Any + Default,
    C: Provider,
    V: View<C, T>,
{
    try_using(move |data, context| match context {
        Some(context) => build(data, context),
        None => build(data, &Default::default()),
    })
}

/// [`View`] that uses `resource` provided by [`provide`].
#[must_use]
pub fn try_using<U, C, T, V>(
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
pub struct Provide<F, U, V> {
    contents: V,
    initial:  F,
    marker:   PhantomData<fn() -> U>,
}

impl<F, U, V> Provide<F, U, V> {
    /// Create a [`Provide`].
    pub fn new<T>(initial: F, contents: V) -> Self
    where
        F: FnOnce(&T) -> U,
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
    F: FnOnce(&T) -> U,
    U: Any,
    C: Provider,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = (Option<Box<U>>, V::State);

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let context = (self.initial)(data);

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
impl<F, U, C, T, V> View<C, T> for Using<F, U>
where
    F: FnOnce(&T, Option<&U>) -> V,
    U: Any,
    C: Provider,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = V::State;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let context = cx.get::<U>();

        let view = (self.build)(data, context);
        view.build(cx, data)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let context = cx.get::<U>();
        let view = (self.build)(data, context);
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
