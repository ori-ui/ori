use std::ops::{Deref, DerefMut};

use crate::{event::RequestFocus, view::ViewState, window::Cursor};

use super::BaseCx;

/// A context for building the view tree.
pub struct BuildCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
}

impl<'a, 'b> Deref for BuildCx<'a, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b> DerefMut for BuildCx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> BuildCx<'a, 'b> {
    /// Create a new build context.
    pub fn new(base: &'a mut BaseCx<'b>, view_state: &'a mut ViewState) -> Self {
        Self { base, view_state }
    }

    /// Create a child context.
    pub fn child(&mut self) -> BuildCx<'_, 'b> {
        BuildCx {
            base: self.base,
            view_state: self.view_state,
        }
    }

    /// Request a layout of the view tree.
    pub fn layout(&mut self) {
        self.view_state.request_layout();
    }

    /// Request a draw of the view tree.
    pub fn draw(&mut self) {
        self.view_state.request_draw();
    }

    /// Request an animation frame.
    pub fn animate(&mut self) {
        self.view_state.request_animate();
    }

    /// Request focus for the view.
    pub fn request_focus(&mut self) {
        let cmd = RequestFocus(self.window().id(), self.id());
        self.cmd(cmd);
    }

    /// Set the cursor of the view.
    pub fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.view_state.set_cursor(cursor);
    }

    /// Set whether the view is hovered.
    ///
    /// Returns `true` if the hovered state changed.
    pub fn set_hovered(&mut self, hovered: bool) -> bool {
        let updated = self.is_hovered() != hovered;
        self.view_state.set_hovered(hovered);
        updated
    }

    /// Set whether the view is focused.
    ///
    /// Returns `true` if the focused state changed.
    pub fn set_focused(&mut self, focused: bool) -> bool {
        let updated = self.is_focused() != focused;
        self.view_state.set_focused(focused);
        updated
    }

    /// Set whether the view is active.
    ///
    /// Returns `true` if the active state changed.
    pub fn set_active(&mut self, active: bool) -> bool {
        let updated = self.is_active() != active;
        self.view_state.set_active(active);
        updated
    }

    /// Set whether the view is focusable.
    ///
    /// Returns `true` if the focusable state changed.
    pub fn set_focusable(&mut self, focusable: bool) -> bool {
        let updated = self.is_focusable() != focusable;
        self.view_state.set_focusable(focusable);
        updated
    }
}
