use crate::Action;

/// A context for a [`View`](crate::View).
pub trait Context {
    /// Queue an [`Action`].
    fn action(&mut self, action: Action);
}
