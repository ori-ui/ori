use crate::{AnyPod, AnyView, Pod};

/// Create a new flexible child.
pub fn flex<T>(flex: f32, content: impl AnyView<T> + 'static) -> Flex<T> {
    Flex::new(flex, content)
}

pub struct Flex<T> {
    pub content: AnyPod<T>,
    pub flex: f32,
}

impl<T> Flex<T> {
    pub fn new(flex: f32, content: impl AnyView<T> + 'static) -> Self {
        Self {
            content: Pod::new(Box::new(content)),
            flex,
        }
    }
}
