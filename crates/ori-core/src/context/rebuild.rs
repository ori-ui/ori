use std::ops::{Deref, DerefMut};

use crate::{
    layout::{Point, Rect, Size},
    view::ViewState,
};

use super::{BaseCx, BuildCx, LayoutCx};

/// A context for rebuilding the view tree.
pub struct RebuildCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
}

impl<'b> Deref for RebuildCx<'_, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl DerefMut for RebuildCx<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> RebuildCx<'a, 'b> {
    /// Create a new rebuild context.
    pub fn new(base: &'a mut BaseCx<'b>, view_state: &'a mut ViewState) -> Self {
        Self { base, view_state }
    }

    /// Create a child context.
    pub fn child(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx {
            base: self.base,
            view_state: self.view_state,
        }
    }

    /// Get a build context.
    pub fn as_build_cx(&mut self) -> BuildCx<'_, 'b> {
        BuildCx::new(self.base, self.view_state)
    }

    /// Get a layout context.
    pub fn as_layout_cx(&mut self) -> LayoutCx<'_, 'b> {
        LayoutCx::new(self.base, self.view_state)
    }

    /// Get the size of the view.
    pub fn size(&self) -> Size {
        self.view_state.size
    }

    /// Get the rect of the view in local space.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Point::ZERO, self.size())
    }
}
