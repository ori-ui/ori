use std::{pin::Pin, sync::Arc};

use crate::{Action, Event};

/// A context with a common base element, that is [`Super`](crate::Super) to all elements in the
/// context.
pub trait BaseElement {
    /// The base element.
    type Element;
}

/// A context for a [`View`](crate::View).
pub trait AsyncContext {
    /// [`Proxy`] associated
    type Proxy: Proxy;

    /// Create a [`Self::Proxy`].
    fn proxy(&mut self) -> Self::Proxy;

    /// Send an action using [`Self::Proxy`].
    fn send_action(&mut self, action: Action) {
        let proxy: Arc<dyn Proxy> = Arc::new(self.proxy());
        proxy.action(action);
    }
}

/// A proxy for [`Action`]s.
pub trait Proxy: Send + Sync + 'static {
    /// Request a rebuild of the [`View`](crate::View) tree.
    fn rebuild(&self);

    /// Send an [`Event`] to the [`View`](crate::View) tree.
    fn event(&self, event: Event);

    /// Spawn a boxed future.
    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>);

    /// Spawn a future.
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static)
    where
        Self: Sized,
    {
        self.spawn_boxed(Box::pin(future));
    }

    /// Send an action using [`Self::rebuild`], [`Self::event`], and [`Self::spawn`].
    fn action(&self, action: Action)
    where
        Self: Clone,
    {
        if action.rebuild {
            self.rebuild();
        }

        for event in action.events {
            self.event(event);
        }

        for future in action.futures {
            self.spawn_boxed({
                let proxy = self.clone();
                Box::pin(async move { proxy.action(future.await) })
            });
        }
    }
}

impl Proxy for Arc<dyn Proxy> {
    fn rebuild(&self) {
        self.as_ref().rebuild();
    }

    fn event(&self, event: Event) {
        self.as_ref().event(event);
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        self.as_ref().spawn_boxed(future);
    }
}
