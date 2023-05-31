use std::fmt::Display;

use smallvec::SmallVec;
use smol_str::SmolStr;

/// A style class, (e.g. `.foo`).
pub type StyleClass = SmolStr;

/// A set of style classes, (e.g. `.foo.bar`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleClasses {
    classes: SmallVec<[StyleClass; 2]>,
}

impl StyleClasses {
    /// Creates an empty [`StyleClasses`].
    pub const fn new() -> Self {
        Self {
            classes: SmallVec::new_const(),
        }
    }

    /// Returns the number of classes.
    pub fn len(&self) -> usize {
        self.classes.len()
    }

    /// Returns true if there are no classes.
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    /// Clears the classes.
    pub fn clear(&mut self) {
        self.classes.clear();
    }

    /// Pushes a class to the end of the list.
    pub fn push(&mut self, class: impl Into<StyleClass>) {
        self.classes.push(class.into());
    }

    /// Extends the list with the given classes.
    pub fn extend(&mut self, classes: impl IntoIterator<Item = impl Into<StyleClass>>) {
        self.classes.extend(classes.into_iter().map(Into::into));
    }

    /// Returns an iterator over the classes.
    pub fn iter(&self) -> impl Iterator<Item = &StyleClass> {
        self.classes.iter()
    }

    /// Checks if the selector matches the given classes.
    pub fn matches(&self, other: &Self) -> bool {
        for class in self.iter() {
            if !other.classes.contains(class) {
                return false;
            }
        }

        true
    }
}

impl IntoIterator for StyleClasses {
    type Item = StyleClass;
    type IntoIter = smallvec::IntoIter<[StyleClass; 2]>;

    fn into_iter(self) -> Self::IntoIter {
        self.classes.into_iter()
    }
}

impl<T: Into<StyleClass>> FromIterator<T> for StyleClasses {
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
