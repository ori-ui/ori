mod attribute;
mod loader;
mod parse;
mod selector;
mod style;
mod transition;

pub use attribute::*;
pub use loader::*;
pub use selector::*;
pub use style::*;
pub use transition::*;

use deref_derive::{Deref, DerefMut};

/// A value with associated style [`Attributes`].
#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Styled<T> {
    #[deref]
    pub value: T,
    pub attributes: Attributes,
}

impl<T: Into<T>> From<T> for Styled<T> {
    fn from(value: T) -> Self {
        Self {
            value: value.into(),
            attributes: Attributes::new(),
        }
    }
}

impl<T> Styled<T> {
    /// Creates a new styled value with no attributes.
    pub const fn new(value: T) -> Self {
        Self {
            value,
            attributes: Attributes::new(),
        }
    }
}

/// A trait for adding style attributes to a value.
pub trait Styleable<T> {
    /// Converts the `self` into a [`Styled<Self>`](Styled) value.
    fn styled(self) -> Styled<T>;

    /// Adds an attribute.
    fn attr(self, key: &str, value: impl Into<AttributeValue>) -> Styled<T>;

    /// Adds an attribute with a transition.
    fn attr_trans(
        self,
        key: &str,
        value: impl Into<AttributeValue>,
        transition: impl Into<Transition>,
    ) -> Styled<T>;
}

impl<T> Styleable<T> for Styled<T> {
    fn styled(self) -> Styled<T> {
        self
    }

    fn attr(mut self, key: &str, value: impl Into<AttributeValue>) -> Styled<T> {
        self.attributes.add(Attribute::new(key, value));
        self
    }

    fn attr_trans(
        mut self,
        key: &str,
        value: impl Into<AttributeValue>,
        transition: impl Into<Transition>,
    ) -> Styled<T> {
        let attr = Attribute::with_transition(key, value, transition);
        self.attributes.add(attr);
        self
    }
}

impl<T> Styleable<T> for T {
    fn styled(self) -> Styled<T> {
        Styled::new(self)
    }

    fn attr(self, key: &str, value: impl Into<AttributeValue>) -> Styled<T> {
        Styled::new(self).attr(key, value)
    }

    fn attr_trans(
        self,
        key: &str,
        value: impl Into<AttributeValue>,
        transition: impl Into<Transition>,
    ) -> Styled<T> {
        Styled::new(self).attr_trans(key, value, transition)
    }
}
