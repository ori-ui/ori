use std::mem;

use crate::{Action, AsyncContext, BaseElement, Event, Proxy, Super, View, ViewId, ViewMarker};

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
#[must_use]
pub struct Suspense<V, F> {
    fallback: V,
    future:   Option<F>,
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
    Contents(F, F::State),
}

impl<V, F> ViewMarker for Suspense<V, F> {}
impl<C, T, V, F> View<C, T> for Suspense<V, F>
where
    C: AsyncContext + BaseElement,
    V: View<C, T>,
    F: Future + Send + 'static,
    F::Output: View<C, T> + Send,
    C::Element: Super<C, V::Element>,
    C::Element: Super<C, <F::Output as View<C, T>>::Element>,
{
    type Element = C::Element;
    type State = (
        ViewId,
        SuspenseState<C, T, V, F::Output>,
    );

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (fallback_element, fallback_state) = self.fallback.build(cx, data);

        let id = ViewId::next();

        if let Some(future) = self.future.take() {
            let proxy = cx.proxy();

            cx.proxy().spawn(async move {
                let contents = future.await;
                proxy.event(Event::new(SuspenseFuture(contents), id));
            });
        }

        let element = C::Element::upcast(cx, fallback_element);
        let state = SuspenseState::Fallback(fallback_state);
        (element, (id, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (id, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        if let Some(future) = self.future.take() {
            let proxy = cx.proxy();
            let key = *id;

            cx.proxy().spawn(async move {
                let contents = future.await;
                proxy.event(Event::new(
                    SuspenseFuture(contents),
                    key,
                ));
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
                );
            }),

            SuspenseState::Contents(_, _) => {}
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (_id, state): Self::State,
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

            SuspenseState::Contents(mut contents, contents_state) => {
                contents.teardown(
                    element.downcast(),
                    contents_state,
                    cx,
                    data,
                );
            }
        }
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (id, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        if let Some(SuspenseFuture::<F::Output>(mut contents)) = event.take_targeted(*id) {
            match state {
                SuspenseState::Fallback(_) => {
                    let (contents_element, contents_state) = contents.build(cx, data);

                    let fallback_element = mem::replace(
                        element,
                        C::Element::upcast(cx, contents_element),
                    );

                    let SuspenseState::Fallback(fallback_state) = mem::replace(
                        state,
                        SuspenseState::Contents(contents, contents_state),
                    ) else {
                        unreachable!()
                    };

                    self.fallback.teardown(
                        fallback_element.downcast(),
                        fallback_state,
                        cx,
                        data,
                    );
                }

                SuspenseState::Contents(old_contents, contents_state) => {
                    element.downcast_with(|element| {
                        contents.rebuild(
                            element,
                            contents_state,
                            cx,
                            data,
                            old_contents,
                        );
                    });

                    *old_contents = contents;
                }
            }
        };

        match state {
            SuspenseState::Fallback(fallback_state) => element.downcast_with(|element| {
                self.fallback
                    .event(element, fallback_state, cx, data, event)
            }),

            SuspenseState::Contents(contents, contents_state) => {
                element.downcast_with(|element| {
                    contents.event(element, contents_state, cx, data, event)
                    //
                })
            }
        }
    }
}
