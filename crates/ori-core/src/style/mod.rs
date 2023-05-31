mod attribute;
mod cache;
mod loader;
mod parse;
mod selector;
mod stylesheet;
mod transition;

pub use attribute::*;
pub use cache::*;
use glam::Vec2;
pub use loader::*;
pub use selector::*;
pub use stylesheet::*;
pub use transition::*;

use deref_derive::{Deref, DerefMut};
use ori_reactive::Event;

use crate::{AvailableSpace, DrawContext, EventContext, LayoutContext, View};

/// Styling for a single element.
#[derive(Clone, Debug, Default)]
pub struct Style {
    /// The element name.
    pub element: Option<&'static str>,
    /// The classes to apply.
    pub classes: StyleClasses,
    /// The attributes to apply.
    pub attributes: StyleAttributes,
}

impl Style {
    /// Creates a new style with the given `element` name.
    pub const fn new(element: &'static str) -> Self {
        Self {
            element: Some(element),
            classes: StyleClasses::new(),
            attributes: StyleAttributes::new(),
        }
    }

    /// Sets the element name.
    pub fn with_element(mut self, element: &'static str) -> Self {
        self.element = Some(element);
        self
    }

    /// Adds classes to the style.
    pub fn with_class(mut self, class: &str) -> Self {
        let classes = class.split_whitespace().map(StyleClass::from);
        self.classes.extend(classes);
        self
    }

    /// Adds classes to the style.
    pub fn with_classes(
        mut self,
        classes: impl IntoIterator<Item = impl Into<StyleClass>>,
    ) -> Self {
        self.classes.extend(classes.into_iter().map(Into::into));
        self
    }

    /// Adds attributes to the style.
    pub fn with_attr(mut self, key: &str, builder: impl StyleAttributeBuilder) -> Self {
        let attr = builder.attribute(key);
        self.attributes.add(attr);
        self
    }

    /// Adds attributes to the style.
    pub fn with_attrs(mut self, attrs: impl IntoIterator<Item = StyleAttribute>) -> Self {
        self.attributes.extend(attrs);
        self
    }
}

/// A value with associated style [`StyleAttributes`].
#[derive(Clone, Debug, Default, Deref, DerefMut)]
pub struct Styled<T> {
    /// The value to be styled.
    #[deref]
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
        space: AvailableSpace,
    ) -> Vec2 {
        self.value.layout(state, cx, space)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.value.draw(state, cx)
    }
}

/// A trait for adding style attributes to a value.
pub trait Styleable<T> {
    /// Converts the `self` into a [`Styled<T>`](Styled) value.
    fn styled(self) -> Styled<T>;

    /// Adds a class.
    fn class(self, class: impl AsRef<str>) -> Styled<T>;

    /// Adds an attribute.
    fn attr(self, key: &str, builder: impl StyleAttributeBuilder) -> Styled<T>;
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
}
