use std::fmt::Display;

use smallvec::SmallVec;
use smol_str::SmolStr;

/// A style tag, (e.g. `:hover`, `:active`, etc.)
pub type StyleTag = SmolStr;

/// A list of style tags, (e.g. `:hover`, `:active`, etc.)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleTags {
    tags: SmallVec<[StyleTag; 3]>,
}

impl StyleTags {
    /// Creates a new empty list.
    pub const fn new() -> Self {
        Self {
            tags: SmallVec::new_const(),
        }
    }

    /// Returns the number of tags in the list.
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Adds a tag to the list.
    pub fn push(&mut self, element: impl Into<StyleTag>) {
        self.tags.push(element.into());
    }

    /// Extends the list with the given tags.
    pub fn extend(&mut self, elements: impl IntoIterator<Item = impl Into<StyleTag>>) {
        let iter = elements.into_iter().map(|element| element.into());
        self.tags.extend(iter);
    }

    /// Returns an iterator over the tags.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.tags.iter().map(|element| element.as_str())
    }

    /// Returns true if `tag` is in the list.
    pub fn contains(&self, tag: impl AsRef<str>) -> bool {
        self.tags.iter().any(|e| e == tag.as_ref())
    }

    /// Checks if the selector matches the given tags.
    pub fn matches(&self, other: &Self) -> bool {
        for tag in self.tags.iter() {
            if !other.contains(tag) {
                return false;
            }
        }

        true
    }
}

impl IntoIterator for StyleTags {
    type Item = StyleTag;
    type IntoIter = smallvec::IntoIter<[Self::Item; 3]>;

    fn into_iter(self) -> Self::IntoIter {
        self.tags.into_iter()
    }
}

impl<T: Into<StyleTag>> FromIterator<T> for StyleTags {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            tags: iter.into_iter().map(|e| e.into()).collect(),
        }
    }
}

impl Display for StyleTags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in self.tags.iter() {
            write!(f, ":{}", element)?;
        }

        Ok(())
    }
}
