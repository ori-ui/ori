use std::{pin::Pin, sync::Arc};

use crate::{Action, Message};

/// A context for a [`View`](crate::View).
pub trait Proxied {
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

    /// Send a [`Message`] to the [`View`](crate::View) tree.
    fn message(&self, message: Message);

    /// Spawn a boxed future.
    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>);

    /// Spawn a future.
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static)
    where
        Self: Sized,
    {
        self.spawn_boxed(Box::pin(future));
    }

    /// Send an action using [`Self::rebuild`], [`Self::message`], and [`Self::spawn`].
    fn action(&self, action: Action)
    where
        Self: Clone,
    {
        if action.rebuild {
            self.rebuild();
        }

        for message in action.messages {
            self.message(message);
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

    fn message(&self, message: Message) {
        self.as_ref().message(message);
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.as_ref().spawn_boxed(future);
    }
}
