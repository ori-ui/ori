use std::mem;

use crate::{
    Action, Base, Event, Mut, Proxied, Proxy, Sub, View, ViewId, ViewMarker,
    future::{Abortable, Aborter},
};

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
    future:   F,
}

impl<V, F> Suspense<V, F> {
    /// Create new [`Suspense`].
    pub fn new(fallback: V, future: F) -> Self {
        Self { fallback, future }
    }
}

pub enum SuspenseState<C, T, V, F>
where
    V: View<C, T>,
    F: View<C, T>,
{
    Fallback(V::State),
    Contents(F::State),
}

impl<V, F> ViewMarker for Suspense<V, F> {}
impl<C, T, V, F> View<C, T> for Suspense<V, F>
where
    C: Proxied + Base,
    V: View<C, T>,
    F: Future + Send + 'static,
    F::Output: View<C, T> + Send,
    V::Element: Sub<C, C::Element>,
    <F::Output as View<C, T>>::Element: Sub<C, C::Element>,
{
    type Element = C::Element;
    type State = (
        ViewId,
        Aborter,
        SuspenseState<C, T, V, F::Output>,
    );

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (fallback_element, fallback_state) = self.fallback.build(cx, data);

        let id = ViewId::next();

        let proxy = cx.proxy();

        let (future, handle) = Abortable::new(async move {
            let contents = self.future.await;
            proxy.event(Event::new(SuspenseFuture(contents), id));
        });

        cx.proxy().spawn(future);

        let element = V::Element::upcast(cx, fallback_element);
        let state = SuspenseState::Fallback(fallback_state);
        (element, (id, handle, state))
    }

    fn rebuild(
        self,
        element: Mut<C, Self::Element>,
        (id, handle, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        handle.abort();

        let proxy = cx.proxy();
        let id = *id;

        let (future, new_handle) = Abortable::new(async move {
            let contents = self.future.await;
            proxy.event(Event::new(SuspenseFuture(contents), id));
        });

        cx.proxy().spawn(future);
        *handle = new_handle;

        match state {
            SuspenseState::Fallback(fallback_state) => {
                V::Element::downcast_mut(cx, element, |cx, element| {
                    self.fallback.rebuild(element, fallback_state, cx, data);
                });
            }

            SuspenseState::Contents(_) => {}
        }
    }

    fn event(
        element: Mut<C, Self::Element>,
        (id, _handle, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        if let Some(SuspenseFuture::<F::Output>(contents)) = event.take_targeted(*id) {
            match state {
                SuspenseState::Fallback(_) => {
                    let (contents_element, contents_state) = contents.build(cx, data);
                    let fallback_element = Sub::replace(cx, element, contents_element);

                    let SuspenseState::Fallback(fallback_state) = mem::replace(
                        state,
                        SuspenseState::Contents(contents_state),
                    ) else {
                        unreachable!()
                    };

                    if let Some(fallback_element) = Sub::downcast(cx, fallback_element) {
                        V::teardown(fallback_element, fallback_state, cx);
                    }
                }

                SuspenseState::Contents(contents_state) => {
                    <<F::Output as View<_, _>>::Element>::downcast_mut(
                        cx,
                        element,
                        |cx, element| contents.rebuild(element, contents_state, cx, data),
                    );
                }
            }

            return Action::new();
        };

        match state {
            SuspenseState::Fallback(fallback_state) => {
                V::Element::downcast_mut(cx, element, |cx, element| {
                    V::event(element, fallback_state, cx, data, event)
                })
                .unwrap_or(Action::new())
            }

            SuspenseState::Contents(contents_state) => {
                <<F::Output as View<_, _>>::Element>::downcast_mut(cx, element, |cx, element| {
                    <F::Output as View<_, _>>::event(element, contents_state, cx, data, event)
                })
                .unwrap_or(Action::new())
            }
        }
    }

    fn teardown(element: Self::Element, (_id, handle, state): Self::State, cx: &mut C) {
        handle.abort();

        match state {
            SuspenseState::Fallback(fallback_state) => {
                if let Some(element) = Sub::downcast(cx, element) {
                    V::teardown(element, fallback_state, cx);
                }
            }

            SuspenseState::Contents(contents_state) => {
                if let Some(element) = Sub::downcast(cx, element) {
                    <F::Output as View<C, T>>::teardown(element, contents_state, cx);
                }
            }
        }
    }
}
