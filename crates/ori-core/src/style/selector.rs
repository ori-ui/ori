use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, AddAssign},
};

use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::StyleSelectorsHash;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleSpecificity {
    pub class: u16,
    pub tag: u16,
}

impl StyleSpecificity {
    pub const MAX: Self = Self::new(u16::MAX, u16::MAX);
    pub const INLINE: Self = Self::MAX;

    pub const fn new(class: u16, tag: u16) -> Self {
        Self { class, tag }
    }
}

impl PartialOrd for StyleSpecificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.class.partial_cmp(&other.class) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.tag.partial_cmp(&other.tag)
    }
}

impl Ord for StyleSpecificity {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.class.cmp(&other.class) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.tag.cmp(&other.tag)
    }
}

impl Add for StyleSpecificity {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            class: self.class + other.class,
            tag: self.tag + other.tag,
        }
    }
}

impl AddAssign for StyleSpecificity {
    fn add_assign(&mut self, other: Self) {
        self.class += other.class;
        self.tag += other.tag;
    }
}

/// A [`Style`](super::Style) selector.
///
/// A selector is a list of classes and an optional element.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct StyleSelectors {
    /// The element name.
    pub selectors: SmallVec<[StyleSelector; 1]>,
}

impl StyleSelectors {
    pub const fn new() -> Self {
        Self {
            selectors: SmallVec::new_const(),
        }
    }

    pub fn len(&self) -> usize {
        self.selectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selectors.is_empty()
    }

    pub fn push(&mut self, selector: StyleSelector) {
        self.selectors.push(selector);
    }

    pub fn with(mut self, selector: StyleSelector) -> Self {
        self.push(selector);
        self
    }

    pub fn hash(&self) -> StyleSelectorsHash {
        StyleSelectorsHash::new(self)
    }

    pub fn specificity(&self) -> StyleSpecificity {
        let mut specificity = StyleSpecificity::default();

        for selector in self.selectors.iter() {
            specificity += selector.specificity();
        }

        specificity
    }

    /// Returns true if `other` is a subset of `self`.
    pub fn select(&self, other: &Self) -> bool {
        if other.len() > self.len() {
            return false;
        }

        for (a, b) in self.iter().rev().zip(other.iter().rev()) {
            if !a.select(b) {
                return false;
            }
        }

        true
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &StyleSelector> {
        self.selectors.iter()
    }
}

impl IntoIterator for StyleSelectors {
    type Item = StyleSelector;
    type IntoIter = smallvec::IntoIter<[Self::Item; 1]>;

    fn into_iter(self) -> Self::IntoIter {
        self.selectors.into_iter()
    }
}

impl<'a> IntoIterator for &'a StyleSelectors {
    type Item = &'a StyleSelector;
    type IntoIter = std::slice::Iter<'a, StyleSelector>;

    fn into_iter(self) -> Self::IntoIter {
        self.selectors.iter()
    }
}

impl FromIterator<StyleSelector> for StyleSelectors {
    fn from_iter<T: IntoIterator<Item = StyleSelector>>(iter: T) -> Self {
        Self {
            selectors: iter.into_iter().collect(),
        }
    }
}

impl Display for StyleSelectors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, selector) in self.selectors.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", selector)?;
        }

        Ok(())
    }
}

pub type StyleElement = SmolStr;
pub type StyleClass = SmolStr;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct StyleSelector {
    pub element: Option<StyleElement>,
    pub classes: StyleClasses,
    pub states: StyleStates,
}

impl StyleSelector {
    pub fn new(element: Option<StyleElement>, classes: StyleClasses, states: StyleStates) -> Self {
        Self {
            element,
            classes,
            states,
        }
    }

    pub fn specificity(&self) -> StyleSpecificity {
        StyleSpecificity {
            class: self.classes.len() as u16 + self.states.len() as u16,
            tag: self.element.is_some() as u16,
        }
    }

    pub fn select(&self, other: &Self) -> bool {
        if other.element.is_some() && self.element != other.element {
            return false;
        }

        self.classes.select(&other.classes) && self.states.select(&other.states)
    }
}

impl Display for StyleSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(element) = &self.element {
            write!(f, "{}", element)?;
        } else {
            write!(f, "*")?;
        }

        write!(f, "{}", self.classes)?;
        write!(f, "{}", self.states)?;

        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct StyleClasses {
    classes: SmallVec<[StyleClass; 4]>,
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

    pub fn extend(&mut self, classes: impl IntoIterator<Item = impl Into<StyleClass>>) {
        self.classes.extend(classes.into_iter().map(Into::into));
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

impl Display for StyleClasses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for class in self.classes.iter() {
            write!(f, ".{}", class)?;
        }

        Ok(())
    }
}

/// A list of style states.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
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

    /// Returns true if the list is empSharedty.
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

impl Display for StyleStates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in self.elements.iter() {
            write!(f, ":{}", element)?;
        }

        Ok(())
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
}
