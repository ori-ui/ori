use std::{marker::PhantomData, sync::Arc};

use crate::{
    Action, AsyncContext, Effect, Event, IntoAction, NoElement, Proxy, View, ViewId, ViewMarker,
};

/// [`Effect`](crate::Effect) that spawns a `task` that emits events to a `handler`.
pub fn task<C, T, E, F, A, I>(
    task: impl FnOnce(&mut T, Sink<E>) -> F + 'static,
    mut handler: impl FnMut(&mut T, E) -> A + 'static,
) -> impl Effect<C, T>
where
    C: AsyncContext,
    E: Send + 'static,
    F: Future<Output = ()> + Send + 'static,
    A: IntoAction<I>,
{
    Task {
        task:    Some(task),
        handler: move |data: &mut T, event| handler(data, event).into_action(),
        marker:  PhantomData,
    }
}

/// Sink for sending events to the `handler` of a [`task`].
#[derive(Clone)]
pub struct Sink<E>
where
    E: Send + 'static,
{
    id:     ViewId,
    proxy:  Arc<dyn Proxy>,
    marker: PhantomData<fn(E)>,
}

impl<E> Sink<E>
where
    E: Send + 'static,
{
    /// Send an `event` to the `handler` of the [`task`].
    pub fn send(&mut self, event: E) {
        self.proxy.event(Event::new(event, self.id))
    }
}

/// [`Effect`](crate::Effect) that spawns a `task` that send events to a `handler`.
pub struct Task<E, F, G> {
    task:    Option<F>,
    handler: G,
    marker:  PhantomData<fn(E)>,
}

impl<E, F, G> ViewMarker for Task<E, F, G> {}
impl<C, T, E, F, G, H> View<C, T> for Task<E, F, G>
where
    C: AsyncContext,
    E: Send + 'static,
    F: FnOnce(&mut T, Sink<E>) -> H,
    G: FnMut(&mut T, E) -> Action,
    H: Future<Output = ()> + Send + 'static,
{
    type Element = NoElement;
    type State = ViewId;

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let task = self.task.take().expect("build should only be called once");

        let id = ViewId::next();
        let proxy = cx.proxy();
        let sink = Sink {
            id,
            proxy: proxy.cloned(),
            marker: PhantomData,
        };

        let future = task(data, sink);
        proxy.spawn(future);

        (NoElement, id)
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _old: &mut Self,
    ) {
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        _state: Self::State,
        _cx: &mut C,
        _data: &mut T,
    ) {
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        id: &mut Self::State,
        _cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        if let Some(event) = event.take_targeted(*id) {
            (self.handler)(data, event)
        } else {
            Action::new()
        }
    }
}
