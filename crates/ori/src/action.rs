use std::ops::{BitOr, BitOrAssign};

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
#[derive(Debug)]
pub struct Action {
    /// Whether the action requests a rebuild.
    pub rebuild: bool,

    /// Events to be triggered by this action.
    pub events: Vec<Event>,
}

impl Action {
    /// No action, does nothing.
    pub const fn none() -> Self {
        Self {
            rebuild: false,
            events: Vec::new(),
        }
    }

    /// Request a rebuild of the [`View`](crate::View) tree.
    pub const fn rebuild() -> Self {
        Self {
            rebuild: true,
            events: Vec::new(),
        }
    }

    /// Request a rebuild and emit an event.
    pub fn event(event: Event) -> Self {
        Self {
            rebuild: true,
            events: vec![event],
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

    /// Merge `self` with `other`.
    pub fn merge(&mut self, other: Self) {
        self.rebuild |= other.rebuild;
        self.events.extend(other.events);
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
