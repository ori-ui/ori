mod class;
mod element;
mod spec;
mod tag;
mod tree;

pub use class::*;
pub use element::*;
pub use spec::*;
pub use tag::*;
pub use tree::*;

use std::{fmt::Display, sync::Arc};

/// The kind of a combinator of a style selector.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StyleSelectorCombinator {
    /// The child combinator, (e.g. `div > foo`).
    Child,
    /// The descendant combinator, (e.g. `div foo`).
    Descendant,
}

impl Display for StyleSelectorCombinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleSelectorCombinator::Child => write!(f, " > "),
            StyleSelectorCombinator::Descendant => write!(f, " "),
        }
    }
}

/// A combinator of a style selector, in the selector `div > foo`, `div >` is the ancestor.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleSelector {
    /// The combinators of the selectors.
    pub combinators: Arc<[(StyleElementSelector, StyleSelectorCombinator)]>,
    /// The top level element selector.
    pub element: StyleElementSelector,
}

impl StyleSelector {
    /// Returns the specificity of the selector, see [`StyleSpec`] for more information.
    pub fn spec(&self) -> StyleSpec {
        let mut spec = self.element.spec();

        for (selector, _) in &*self.combinators {
            spec += selector.spec();
        }

        spec
    }

    /// Checks if the selector matches an element in the tree.
    pub fn matches(&self, tree: &StyleTree) -> bool {
        // check if the element matches
        if !self.element.matches(&tree.element) {
            return false;
        }

        let mut ancestors = tree.ancestors.as_slice();

        // check if the combinators match
        for (selector, combinator) in self.combinators.iter() {
            match combinator {
                // the child combinator checks if the parent matches
                StyleSelectorCombinator::Child => {
                    // if there are no ancestors left, the selector doesn't match
                    if ancestors.is_empty() {
                        return false;
                    }

                    let parent = ancestors.last().unwrap();

                    if !selector.matches(parent) {
                        return false;
                    }

                    ancestors = &ancestors[..ancestors.len() - 1];
                }
                // the descendant combinator checks if any of the ancestors matches
                StyleSelectorCombinator::Descendant => {
                    // if there are no ancestors left, the selector doesn't match
                    if ancestors.is_empty() {
                        return false;
                    }

                    let mut matched = false;

                    while !ancestors.is_empty() {
                        let parent = ancestors.last().unwrap();
                        ancestors = &ancestors[..ancestors.len() - 1];

                        if selector.matches(parent) {
                            matched = true;
                            break;
                        }
                    }

                    if !matched {
                        return false;
                    }
                }
            }
        }

        true
    }
}

impl Display for StyleSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut combinators = String::new();

        for (selector, combinator) in &*self.combinators {
            combinators.push_str(&format!("{}{}", selector, combinator));
        }

        write!(f, "{}{}", combinators, self.element)
    }
}
