use std::{
    fmt,
    ops::{BitOr, BitOrAssign},
    pin::Pin,
    sync::Arc,
};

use crate::{Event, Proxy};

/// [`Future`] that to be run by an [`Action`].
pub type ActionFuture = Pin<Box<dyn Future<Output = Action> + Send>>;

/// Callback to be run by an [`Action`].
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
        Self::new().with_event(event)
    }

    /// Spawn a future that emits an action.
    pub fn spawn(fut: impl Future<Output: Into<Action>> + Send + 'static) -> Self {
        Self::new().with_spawn(fut)
    }

    /// Add a task that has access to a proxy.
    pub fn task<F>(task: impl FnOnce(Arc<dyn Proxy>) -> F + 'static) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self::new().with_task(task)
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
    pub fn add_spawn(&mut self, fut: impl Future<Output: Into<Action>> + Send + 'static) {
        self.futures.push(Box::pin(async { fut.await.into() }));
    }

    /// Add a task that has access to a proxy.
    pub fn add_task<F>(&mut self, task: impl FnOnce(Arc<dyn Proxy>) -> F + 'static)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.callbacks.push(Box::new(|proxy| {
            let proxy = proxy.cloned();
            let future = task(proxy.clone());
            proxy.spawn(future);
        }))
    }

    /// Set whether a rebuild is requested.
    pub fn with_rebuild(mut self, rebuild: bool) -> Self {
        self.set_rebuild(rebuild);
        self
    }

    /// Add an event to the action.
    pub fn with_event(mut self, event: Event) -> Self {
        self.add_event(event);
        self
    }

    /// Add a future that emits an action.
    pub fn with_spawn(mut self, fut: impl Future<Output: Into<Action>> + Send + 'static) -> Self {
        self.add_spawn(fut);
        self
    }

    /// Add a task that has access to a proxy.
    pub fn with_task<F>(mut self, task: impl FnOnce(Arc<dyn Proxy>) -> F + 'static) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.add_task(task);
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

impl From<()> for Action {
    fn from((): ()) -> Self {
        Action::rebuild()
    }
}

impl From<Event> for Action {
    fn from(event: Event) -> Self {
        Action::event(event)
    }
}
