use std::{any::Any, pin::Pin, sync::Arc};

use crate::{Action, Event};

/// A context with a common base element, that is [`Super`](crate::Super) to all elements in the
/// context.
pub trait BaseElement {
    /// The base element.
    type Element;
}

/// A context for keeping track of user contexts.
pub trait Providable {
    /// Push a `resource` to the stack.
    fn push<T: Any>(&mut self, resource: Box<T>);

    /// Pop the last `resource` from the stack.
    fn pop<T: Any>(&mut self) -> Option<Box<T>>;

    /// Get the latest inserted `resouce` of type `T`.
    fn get<T: Any>(&self) -> Option<&T>;

    /// Get a mutable reference to the latest inserted `resource` of type `T`.
    fn get_mut<T: Any>(&mut self) -> Option<&mut T>;

    /// [`Self::get`] a resource or the [`Default::default`].
    fn get_or_default<T>(&self) -> Option<T>
    where
        T: Any + Clone + Default,
    {
        self.get().cloned().unwrap_or_default()
    }
}

/// A context for a [`View`](crate::View).
pub trait Proxyable {
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
    /// Clone `self` into an [`Arc`].
    fn cloned(&self) -> Arc<dyn Proxy>;

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
                let proxy = Clone::clone(self);
                Box::pin(async move { proxy.action(future.await) })
            });
        }

        for callback in action.callbacks {
            callback(self);
        }
    }
}

impl Proxy for Arc<dyn Proxy> {
    fn cloned(&self) -> Arc<dyn Proxy> {
        Clone::clone(self)
    }

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
