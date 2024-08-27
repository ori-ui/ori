use std::ops::{Deref, DerefMut};

use crate::{
    layout::{Affine, Point, Rect, Size},
    view::{ViewFlags, ViewState},
    window::Cursor,
};

use super::{BaseCx, BuildCx, RebuildCx};

/// A context for handling events.
pub struct EventCx<'a, 'b> {
    pub(crate) base: &'a mut BaseCx<'b>,
    pub(crate) view_state: &'a mut ViewState,
    pub(crate) rebuild: &'a mut bool,
    pub(crate) transform: Affine,
}

impl<'a, 'b> Deref for EventCx<'a, 'b> {
    type Target = BaseCx<'b>;

    fn deref(&self) -> &Self::Target {
        self.base
    }
}

impl<'a, 'b> DerefMut for EventCx<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.base
    }
}

impl<'a, 'b> EventCx<'a, 'b> {
    /// Create a new event context.
    pub fn new(
        base: &'a mut BaseCx<'b>,
        view_state: &'a mut ViewState,
        rebuild: &'a mut bool,
    ) -> Self {
        let transform = view_state.transform;

        Self {
            base,
            view_state,
            rebuild,
            transform,
        }
    }

    /// Create a child context.
    pub fn child(&mut self) -> EventCx<'_, 'b> {
        EventCx {
            base: self.base,
            view_state: self.view_state,
            rebuild: self.rebuild,
            transform: self.transform,
        }
    }

    /// Get a build context.
    pub fn build_cx(&mut self) -> BuildCx<'_, 'b> {
        BuildCx::new(self.base, self.view_state)
    }

    /// Get a rebuild context.
    pub fn rebuild_cx(&mut self) -> RebuildCx<'_, 'b> {
        RebuildCx::new(self.base, self.view_state)
    }

    /// Get the size of the view.
    pub fn size(&self) -> Size {
        self.view_state.size
    }

    /// Get the rect of the view in local space.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Point::ZERO, self.size())
    }

    /// Get the transform of the view.
    pub fn transform(&self) -> Affine {
        self.transform
    }

    /// Transform a point from global space to local space.
    pub fn local(&self, point: Point) -> Point {
        self.transform.inverse() * point
    }

    /// Request a rebuild of the view tree.
    pub fn request_rebuild(&mut self) {
        *self.rebuild = true;
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

    /// Set the cursor of the view.
    pub fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.view_state.set_cursor(cursor);
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

    /// Get whether the view was hot last call.
    pub fn was_hot(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HOT)
    }

    /// Get whether the view was focused last call.
    pub fn was_focused(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::FOCUSED)
    }

    /// Get whether the view was active last call.
    pub fn was_active(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::ACTIVE)
    }

    /// Get whether a child view was hot last call.
    pub fn had_hot(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HAS_HOT)
    }

    /// Get whether a child view was focused last call.
    pub fn had_focused(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HAS_FOCUSED)
    }

    /// Get whether a child view was active last call.
    pub fn had_active(&self) -> bool {
        self.view_state.prev_flags.contains(ViewFlags::HAS_ACTIVE)
    }

    /// Get whether the view's hot state changed.
    pub fn hot_changed(&self) -> bool {
        self.was_hot() != self.is_hot()
    }

    /// Get whether the view's focused state changed.
    pub fn focused_changed(&self) -> bool {
        self.was_focused() != self.is_focused()
    }

    /// Get whether the view's active state changed.
    pub fn active_changed(&self) -> bool {
        self.was_active() != self.is_active()
    }

    /// Get whether a child view's hot state changed.
    pub fn has_hot_changed(&self) -> bool {
        self.had_hot() != self.has_hot()
    }

    /// Get whether a child view's focused state changed.
    pub fn has_focused_changed(&self) -> bool {
        self.had_focused() != self.has_focused()
    }

    /// Get whether a child view's active state changed.
    pub fn has_active_changed(&self) -> bool {
        self.had_active() != self.has_active()
    }
}
