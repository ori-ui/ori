use std::ops::{Deref, DerefMut};

use crate::{
    layout::{Point, Rect, Size},
    view::ViewState,
    window::Window,
};

use super::{BaseCx, RebuildCx};

/// A context for drawing the view tree.
pub struct DrawCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
}

impl<'a, 'b> Deref for DrawCx<'a, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b> DerefMut for DrawCx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> DrawCx<'a, 'b> {
    /// Create a new draw context.
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
    pub fn child(&mut self) -> DrawCx<'_, 'b> {
        DrawCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
        }
    }

    /// Get a rebuild context.
    pub fn rebuild_cx(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx::new(self.base, self.view_state, self.window)
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
