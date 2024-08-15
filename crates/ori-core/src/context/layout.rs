use std::ops::{Deref, DerefMut};

use crate::{view::ViewState, window::Window};

use super::BaseCx;

/// A context for laying out the view tree.
pub struct LayoutCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
}

impl<'a, 'b> Deref for LayoutCx<'a, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b> DerefMut for LayoutCx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> LayoutCx<'a, 'b> {
    /// Create a new layout context.
    pub fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        window: &'a mut Window,
    ) -> Self {
        Self {
            base,
            view_state,
            window,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> LayoutCx<'_, 'b> {
        LayoutCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
        }
    }
}
