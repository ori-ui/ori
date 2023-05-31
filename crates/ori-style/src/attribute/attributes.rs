use crate::StyleAttribute;

/// A collection of attributes.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleAttributes {
    attributes: Vec<StyleAttribute>,
}

impl StyleAttributes {
    /// Create a new collection of attributes.
    pub const fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    /// Push an attribute to the collection.
    pub fn push(&mut self, attribute: StyleAttribute) {
        self.attributes.push(attribute);
    }

    /// Extend the collection with an iterator of attributes.
    pub fn extend(&mut self, iter: impl IntoIterator<Item = StyleAttribute>) {
        self.attributes.extend(iter);
    }

    /// Set an attribute in the collection.
    pub fn set(&mut self, attribute: StyleAttribute) {
        let key = attribute.key().clone();

        for attr in &mut self.attributes.iter_mut().rev() {
            if attr.key() == &key {
                *attr = attribute;
                return;
            }
        }

        self.push(attribute);
    }

    /// Get an attribute from the collection.
    pub fn get(&self, key: &str) -> Option<&StyleAttribute> {
        self.attributes.iter().rev().find(|a| a.key() == key)
    }

    /// Get the attributes in the collection.
    pub fn attributes(&self) -> &[StyleAttribute] {
        &self.attributes
    }
}

impl IntoIterator for StyleAttributes {
    type Item = StyleAttribute;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.into_iter()
    }
}

impl<'a> IntoIterator for &'a StyleAttributes {
    type Item = &'a StyleAttribute;
    type IntoIter = std::slice::Iter<'a, StyleAttribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter()
    }
}

impl<'a> IntoIterator for &'a mut StyleAttributes {
    type Item = &'a mut StyleAttribute;
    type IntoIter = std::slice::IterMut<'a, StyleAttribute>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter_mut()
    }
}
