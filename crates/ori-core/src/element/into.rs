use crate::{AnyView, Element, ElementView, View};

/// A trait for types that can be converted into an [`Element`].
pub trait IntoElement<V: ElementView = Box<dyn AnyView>> {
    /// Convert `self` into an [`Element`].
    fn into_element(self) -> Element<V>;
}

impl<T: View> IntoElement<T> for T {
    fn into_element(self) -> Element<T> {
        Element::from_view(self)
    }
}

impl<T: View> IntoElement for T {
    fn into_element(self) -> Element {
        Element::from_view(Box::new(self))
    }
}

impl<T: ElementView> IntoElement<T> for Element<T> {
    fn into_element(self) -> Element<T> {
        self
    }
}
