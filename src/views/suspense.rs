use std::{
    future::Future,
    mem,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

use crate::{Action, Base, Is, Message, Mut, Proxied, Proxy, Tracked, View, ViewId, ViewMarker};

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

    fn poll<'a, C, T>(
        element: Mut<'a, C::Element>,
        mut future: Pin<&mut F>,
        state: &mut SuspenseState<C, T, V, F::Output>,
        waker: &Waker,
        cx: &mut C,
        data: &mut T,
    ) -> Option<Mut<'a, C::Element>>
    where
        C: Base,
        V: View<C, T>,
        V::Element: Is<C, C::Element>,
        F: Future,
        F::Output: View<C, T>,
        FutElement<F, C, T>: Is<C, C::Element>,
    {
        let Poll::Ready(view) = future.as_mut().poll(&mut Context::from_waker(waker)) else {
            return Some(element);
        };

        if let SuspenseState::Contents(state) = state {
            if let Ok(element) = FutElement::<F, C, T>::downcast_mut(element) {
                view.rebuild(element, state, cx, data);
            }

            return None;
        }

        let (new_element, new_state) = view.build(cx, data);
        let element = Is::replace(cx, element, new_element);
        let state = mem::replace(
            state,
            SuspenseState::Contents(new_state),
        );

        match state {
            SuspenseState::Fallback(state) => {
                if let Ok(element) = V::Element::downcast(element) {
                    V::teardown(element, state, cx);
                }
            }

            SuspenseState::Contents(state) => {
                if let Ok(element) = FutElement::<F, C, T>::downcast(element) {
                    F::Output::teardown(element, state, cx);
                }
            }
        }

        None
    }

    fn waker<P>(proxy: P, view: ViewId) -> Waker
    where
        P: Proxy,
    {
        struct ProxyWaker<P> {
            proxy: P,
            view:  ViewId,
        }

        impl<P> Wake for ProxyWaker<P>
        where
            P: Proxy,
        {
            fn wake(self: Arc<Self>) {
                self.proxy.message(Message::new(ShouldPoll, self.view));
            }
        }

        Waker::from(Arc::new(ProxyWaker { proxy, view }))
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

type FutElement<F, C, T> = <<F as Future>::Output as View<C, T>>::Element;

struct ShouldPoll;

impl<V, F> ViewMarker for Suspense<V, F> {}
impl<C, T, V, F> View<C, T> for Suspense<V, F>
where
    C: Tracked + Proxied + Base,
    V: View<C, T>,
    F: Future + 'static,
    F::Output: View<C, T>,
    V::Element: Is<C, C::Element>,
    FutElement<F, C, T>: Is<C, C::Element>,
{
    type Element = C::Element;
    type State = (
        ViewId,
        Pin<Box<F>>,
        Waker,
        SuspenseState<C, T, V, F::Output>,
    );

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let view_id = ViewId::next();
        cx.register(view_id);

        let waker = Self::waker(cx.proxy(), view_id);
        let mut future = Box::pin(self.future);

        let (element, state) = match future.as_mut().poll(&mut Context::from_waker(&waker)) {
            Poll::Pending => {
                let (element, state) = self.fallback.build(cx, data);

                let element = Is::upcast(cx, element);
                let state = SuspenseState::Fallback(state);

                (element, state)
            }

            Poll::Ready(view) => {
                let (element, state) = view.build(cx, data);

                let element = Is::upcast(cx, element);
                let state = SuspenseState::Contents(state);

                (element, state)
            }
        };

        (element, (view_id, future, waker, state))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (_id, future, waker, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        future.as_mut().set(self.future);

        let element = Self::poll(
            element,
            future.as_mut(),
            state,
            waker,
            cx,
            data,
        );

        if let SuspenseState::Fallback(state) = state
            && let Some(element) = element
            && let Ok(element) = V::Element::downcast_mut(element)
        {
            self.fallback.rebuild(element, state, cx, data);
        }
    }

    fn message(
        element: Mut<'_, Self::Element>,
        (id, future, waker, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(ShouldPoll) = message.take(*id) {
            Self::poll(
                element,
                future.as_mut(),
                state,
                waker,
                cx,
                data,
            );

            return Action::new();
        }

        match state {
            SuspenseState::Fallback(state) => match V::Element::downcast_mut(element) {
                Ok(element) => V::message(element, state, cx, data, message),
                Err(_) => Action::new(),
            },

            SuspenseState::Contents(state) => match FutElement::<F, C, T>::downcast_mut(element) {
                Ok(element) => F::Output::message(element, state, cx, data, message),
                Err(_) => Action::new(),
            },
        }
    }

    fn teardown(element: Self::Element, (id, _future, _waker, state): Self::State, cx: &mut C) {
        match state {
            SuspenseState::Fallback(state) => {
                if let Ok(element) = V::Element::downcast(element) {
                    V::teardown(element, state, cx);
                }
            }

            SuspenseState::Contents(state) => {
                if let Ok(element) = FutElement::<F, C, T>::downcast(element) {
                    F::Output::teardown(element, state, cx);
                }
            }
        }

        cx.unregister(id);
    }
}
