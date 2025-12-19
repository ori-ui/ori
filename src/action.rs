use std::{
    fmt,
    marker::PhantomData,
    ops::{BitOr, BitOrAssign},
    pin::Pin,
};

use crate::{Event, Proxy};

/// [`Future`] that to be run by an [`Action`].
pub type ActionFuture = Pin<Box<dyn Future<Output = Action> + Send>>;

/// [`Callback`] to be run by an [`Action`].
pub type ActionCallback = Box<dyn FnOnce(&dyn Proxy)>;

/// Action to be taken as a result of [`View::event`].
///
/// Callbacks from [`View`]s will usually return one of these, note that `()` implements
/// [`IntoAction`], which means that if no action is explicitly return by a callback, the default
/// action is [`Action::rebuild`]. If this behaviour is not desired, callbacks should explicitly
/// return [`Action::new`].
///
/// [`View`]: crate::View
/// [`View::event`]: crate::View::event
#[must_use]
pub struct Action {
    /// Whether the action requests a rebuild.
    pub rebuild: bool,

    /// Events to be triggered by this action.
    pub events: Vec<Event>,

    /// Futures to spawned by this action.
    pub futures: Vec<ActionFuture>,

    /// Callback that may use a proxy.
    pub callbacks: Vec<ActionCallback>,
}

impl Default for Action {
    fn default() -> Self {
        Self::new()
    }
}

impl Action {
    /// New empty action, does nothing.
    pub const fn new() -> Self {
        Self {
            rebuild:   false,
            events:    Vec::new(),
            futures:   Vec::new(),
            callbacks: Vec::new(),
        }
    }

    /// Request a rebuild of the [`View`](crate::View) tree.
    pub const fn rebuild() -> Self {
        Self {
            rebuild:   true,
            events:    Vec::new(),
            futures:   Vec::new(),
            callbacks: Vec::new(),
        }
    }

    /// Request a rebuild and emit an event.
    pub fn event(event: Event) -> Self {
        Self {
            rebuild:   true,
            events:    vec![event],
            futures:   Vec::new(),
            callbacks: Vec::new(),
        }
    }

    /// Spawn a future that emits an action.
    pub fn spawn<I>(fut: impl Future<Output: IntoAction<I>> + Send + 'static) -> Self {
        let fut = Box::pin(async { fut.await.into_action() });

        Self {
            rebuild:   false,
            events:    Vec::new(),
            futures:   vec![fut],
            callbacks: Vec::new(),
        }
    }

    /// Run a callback that takes a [`Proxy`].
    pub fn proxy(callback: impl FnOnce(&dyn Proxy) + 'static) -> Self {
        Self {
            rebuild:   false,
            events:    Vec::new(),
            futures:   Vec::new(),
            callbacks: vec![Box::new(callback)],
        }
    }

    /// Set whether a rebuild is requested.
    pub fn set_rebuild(&mut self, rebuild: bool) {
        self.rebuild |= rebuild;
    }

    /// Add an event to the action.
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Add a future that emits an action.
    pub fn add_spawn<I>(&mut self, fut: impl Future<Output: IntoAction<I>> + Send + 'static) {
        self.futures.push(Box::pin(async {
            fut.await.into_action()
        }));
    }

    /// Add a callback that takes a [`Proxy`].
    pub fn add_proxy(&mut self, callback: impl FnOnce(&dyn Proxy) + 'static) {
        self.callbacks.push(Box::new(callback));
    }

    /// Set whether a rebuild is requested.
    pub fn with_rebuild(mut self, rebuild: bool) -> Self {
        self.set_rebuild(rebuild);
        self
    }

    /// Add an event to the action.
    pub fn with_event(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    /// Add a future that emits an action.
    pub fn with_spawn<I>(
        mut self,
        fut: impl Future<Output: IntoAction<I>> + Send + 'static,
    ) -> Self {
        self.futures.push(Box::pin(async {
            fut.await.into_action()
        }));

        self
    }

    /// Add a callback that takes a [`Proxy`].
    pub fn with_proxy(mut self, callback: impl FnOnce(&dyn Proxy) + 'static) -> Self {
        self.add_proxy(callback);
        self
    }

    /// Merge `self` with `other`.
    pub fn merge(&mut self, mut other: Self) {
        self.rebuild |= other.rebuild;
        self.events.append(&mut other.events);
        self.futures.append(&mut other.futures);
        self.callbacks.append(&mut other.callbacks);
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("rebuild", &self.rebuild)
            .field("events", &self.events)
            .finish()
    }
}

impl BitOr for Action {
    type Output = Action;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.merge(rhs);
        self
    }
}

impl BitOrAssign for Action {
    fn bitor_assign(&mut self, rhs: Self) {
        self.merge(rhs);
    }
}

/// Trait for types that can be converted to an [`Action`].
pub trait IntoAction<I> {
    /// Convert `self` in an `Action`.
    fn into_action(self) -> Action;
}

impl IntoAction<Action> for Action {
    fn into_action(self) -> Action {
        self
    }
}

impl IntoAction<()> for () {
    fn into_action(self) -> Action {
        Action::rebuild()
    }
}

impl IntoAction<Event> for Event {
    fn into_action(self) -> Action {
        Action::event(self)
    }
}

const _: () = {
    pub struct FutImpl<F, A, I>(PhantomData<(F, A, I)>);
    impl<F, A, I> IntoAction<FutImpl<F, A, I>> for F
    where
        F: Future<Output = A> + Send + 'static,
        A: IntoAction<I>,
    {
        fn into_action(self) -> Action {
            Action::spawn(self)
        }
    }

    pub struct FnImpl<F, A, I>(PhantomData<(F, A, I)>);
    impl<F, A, I> IntoAction<FnImpl<F, A, I>> for F
    where
        F: FnOnce(&dyn Proxy) -> A + 'static,
        A: IntoAction<I>,
    {
        fn into_action(self) -> Action {
            Action::proxy(|proxy| {
                let action = self(proxy);
                proxy.clone().action(action.into_action());
            })
        }
    }
};
