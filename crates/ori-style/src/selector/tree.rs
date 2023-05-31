use std::{fmt::Display, mem};

use crate::StyleElementSelector;

/// A style selector tree.
///
/// This is used to match a style selector against an element.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StyleTree {
    /// The ancestors of the selector.
    pub ancestors: Vec<StyleElementSelector>,
    /// The top level element selector.
    pub element: StyleElementSelector,
}

impl StyleTree {
    /// Creates a new [`StyleTree`].
    pub fn new(element: StyleElementSelector) -> Self {
        Self {
            ancestors: Vec::new(),
            element,
        }
    }

    /// Pushes an ancestor to the tree.
    pub fn push(&mut self, ancestor: StyleElementSelector) {
        let ancestor = mem::replace(&mut self.element, ancestor);
        self.ancestors.push(ancestor);
    }

    /// Pops an ancestor from the tree.
    pub fn pop(&mut self) -> Option<StyleElementSelector> {
        let ancestor = self.ancestors.pop()?;
        Some(mem::replace(&mut self.element, ancestor))
    }
}

impl Display for StyleTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ancestor in &self.ancestors {
            write!(f, "{} ", ancestor)?;
        }

        write!(f, "{}", self.element)
    }
}
