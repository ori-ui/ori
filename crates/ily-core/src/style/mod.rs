mod attribute;
mod loader;
mod parse;
mod selector;
mod stylesheet;
mod transition;

pub use attribute::*;
pub use loader::*;
pub use selector::*;
pub use stylesheet::*;
pub use transition::*;

use deref_derive::{Deref, DerefMut};

use crate::{BoxConstraints, DrawContext, Event, EventContext, LayoutContext, View};

#[derive(Clone, Debug, Default)]
pub struct Style {
    pub element: Option<&'static str>,
    pub classes: StyleClasses,
    pub attributes: StyleAttributes,
}

impl Style {
    pub const fn new(element: &'static str) -> Self {
        Self {
            element: Some(element),
            classes: StyleClasses::new(),
            attributes: StyleAttributes::new(),
        }
    }

    pub fn with_element(mut self, name: &'static str) -> Self {
        self.element = Some(name);
        self
    }

    pub fn with_class(mut self, class: &str) -> Self {
        let classes = class.split_whitespace().map(StyleClass::from);
        self.classes.extend(classes);
        self
    }

    pub fn with_classes(
        mut self,
        classes: impl IntoIterator<Item = impl Into<StyleClass>>,
    ) -> Self {
        self.classes.extend(classes.into_iter().map(Into::into));
        self
    }

    pub fn with_attr(mut self, key: &str, builder: impl StyleAttributeBuilder) -> Self {
        let attr = builder.attribute(key);
        self.attributes.add(attr);
        self
    }

    pub fn with_attrs(mut self, attrs: impl IntoIterator<Item = StyleAttribute>) -> Self {
        self.attributes.extend(attrs);
        self
    }

    pub fn selectors(&self, mut ancestors: StyleElements) -> StyleSelectors {
        ancestors.add(StyleElement::new(
            self.element.map(Into::into),
            self.classes.iter().cloned().collect(),
        ));

        StyleSelectors {
            elements: ancestors,
            classes: self.classes.clone(),
        }
    }
}

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

    fn build(&self) -> Self::State {
        self.value.build()
    }

    fn style(&self) -> Style {
        self.value
            .style()
            .with_classes(self.classes.iter().cloned())
            .with_attrs(self.attributes.iter().cloned())
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
    fn class(self, class: impl AsRef<str>) -> Styled<T>;

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

    fn class(mut self, class: impl AsRef<str>) -> Styled<T> {
        let classes = class.as_ref().split_whitespace().map(StyleClass::from);
        self.classes.extend(classes);
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

    fn class(self, class: impl AsRef<str>) -> Styled<T> {
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
