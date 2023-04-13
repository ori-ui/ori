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

use crate::{BoxConstraints, DrawContext, Event, EventContext, LayoutContext, View, ViewState};

/// A value with associated style [`Attributes`].
#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Styled<T> {
    #[deref]
    pub value: T,
    pub classes: StyleClasses,
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
}

impl<T: View> View for Styled<T> {
    type State = T::State;

    fn build(&self) -> ViewState<Self::State> {
        let mut state = self.value.build();
        state.classes.extend(self.classes.clone());
        state.attributes.extend(self.attributes.clone());
        state
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.value.event(state, cx, event)
    }

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        bc: BoxConstraints,
    ) -> glam::Vec2 {
        self.value.layout(state, cx, bc)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.value.draw(state, cx)
    }
}

/// A trait for adding style attributes to a value.
pub trait Styleable<T> {
    /// Converts the `self` into a [`Styled<Self>`](Styled) value.
    fn styled(self) -> Styled<T>;

    /// Adds a class.
    fn class(self, class: impl Into<StyleClass>) -> Styled<T>;

    /// Adds an attribute.
    fn attr(self, key: &str, builder: impl StyleAttributeBuilder) -> Styled<T>;

    /// Adds an attribute with a transition.
    fn attr_trans(
        self,
        key: &str,
        value: impl Into<StyleAttributeValue>,
        transition: impl Into<StyleTransition>,
    ) -> Styled<T>;
}

impl<T> Styleable<T> for Styled<T> {
    fn styled(self) -> Styled<T> {
        self
    }

    fn class(mut self, class: impl Into<StyleClass>) -> Styled<T> {
        self.classes.add(class.into());
        self
    }

    fn attr(mut self, key: &str, builder: impl StyleAttributeBuilder) -> Styled<T> {
        self.attributes.add(builder.attribute(key));
        self
    }

    fn attr_trans(
        mut self,
        key: &str,
        value: impl Into<StyleAttributeValue>,
        transition: impl Into<StyleTransition>,
    ) -> Styled<T> {
        let attr = StyleAttribute::with_transition(key, value, transition);
        self.attributes.add(attr);
        self
    }
}

impl<T> Styleable<T> for T {
    fn styled(self) -> Styled<T> {
        Styled::new(self)
    }

    fn class(self, class: impl Into<StyleClass>) -> Styled<T> {
        Styled::new(self).class(class)
    }

    fn attr(self, key: &str, value: impl StyleAttributeBuilder) -> Styled<T> {
        Styled::new(self).attr(key, value)
    }

    fn attr_trans(
        self,
        key: &str,
        value: impl Into<StyleAttributeValue>,
        transition: impl Into<StyleTransition>,
    ) -> Styled<T> {
        Styled::new(self).attr_trans(key, value, transition)
    }
}
