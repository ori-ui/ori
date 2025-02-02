use std::{any::Any, mem};

/// A context for a view.
#[derive(Debug, Default)]
pub struct Contexts {
    contexts: Vec<Box<dyn Any>>,
}

impl Contexts {
    /// Create a new context.
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    fn index_of<T: Any>(&self) -> Option<usize> {
        self.contexts
            .iter()
            .enumerate()
            .find(|(_, c)| c.as_ref().is::<T>())
            .map(|(i, _)| i)
    }

    /// Get the number of contexts.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    /// Check if there are no contexts.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }

    /// Check if the context is present.
    #[inline(always)]
    pub fn contains<T: Any>(&self) -> bool {
        self.index_of::<T>().is_some()
    }

    /// Push a context.
    #[inline(always)]
    pub fn insert<T: Any>(&mut self, mut context: T) -> Option<T> {
        if let Some(index) = self.get_mut::<T>() {
            mem::swap(index, &mut context);
            return Some(context);
        }

        self.contexts.push(Box::new(context));

        None
    }

    /// Pop a context.
    #[inline(always)]
    pub fn remove<T: Any>(&mut self) -> Option<T> {
        let index = self.index_of::<T>()?;

        let context = self.contexts.remove(index);
        Some(*context.downcast::<T>().expect("downcast failed"))
    }

    /// Get a context.
    #[inline(always)]
    pub fn get<T: Any>(&self) -> Option<&T> {
        let index = self.index_of::<T>()?;
        self.contexts[index].downcast_ref::<T>()
    }

    /// Get a mutable context.
    #[inline(always)]
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let index = self.index_of::<T>()?;
        self.contexts[index].downcast_mut::<T>()
    }

    /// Get a context or insert a `default`.
    #[inline(always)]
    pub fn get_or_default<T: Any + Default>(&mut self) -> &mut T {
        if !self.contains::<T>() {
            self.insert(T::default());
        }

        self.get_mut::<T>().unwrap()
    }
}
