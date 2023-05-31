use crate::{StyleAttribute, StyleAttributeBuilder, StyleAttributes, StyleClass, StyleClasses};

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
        self.attributes.push(attr);
        self
    }

    /// Adds attributes to the style.
    pub fn with_attrs(mut self, attrs: impl IntoIterator<Item = StyleAttribute>) -> Self {
        self.attributes.extend(attrs);
        self
    }

    /// Gets the attribute with the given `key`.
    pub fn get_attribute(&self, key: &str) -> Option<&StyleAttribute> {
        self.attributes.get(key)
    }
}
