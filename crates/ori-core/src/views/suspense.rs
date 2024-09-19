use std::{cell::RefCell, future::Future};

use crate::{
    context::{BaseCx, BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::{Pod, State, View},
};

/// Create a new [`Suspense`] view.
pub fn suspense<F>(future: F) -> Suspense<(), F>
where
    F: Future + Send + 'static,
{
    Suspense::new(future)
}

/// A view that suspends rendering while a future is pending.
pub struct Suspense<V, F> {
    fallback: Pod<V>,
    future: Option<F>,
}

impl<F> Suspense<(), F> {
    /// Create a new [`Suspense`] view.
    pub fn new(future: F) -> Self
    where
        F: Future + Send + 'static,
    {
        Self {
            fallback: Pod::new(()),
            future: Some(future),
        }
    }
}

impl<F> Suspense<(), F> {
    /// Set the fallback view to display while the future is pending.
    pub fn fallback<V>(self, fallback: V) -> Suspense<V, F> {
        Suspense {
            fallback: Pod::new(fallback),
            future: self.future,
        }
    }
}

#[doc(hidden)]
pub struct SuspenseState<T, F, V>
where
    V: View<T>,
    F: Future,
    F::Output: View<T>,
{
    id: SuspenseId,
    fallback_state: Option<State<T, V>>,
    future: Option<Pod<F::Output>>,
    future_state: Option<State<T, F::Output>>,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct SuspenseId(usize);

struct SuspenseCompleted<T> {
    id: SuspenseId,
    view: RefCell<Option<T>>,
}

impl<T, V, F> View<T> for Suspense<V, F>
where
    V: View<T>,
    F: Future + Send + 'static,
    F::Output: View<T> + Send,
{
    type State = SuspenseState<T, F, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let id = spawn(&mut self.future, cx);

        let fallback_state = self.fallback.build(cx, data);

        SuspenseState {
            id,
            fallback_state: Some(fallback_state),
            future: None,
            future_state: None,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        state.id = spawn(&mut self.future, cx);

        if let (Some(fallback_state), None) = (&mut state.fallback_state, &mut state.future_state) {
            (self.fallback).rebuild(fallback_state, cx, data, &old.fallback);
        }
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        if let Some(completed) = event.cmd::<SuspenseCompleted<F::Output>>() {
            if completed.id == state.id {
                let mut view = completed.view.borrow_mut().take().map(Pod::new);
                state.future_state = view.as_mut().map(|v| v.build(&mut cx.as_build_cx(), data));
                state.future = view;

                state.fallback_state.take();

                cx.layout();
            }
        }

        match (
            &mut state.fallback_state,
            &mut state.future,
            &mut state.future_state,
        ) {
            (None, Some(fut), Some(fut_state)) => fut.event(fut_state, cx, data, event),
            (Some(fallback_state), _, _) => self.fallback.event(fallback_state, cx, data, event),
            _ => false,
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        match (
            &mut state.fallback_state,
            &mut state.future,
            &mut state.future_state,
        ) {
            (None, Some(fut), Some(fut_state)) => fut.layout(fut_state, cx, data, space),
            (Some(fallback_state), _, _) => self.fallback.layout(fallback_state, cx, data, space),
            _ => Size::ZERO,
        }
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        match (
            &mut state.fallback_state,
            &mut state.future,
            &mut state.future_state,
        ) {
            (None, Some(fut), Some(fut_state)) => fut.draw(fut_state, cx, data),
            (Some(fallback_state), _, _) => self.fallback.draw(fallback_state, cx, data),
            _ => {}
        }
    }
}

fn spawn<F>(future: &mut Option<F>, cx: &mut BaseCx) -> SuspenseId
where
    F: Future + Send + 'static,
    F::Output: Send,
{
    let future = future.take().expect("future not taken");

    let id = *cx.context_or_default::<SuspenseId>();
    cx.context_or_default::<SuspenseId>().0 += 1;

    cx.cmd_async({
        async move {
            let view = RefCell::new(Some(future.await));
            SuspenseCompleted { id, view }
        }
    });

    id
}
