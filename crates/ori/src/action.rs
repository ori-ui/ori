use std::ops::{BitOr, BitOrAssign};

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
#[derive(Clone, Debug)]
pub struct Action {
    /// Whether the action requests a rebuild.
    pub rebuild: bool,
}

impl Action {
    /// No action, does nothing.
    pub const fn none() -> Self {
        Self { rebuild: false }
    }

    /// Request a rebuild of the [`View`](crate::View) tree.
    pub const fn rebuild() -> Self {
        Self { rebuild: true }
    }

    /// Merge `self` with `other`.
    pub fn merge(&mut self, other: Self) {
        self.rebuild |= other.rebuild;
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

pub trait IntoAction {
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
