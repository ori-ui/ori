use std::fmt::Display;

use smallvec::SmallVec;
use smol_str::SmolStr;

/// A [`Style`] selector.
///
/// A selector is a list of classes and an optional element.
#[derive(Clone, Debug, Default)]
pub struct StyleSelectors {
    /// The element name.
    pub elements: StyleElements,
    /// The list of classes.
    pub classes: StyleClasses,
}

impl StyleSelectors {
    /// Creates a new selector.
    pub fn new(elements: StyleElements, classes: StyleClasses) -> Self {
        Self { elements, classes }
    }

    /// Returns true if `other` is a subset of `self`.
    pub fn select(&self, other: &Self) -> bool {
        self.elements.select(&other.elements) && self.classes.select(&other.classes)
    }
}

impl Display for StyleSelectors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, element) in self.elements.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", element)?;
            } else {
                write!(f, " {}", element)?;
            }
        }

        for (i, class) in self.classes.iter().enumerate() {
            if i == 0 && self.elements.is_empty() {
                write!(f, ".{}", class)?;
            } else {
                write!(f, " .{}", class)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct StyleElements {
    elements: SmallVec<[StyleElement; 4]>,
}

impl StyleElements {
    pub const fn new() -> Self {
        Self {
            elements: SmallVec::new_const(),
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn push(&mut self, element: StyleElement) {
        self.elements.push(element);
    }

    pub fn iter(&self) -> impl Iterator<Item = &StyleElement> {
        self.elements.iter()
    }

    /// Returns true if `other` is a subset of `self`.
    ///
    /// The order of the elements is important.
    pub fn select(&self, other: &Self) -> bool {
        let mut elements = self.elements.as_slice();

        for element in other.elements.iter() {
            if let Some(index) = elements.iter().position(|e| e.select(element)) {
                elements = &elements[index + 1..];
            } else {
                return false;
            }
        }

        true
    }
}

impl IntoIterator for StyleElements {
    type Item = StyleElement;
    type IntoIter = smallvec::IntoIter<[StyleElement; 4]>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<T: Into<StyleElement>> FromIterator<T> for StyleElements {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            elements: iter.into_iter().map(Into::into).collect(),
        }
    }
}

pub type StyleClass = SmolStr;

#[derive(Clone, Debug, Default)]
pub struct StyleClasses {
    classes: SmallVec<[SmolStr; 4]>,
}

impl StyleClasses {
    pub const fn new() -> Self {
        Self {
            classes: SmallVec::new_const(),
        }
    }

    pub fn len(&self) -> usize {
        self.classes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    pub fn push(&mut self, class: impl Into<SmolStr>) {
        self.classes.push(class.into());
    }

    pub fn iter(&self) -> impl Iterator<Item = &SmolStr> {
        self.classes.iter()
    }

    /// Returns true if `other` is a subset of `self`.
    pub fn select(&self, other: &Self) -> bool {
        for class in other.classes.iter() {
            if !self.classes.contains(class) {
                return false;
            }
        }

        true
    }
}

impl IntoIterator for StyleClasses {
    type Item = SmolStr;
    type IntoIter = smallvec::IntoIter<[SmolStr; 4]>;

    fn into_iter(self) -> Self::IntoIter {
        self.classes.into_iter()
    }
}

impl<T: Into<SmolStr>> FromIterator<T> for StyleClasses {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            classes: iter.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StyleElement {
    pub name: Option<SmolStr>,
    pub states: StyleStates,
}

impl StyleElement {
    pub fn new(name: impl Into<Option<SmolStr>>, states: StyleStates) -> Self {
        Self {
            name: name.into(),
            states,
        }
    }

    pub fn push_state(&mut self, state: impl Into<SmolStr>) {
        self.states.push(state);
    }

    pub fn select(&self, other: &Self) -> bool {
        let name_matches = other.name.is_none() || self.name == other.name;
        name_matches && self.states.select(&other.states)
    }
}

impl Display for StyleElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}", name)?;
        }

        for state in self.states.iter() {
            write!(f, ":{}", state)?;
        }

        Ok(())
    }
}

impl From<SmolStr> for StyleElement {
    fn from(name: SmolStr) -> Self {
        Self::new(name, StyleStates::new())
    }
}

impl From<&str> for StyleElement {
    fn from(name: &str) -> Self {
        Self::new(Some(name.into()), StyleStates::new())
    }
}

/// A list of style states.
#[derive(Clone, Debug, Default)]
pub struct StyleStates {
    elements: SmallVec<[SmolStr; 4]>,
}

impl StyleStates {
    /// Creates a new empty list.
    pub const fn new() -> Self {
        Self {
            elements: SmallVec::new_const(),
        }
    }

    /// Returns the number of states in the list.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Adds a state to the list.
    pub fn push(&mut self, element: impl Into<SmolStr>) {
        self.elements.push(element.into());
    }

    /// Extends the list with the given states.
    pub fn extend(&mut self, elements: impl IntoIterator<Item = impl Into<SmolStr>>) {
        let iter = elements.into_iter().map(|element| element.into());
        self.elements.extend(iter);
    }

    /// Returns an iterator over the states.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.elements.iter().map(|element| element.as_str())
    }

    /// Returns true if `element` is in the list.
    pub fn contains(&self, element: impl AsRef<str>) -> bool {
        self.elements.iter().any(|e| e == element.as_ref())
    }

    /// Returns true if `other` is a subset of `self`.
    pub fn select(&self, other: &Self) -> bool {
        for element in other.elements.iter() {
            if !self.contains(element) {
                return false;
            }
        }

        true
    }
}

impl IntoIterator for StyleStates {
    type Item = SmolStr;
    type IntoIter = smallvec::IntoIter<[Self::Item; 4]>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<T: Into<SmolStr>> FromIterator<T> for StyleStates {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            elements: iter.into_iter().map(|e| e.into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn selector_select() {
        let selector = StyleSelectors::from_str("a .b .c").unwrap();
        let other = StyleSelectors::from_str(".b .c").unwrap();

        assert!(selector.select(&other));
    }

    #[test]
    fn selector_select_not() {
        let selector = StyleSelectors::from_str("a .b .c").unwrap();
        let other = StyleSelectors::from_str(".b .c .d").unwrap();

        assert!(!selector.select(&other));
    }

    #[test]
    fn classes_select() {
        let classes =
            StyleClasses::from_iter(vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
        let other = StyleClasses::from_iter(vec!["b", "d", "f", "h", "j"]);

        assert!(classes.select(&other));
    }

    #[test]
    fn classes_select_not() {
        let classes =
            StyleClasses::from_iter(vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
        let other = StyleClasses::from_iter(vec!["b", "d", "f", "h", "j", "k"]);

        assert!(!classes.select(&other));
    }

    #[test]
    fn elements_select() {
        let elements =
            StyleElements::from_iter(vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
        let other = StyleElements::from_iter(vec!["b", "d", "f", "h", "j"]);

        assert!(elements.select(&other));
    }

    #[test]
    fn elements_select_not() {
        let elements =
            StyleElements::from_iter(vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]);
        let other = StyleElements::from_iter(vec!["b", "d", "f", "h", "j", "k"]);

        assert!(!elements.select(&other));
    }
}
