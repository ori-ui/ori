use std::{marker::PhantomData, sync::Arc};

use crate::{
    Action, Effect, Event, Mut, Proxied, Proxy, View, ViewId, ViewMarker,
    future::{Abortable, Aborter},
};

/// [`Effect`](crate::Effect) that spawns a `task` that emits events to a `handler`.
pub fn task<C, T, E, F, A>(
    task: impl FnOnce(&mut T, Sink<E>) -> F + 'static,
    mut handler: impl FnMut(&mut T, E) -> A + 'static,
) -> impl Effect<C, T>
where
    C: Proxied,
    E: Send + 'static,
    F: Future<Output = ()> + Send + 'static,
    A: Into<Action>,
{
    Task {
        task,
        handler: move |data: &mut T, event| handler(data, event).into(),
        marker: PhantomData,
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
    pub fn send(&self, event: E) {
        self.proxy.event(Event::new(event, self.id))
    }
}

/// [`Effect`](crate::Effect) that spawns a `task` that send events to a `handler`.
pub struct Task<E, F, G> {
    task:    F,
    handler: G,
    marker:  PhantomData<fn(E)>,
}

impl<E, F, G> ViewMarker for Task<E, F, G> {}
impl<C, T, E, F, G, H> View<C, T> for Task<E, F, G>
where
    C: Proxied,
    E: Send + 'static,
    F: FnOnce(&mut T, Sink<E>) -> H,
    G: FnMut(&mut T, E) -> Action,
    H: Future<Output = ()> + Send + 'static,
{
    type Element = ();
    type State = (G, ViewId, Aborter);

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let id = ViewId::next();
        let proxy = cx.proxy();
        let sink = Sink {
            id,
            proxy: proxy.cloned(),
            marker: PhantomData,
        };

        let task = (self.task)(data, sink);
        let (future, handle) = Abortable::new(task);
        proxy.spawn(future);

        ((), (self.handler, id, handle))
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
    ) {
    }

    fn event(
        _element: Mut<'_, Self::Element>,
        (handler, id, _): &mut Self::State,
        _cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        if let Some(event) = event.take_targeted(*id) {
            handler(data, event)
        } else {
            Action::new()
        }
    }

    fn teardown(_element: Self::Element, (_, _, handle): Self::State, _cx: &mut C) {
        handle.abort();
    }
}
