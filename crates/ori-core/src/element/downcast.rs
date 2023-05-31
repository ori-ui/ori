use crate::{AnyView, ElementView, View};

#[derive(Clone, Copy, Debug, Default)]
pub struct ElementDowncastError;

/// A trait for downcasting a [`Element`] to a specific [`ElementView`].
pub trait DowncastElement<T: ElementView> {
    /// Downcast the [`Element`] to `&T`.
    fn downcast_ref(&self) -> Option<&T>;
    /// Downcast the [`Element`] to `&mut T`.
    fn downcast_mut(&mut self) -> Option<&mut T>;
}

impl<T: View> DowncastElement<T> for T {
    fn downcast_ref(&self) -> Option<&T> {
        Some(self)
    }

    fn downcast_mut(&mut self) -> Option<&mut T> {
        Some(self)
    }
}

impl<T: View> DowncastElement<T> for Box<dyn AnyView> {
    fn downcast_ref(&self) -> Option<&T> {
        self.as_ref().downcast_ref()
    }

    fn downcast_mut(&mut self) -> Option<&mut T> {
        self.as_mut().downcast_mut()
    }
}

impl DowncastElement<Box<dyn AnyView>> for Box<dyn AnyView> {
    fn downcast_ref(&self) -> Option<&Box<dyn AnyView>> {
        Some(self)
    }

    fn downcast_mut(&mut self) -> Option<&mut Box<dyn AnyView>> {
        Some(self)
    }
}
