use std::{
    fmt, mem,
    ops::{BitOr, BitOrAssign},
    pin::Pin,
    sync::Arc,
};

use crate::{Message, Proxy};

/// [`Future`] that to be run by an [`Action`].
pub type ActionFuture = Pin<Box<dyn Future<Output = Action> + Send>>;

/// Callback to be run by an [`Action`].
pub type ActionCallback = Box<dyn FnOnce(&dyn Proxy)>;

/// Action to be taken as a result of [`View::message`].
///
/// Callbacks from [`View`]s will usually return one of these, note that `()` implements
/// `Into<Action>`, which means that if no action is explicitly return by a callback, the default
/// action is [`Action::rebuild`]. If this behaviour is not desired, callbacks should explicitly
/// return [`Action::new`].
///
/// [`View`]: crate::View
/// [`View::message`]: crate::View::event
#[must_use]
pub struct Action {
    /// Whether the action requests a rebuild.
    pub rebuild: bool,

    /// Messages to be sent by this action.
    pub messages: Vec<Message>,

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
            messages:  Vec::new(),
            futures:   Vec::new(),
            callbacks: Vec::new(),
        }
    }

    /// Request a rebuild of the [`View`](crate::View) tree.
    pub const fn rebuild() -> Self {
        Self {
            rebuild:   true,
            messages:  Vec::new(),
            futures:   Vec::new(),
            callbacks: Vec::new(),
        }
    }

    /// Request a rebuild and emit an message.
    pub fn message(message: Message) -> Self {
        Self::new().with_message(message)
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

    /// Add an message to the action.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
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

    /// Add an message to the action.
    pub fn with_message(mut self, message: Message) -> Self {
        self.add_message(message);
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

    /// Take the `rebuild` flag, and handle it now.
    pub fn take_rebuild(&mut self) -> bool {
        mem::replace(&mut self.rebuild, false)
    }

    /// Merge `self` with `other`.
    pub fn merge(&mut self, mut other: Self) {
        self.rebuild |= other.rebuild;
        self.messages.append(&mut other.messages);
        self.futures.append(&mut other.futures);
        self.callbacks.append(&mut other.callbacks);
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("rebuild", &self.rebuild)
            .field("messages", &self.messages)
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

impl From<Message> for Action {
    fn from(message: Message) -> Self {
        Action::message(message)
    }
}
