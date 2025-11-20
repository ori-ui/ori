use std::{any::Any, pin::Pin, sync::Arc};

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

/// A context for keeping track of user contexts.
pub trait ProviderContext {
    /// Push a context to the stack.
    fn push_context<T: Any>(&mut self, context: Box<T>);

    /// Pop the last context from the stack.
    fn pop_context<T: Any>(&mut self) -> Option<Box<T>>;

    /// Get the previously inserted context of type `T`.
    fn get_context<T: Any>(&mut self) -> Option<&T>;

    /// Get a mutable reference to the previously inserted context of type `T`.
    fn get_context_mut<T: Any>(&mut self) -> Option<&mut T>;
}

/// A proxy for [`Action`]s.
pub trait Proxy: Send + Sync + 'static {
    /// Request a rebuild of the [`View`](crate::View) tree.
    fn rebuild(&self);

    /// Send an [`Event`] to the [`View`](crate::View) tree.
    fn event(&self, event: Event);

    /// Spawn a boxed future.
    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);

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

impl Proxy for Box<dyn Proxy> {
    fn rebuild(&self) {
        self.as_ref().rebuild();
    }

    fn event(&self, event: Event) {
        self.as_ref().event(event);
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.as_ref().spawn_boxed(future);
    }
}

impl Proxy for Arc<dyn Proxy> {
    fn rebuild(&self) {
        self.as_ref().rebuild();
    }

    fn event(&self, event: Event) {
        self.as_ref().event(event);
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.as_ref().spawn_boxed(future);
    }
}
