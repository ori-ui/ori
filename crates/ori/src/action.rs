use std::{
    fmt,
    ops::{BitOr, BitOrAssign},
    pin::Pin,
};

use crate::Event;

/// Action to be taken as a result of [`View::event`].
///
/// Callbacks from [`View`]s will usually return one of these, note that `()` implements
/// [`IntoAction`], which means that if no action is explicitly return by a callback, the default
/// action is [`Action::rebuild`]. If this behaviour is not desired, callbacks should explicitly
/// return [`Action::none`].
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
    pub futures: Vec<Pin<Box<dyn Future<Output = Action> + Send>>>,
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
            rebuild: false,
            events: Vec::new(),
            futures: Vec::new(),
        }
    }

    /// Request a rebuild of the [`View`](crate::View) tree.
    pub const fn rebuild() -> Self {
        Self {
            rebuild: true,
            events: Vec::new(),
            futures: Vec::new(),
        }
    }

    /// Request a rebuild and emit an event.
    pub fn event(event: Event) -> Self {
        Self {
            rebuild: true,
            events: vec![event],
            futures: Vec::new(),
        }
    }

    /// Spawn a future that emits an action.
    pub fn spawn(
        fut: impl Future<Output: IntoAction> + Send + 'static,
    ) -> Self {
        let fut = Box::pin(async { fut.await.into_action() });

        Self {
            rebuild: false,
            events: Vec::new(),
            futures: vec![fut],
        }
    }

    /// Set whether the action requests a rebuild.
    pub fn with_rebuild(mut self, rebuild: bool) -> Self {
        self.rebuild = rebuild;
        self
    }

    /// Add an event to the action.
    pub fn with_event(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    /// Add a future that emits an action.
    pub fn with_spawn(
        mut self,
        fut: impl Future<Output: IntoAction> + Send + 'static,
    ) -> Self {
        self.futures.push(Box::pin(async {
            fut.await.into_action()
        }));

        self
    }

    /// Merge `self` with `other`.
    pub fn merge(&mut self, mut other: Self) {
        self.rebuild |= other.rebuild;
        self.events.append(&mut other.events);
        self.futures.append(&mut other.futures);
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
pub trait IntoAction {
    /// Convert `self` in an `Action`.
    fn into_action(self) -> Action;
}

impl IntoAction for Action {
    fn into_action(self) -> Action {
        self
    }
}

impl IntoAction for () {
    fn into_action(self) -> Action {
        Action::rebuild()
    }
}

impl IntoAction for Event {
    fn into_action(self) -> Action {
        Action::event(self)
    }
}
