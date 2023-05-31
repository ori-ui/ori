use crate::{StyleAttribute, StyleAttributeKey, StyleAttributeValue, StyleTransition};

/// A trait for types that can be converted into a [`StyleAttribute`].
pub trait StyleAttributeBuilder {
    /// Create a new [`StyleAttribute`], with the given `key`.
    fn attribute(self, key: impl Into<StyleAttributeKey>) -> StyleAttribute;
}

impl<T: Into<StyleAttributeValue>> StyleAttributeBuilder for T {
    fn attribute(self, key: impl Into<StyleAttributeKey>) -> StyleAttribute {
        StyleAttribute::new(key.into(), self.into(), None)
    }
}

impl<T: Into<StyleAttributeValue>, U: Into<StyleTransition>> StyleAttributeBuilder for (T, U) {
    fn attribute(self, key: impl Into<StyleAttributeKey>) -> StyleAttribute {
        StyleAttribute::new(key.into(), self.0.into(), Some(self.1.into()))
    }
}

/// An ease of use function to create an [`StyleAttributeBuilder`] with a transition.
pub fn trans(
    value: impl Into<StyleAttributeValue>,
    transition: impl Into<StyleTransition>,
) -> impl StyleAttributeBuilder {
    (value, transition)
}
