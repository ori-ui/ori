use std::ops::{Deref, DerefMut};

use crate::{view::ViewState, window::Window};

use super::BaseCx;

/// A context for building the view tree.
pub struct BuildCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) window: &'a mut Window,
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
    pub fn child(&mut self) -> BuildCx<'_, 'b> {
        BuildCx {
            base: self.base,
            view_state: self.view_state,
            window: self.window,
        }
    }

    /// Request a layout of the view tree.
    pub fn request_layout(&mut self) {
        self.view_state.request_layout();
    }

    /// Request a draw of the view tree.
    pub fn request_draw(&mut self) {
        self.view_state.request_draw();
    }

    /// Request an animation frame.
    pub fn animate(&mut self) {
        self.view_state.request_animate();
    }

    /// Set whether the view is hot.
    ///
    /// Returns `true` if the hot state changed.
    pub fn set_hot(&mut self, hot: bool) -> bool {
        let updated = self.is_hot() != hot;
        self.view_state.set_hot(hot);
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

    /// Set the flex of the view.
    pub fn set_flex(&mut self, flex: f32) {
        self.view_state.set_flex(flex);
    }

    /// Set whether the view is tight.
    pub fn set_tight(&mut self, tight: bool) {
        self.view_state.set_tight(tight);
    }
}
