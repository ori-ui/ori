use std::mem;

use crate::{Action, AsyncContext, Event, Key, Proxy, Super, SuperElement, View};

struct SuspenseFuture<V>(V);

/// [`View`] that displays a `fallback` until a `future` completes.
///
/// Note that future is spawned on every rebuild, so using a [`Memo`](crate::views::Memo) or
/// [`Freeze`](crate::views::freeze) is strongly recommended.
///
/// # Example
///
/// ```ignore
/// let suspense = freeze(|| {
///     suspense(
///         label("Waiting for future to complete..."),
///         async {
///             Delay::new(Duration::from_secs(5)).await;
///             label("Future has now completed!")
///         },
///     )
/// });
/// ```
pub fn suspense<V, F>(fallback: V, future: F) -> Suspense<V, F> {
    Suspense::new(fallback, future)
}

/// [`View`] that displays a `fallback` until a `future` completes.
///
/// Note that future is spawned on every rebuild, so using a [`Memo`](crate::views::Memo) or
/// [`Freeze`](crate::views::freeze) is strongly recommended.
///
/// # Example
///
/// ```ignore
/// let suspense = freeze(|| {
///     suspense(
///         label("Waiting for future to complete..."),
///         async {
///             Delay::new(Duration::from_secs(5)).await;
///             label("Future has now completed!")
///         },
///     )
/// });
/// ```
pub struct Suspense<V, F> {
    fallback: V,
    future: Option<F>,
}

impl<V, F> Suspense<V, F> {
    /// Create new [`Suspense`].
    pub fn new(fallback: V, future: F) -> Self {
        Self {
            fallback,
            future: Some(future),
        }
    }
}

pub enum SuspenseState<C, T, V, F>
where
    V: View<C, T>,
    F: View<C, T>,
{
    Fallback(V::State),
    Content(F, F::State),
}

impl<C, T, V, F> View<C, T> for Suspense<V, F>
where
    C: AsyncContext + SuperElement,
    V: View<C, T>,
    F: Future + Send + 'static,
    F::Output: View<C, T> + Send,
    C::Element: Super<C, V::Element>,
    C::Element: Super<C, <F::Output as View<C, T>>::Element>,
{
    type Element = C::Element;
    type State = (Key, SuspenseState<C, T, V, F::Output>);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (fallback_element, fallback_state) = self.fallback.build(cx, data);

        let key = Key::next();

        if let Some(future) = self.future.take() {
            let proxy = cx.proxy();

            cx.proxy().spawn(async move {
                let content = future.await;
                proxy.event(Event::new(SuspenseFuture(content), key));
            });
        }

        let element = C::Element::upcast(cx, fallback_element);
        let state = SuspenseState::Fallback(fallback_state);
        (element, (key, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (key, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) -> bool {
        if let Some(future) = self.future.take() {
            let proxy = cx.proxy();
            let key = *key;

            cx.proxy().spawn(async move {
                let content = future.await;
                proxy.event(Event::new(SuspenseFuture(content), key));
            });
        }

        match state {
            SuspenseState::Fallback(fallback_state) => element.downcast_with(|element| {
                self.fallback.rebuild(
                    element,
                    fallback_state,
                    cx,
                    data,
                    &mut old.fallback,
                )
            }),

            SuspenseState::Content(_, _) => false,
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (_key, state): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        match state {
            SuspenseState::Fallback(fallback_state) => {
                self.fallback.teardown(
                    element.downcast(),
                    fallback_state,
                    cx,
                    data,
                );
            }

            SuspenseState::Content(mut content, content_state) => {
                content.teardown(
                    element.downcast(),
                    content_state,
                    cx,
                    data,
                );
            }
        }
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (key, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> (bool, Action) {
        let element_changed = match event.take_targeted(*key) {
            Some(SuspenseFuture::<F::Output>(mut content)) => match state {
                SuspenseState::Fallback(_) => {
                    let (content_element, content_state) = content.build(cx, data);

                    let fallback_element = mem::replace(
                        element,
                        C::Element::upcast(cx, content_element),
                    );

                    let SuspenseState::Fallback(fallback_state) = mem::replace(
                        state,
                        SuspenseState::Content(content, content_state),
                    ) else {
                        unreachable!()
                    };

                    self.fallback.teardown(
                        fallback_element.downcast(),
                        fallback_state,
                        cx,
                        data,
                    );

                    true
                }

                SuspenseState::Content(old_content, content_state) => {
                    // rustfmt please
                    element.downcast_with(|element| {
                        content.rebuild(
                            element,
                            content_state,
                            cx,
                            data,
                            old_content,
                        )
                    })
                }
            },

            _ => false,
        };

        let (changed, action) = match state {
            SuspenseState::Fallback(fallback_state) => element.downcast_with(|element| {
                self.fallback
                    .event(element, fallback_state, cx, data, event)
            }),

            SuspenseState::Content(content, content_state) => {
                // rustfmt please
                element
                    .downcast_with(|element| content.event(element, content_state, cx, data, event))
            }
        };

        (changed || element_changed, action)
    }
}
