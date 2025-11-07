use crate::{Action, Event};

/// A context for a [`View`](crate::View).
pub trait AsyncContext {
    /// [`Proxy`] associated
    type Proxy: Proxy;

    /// Create a [`Self::Proxy`].
    fn proxy(&mut self) -> Self::Proxy;
}

/// A proxy for [`Action`]s.
pub trait Proxy: Clone + Send + Sync + 'static {
    /// Request a rebuild of the [`View`](crate::View) tree.
    fn rebuild(&self);

    /// Send an [`Event`] to the [`View`](crate::View) tree.
    fn event(&self, event: Event);

    /// Spawn a future.
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);

    /// Handle [`Action`], using [`Self::rebuild`], [`Self::event`], and [`Self::spawn`].
    fn action(&self, action: Action) {
        if action.rebuild {
            self.rebuild();
        }

        for event in action.events {
            self.event(event);
        }

        for future in action.futures {
            let proxy = self.clone();
            self.spawn(async move {
                proxy.action(future.await);
            });
        }
    }
}
