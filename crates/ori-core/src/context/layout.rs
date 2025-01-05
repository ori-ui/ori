use std::ops::{Deref, DerefMut};

use crate::view::ViewState;

use super::BaseCx;

/// A context for laying out the view tree.
pub struct LayoutCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
}

impl<'b> Deref for LayoutCx<'_, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl DerefMut for LayoutCx<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> LayoutCx<'a, 'b> {
    /// Create a new layout context.
    pub fn new(base: &'a mut BaseCx<'b>, view_state: &'a mut ViewState) -> Self {
        Self { base, view_state }
    }

    /// Create a child context.
    pub fn child(&mut self) -> LayoutCx<'_, 'b> {
        LayoutCx {
            base: self.base,
            view_state: self.view_state,
        }
    }
}
