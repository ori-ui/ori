use std::any::Any;

/// A context for keeping track of user contexts.
pub trait Provider {
    /// Push a `resource` to the stack.
    fn push<T: Any>(&mut self, resource: Box<T>);

    /// Pop the last `resource` from the stack.
    fn pop<T: Any>(&mut self) -> Option<Box<T>>;

    /// Get the latest inserted `resouce` of type `T`.
    fn get<T: Any>(&self) -> Option<&T>;

    /// Get a mutable reference to the latest inserted `resource` of type `T`.
    fn get_mut<T: Any>(&mut self) -> Option<&mut T>;

    /// [`Self::get`] a resource or the [`Default::default`].
    fn get_or_default<T>(&self) -> T
    where
        T: Any + Clone + Default,
    {
        self.get().cloned().unwrap_or_default()
    }
}
