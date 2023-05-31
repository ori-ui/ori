use std::ops::{Deref, DerefMut};

use crate::{StyleAttributeBuilder, StyleAttributes, StyleClass, StyleClasses};

/// A value with associated style [`StyleAttributes`].
#[derive(Clone, Debug, Default)]
pub struct Styled<T> {
    /// The value to be styled.
    pub value: T,
    /// The classes to apply.
    pub classes: StyleClasses,
    /// The attributes to apply.
    pub attributes: StyleAttributes,
}

impl<T> Styled<T> {
    /// Creates a new styled value with no attributes.
    pub const fn new(value: T) -> Self {
        Self {
            value,
            classes: StyleClasses::new(),
            attributes: StyleAttributes::new(),
        }
    }

    /// Sets the classes to apply.
    pub fn set_class(&mut self, class: impl AsRef<str>) {
        self.classes.clear();

        let classes = class.as_ref().split_whitespace().map(StyleClass::from);
        self.classes.extend(classes);
    }

    /// Adds attributes to the style.
    pub fn set_attr(&mut self, key: &str, builder: impl StyleAttributeBuilder) {
        self.attributes.set(builder.attribute(key));
    }
}

impl<T> Deref for Styled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Styled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// A trait for adding style attributes to a value.
pub trait Stylable<T> {
    /// Converts the `self` into a [`Styled<T>`](Styled) value.
    fn styled(self) -> Styled<T>;

    /// Adds a class.
    fn class(self, class: impl AsRef<str>) -> Styled<T>;

    /// Adds an attribute.
    fn attr(self, key: &str, builder: impl StyleAttributeBuilder) -> Styled<T>;
}

impl<T> Stylable<T> for Styled<T> {
    fn styled(self) -> Styled<T> {
        self
    }

    fn class(mut self, class: impl AsRef<str>) -> Styled<T> {
        let classes = class.as_ref().split_whitespace().map(StyleClass::from);
        self.classes.extend(classes);
        self
    }

    fn attr(mut self, key: &str, builder: impl StyleAttributeBuilder) -> Styled<T> {
        self.attributes.push(builder.attribute(key));
        self
    }
}

impl<T> Stylable<T> for T {
    fn styled(self) -> Styled<T> {
        Styled::new(self)
    }

    fn class(self, class: impl AsRef<str>) -> Styled<T> {
        Styled::new(self).class(class)
    }

    fn attr(self, key: &str, value: impl StyleAttributeBuilder) -> Styled<T> {
        Styled::new(self).attr(key, value)
    }
}
