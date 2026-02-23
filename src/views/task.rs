use std::{marker::PhantomData, sync::Arc};

use crate::{
    Action, Effect, Message, Mut, Proxied, Proxy, View, ViewId, ViewMarker,
    future::{Abortable, Aborter},
};

/// [`Effect`](crate::Effect) that spawns a `task` that emits messages to a `handler`.
#[must_use]
pub fn task<C, T, E, F, A>(
    task: impl FnOnce(&mut T, Sink<E>) -> F,
    mut handler: impl FnMut(&mut T, Sink<E>, E) -> A,
) -> impl Effect<C, T>
where
    C: Proxied,
    E: Send + 'static,
    F: Future<Output = ()> + Send + 'static,
    A: Into<Action>,
{
    Task {
        task,
        handler: move |data: &mut T, sink, message| handler(data, sink, message).into(),
        marker: PhantomData,
    }
}

/// Sink for sending events to the `handler` of a [`task`].
pub struct Sink<E>
where
    E: Send + 'static,
{
    id:     ViewId,
    proxy:  Arc<dyn Proxy>,
    marker: PhantomData<fn(E)>,
}

impl<E> Clone for Sink<E>
where
    E: Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            id:     self.id,
            proxy:  self.proxy.clone(),
            marker: PhantomData,
        }
    }
}

impl<E> Sink<E>
where
    E: Send + 'static,
{
    /// Send a `message` to the `handler` of the [`task`].
    pub fn send(&self, message: E) {
        self.proxy.message(Message::new(message, self.id))
    }
}

/// [`Effect`](crate::Effect) that spawns a `task` that send messages to a `handler`.
#[must_use]
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
    G: FnMut(&mut T, Sink<E>, E) -> Action,
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

    fn message(
        _element: Mut<'_, Self::Element>,
        (handler, id, _): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        if let Some(message) = message.take_targeted(*id) {
            let sink = Sink {
                id:     *id,
                proxy:  cx.proxy().cloned(),
                marker: PhantomData,
            };

            handler(data, sink, message)
        } else {
            Action::new()
        }
    }

    fn teardown(_element: Self::Element, (_, _, handle): Self::State, _cx: &mut C) {
        handle.abort();
    }
}
