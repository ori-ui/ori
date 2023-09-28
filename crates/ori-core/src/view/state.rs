use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    layout::{Affine, Point, Rect, Size, Vector},
    window::Cursor,
};

bitflags::bitflags! {
    /// Flags that indicate what needs to be updated.
    #[must_use]
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct Update: u8 {
        /// The view needs to be laid out.
        const LAYOUT = 1 << 0;
        /// The view needs to be drawn.
        const DRAW = 1 << 1;
    }
}

/// An opaque unique identifier for a view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewId {
    id: usize,
}

impl Default for ViewId {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewId {
    /// Create a new [`ViewId`].
    pub fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self { id }
    }
}

/// State associated with a [`View`](super::View).
#[derive(Clone, Debug)]
pub struct ViewState {
    pub(crate) id: ViewId,
    /* flags */
    pub(crate) hot: bool,
    pub(crate) focused: bool,
    pub(crate) active: bool,
    pub(crate) has_active: bool,
    pub(crate) update: Update,
    /* layout */
    pub(crate) flex_grow: f32,
    pub(crate) flex_shrink: f32,
    pub(crate) size: Size,
    pub(crate) transform: Affine,
    /* cursor */
    pub(crate) cursor: Option<Cursor>,
    pub(crate) has_cursor: bool,
    /* input */
    pub(crate) soft_input: bool,
    pub(crate) has_soft_input: bool,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            id: ViewId::new(),
            /* flags */
            hot: false,
            focused: false,
            active: false,
            has_active: false,
            update: Update::LAYOUT | Update::DRAW,
            /* layout */
            flex_grow: 0.0,
            flex_shrink: 0.0,
            size: Size::ZERO,
            transform: Affine::IDENTITY,
            /* cursor */
            cursor: None,
            has_cursor: false,
            /* input */
            soft_input: false,
            has_soft_input: false,
        }
    }
}

impl ViewState {
    pub(crate) fn prepare(&mut self) {
        self.has_active = false;
        self.has_cursor = false;
        self.has_soft_input = false;
    }

    pub(crate) fn prepare_layout(&mut self) {
        self.prepare();
        self.mark_layed_out();
    }

    pub(crate) fn prepare_draw(&mut self) {
        self.prepare();
        self.mark_drawn();
    }

    pub(crate) fn propagate(&mut self, child: &mut Self) {
        self.has_active |= child.active || child.has_active;
        self.has_cursor |= child.has_cursor || child.cursor.is_some();
        self.has_soft_input |= self.has_soft_input || child.has_soft_input || child.soft_input;
        self.update |= child.update;
    }
}

impl ViewState {
    /// Get the id of the view.
    pub fn id(&self) -> ViewId {
        self.id
    }

    /// Get whether the view is hot.
    pub fn is_hot(&self) -> bool {
        self.hot
    }

    /// Set whether the view is hot.
    pub fn set_hot(&mut self, hot: bool) {
        self.hot = hot;
    }

    /// Get whether the view is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set whether the view is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Get whether the view is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Set whether the view is active.
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Get whether the view has an active child.
    pub fn has_active(&self) -> bool {
        self.has_active
    }

    /// Get whether the view has a child with a cursor.
    pub fn has_cursor(&self) -> bool {
        self.has_cursor
    }

    /// Get whether the view has a child with soft input.
    pub fn has_soft_input(&self) -> bool {
        self.has_soft_input
    }

    /// Get the flex grow of the view.
    pub fn flex_grow(&self) -> f32 {
        self.flex_grow
    }

    /// Get the flex shrink of the view.
    pub fn flex_shrink(&self) -> f32 {
        self.flex_shrink
    }

    /// Set the flex grow of the view.
    pub fn set_flex_grow(&mut self, flex: f32) {
        self.flex_grow = flex;
    }

    /// Set the flex shrink of the view.
    pub fn set_flex_shrink(&mut self, flex: f32) {
        self.flex_shrink = flex;
    }

    /// Get whether the view is growable.
    pub fn is_grow(&self) -> bool {
        self.flex_grow > 0.0
    }

    /// Get whether the view is shrinkable.
    pub fn is_shrink(&self) -> bool {
        self.flex_shrink > 0.0
    }

    /// Get whether the view is flexible.
    pub fn is_flex(&self) -> bool {
        self.is_grow() || self.is_shrink()
    }

    /// Get the size of the view.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get the rect of the view in local coordinates.
    pub fn rect(&self) -> Rect {
        Rect::min_size(Point::ZERO, self.size)
    }

    /// Get the transform of the view.
    pub fn transform(&self) -> Affine {
        self.transform
    }

    /// Set the transform of the view.
    pub fn set_transform(&mut self, transform: Affine) {
        self.transform = transform;
    }

    /// Translate the transform of the view.
    pub fn translate(&mut self, translation: Vector) {
        self.transform = Affine::translate(translation);
    }

    /// Get the cursor of the view.
    pub fn cursor(&self) -> Option<Cursor> {
        self.cursor
    }

    /// Set the cursor of the view.
    pub fn set_cursor(&mut self, cursor: impl Into<Option<Cursor>>) {
        self.cursor = cursor.into();
    }

    /// Set whether the view has soft input.
    pub fn set_soft_input(&mut self, soft_input: bool) {
        self.soft_input = soft_input;
    }

    /// Request a layout of the view tree.
    pub fn request_layout(&mut self) {
        self.update |= Update::LAYOUT | Update::DRAW;
    }

    /// Request a draw of the view tree.
    pub fn request_draw(&mut self) {
        self.update |= Update::DRAW;
    }

    /// Get whether the view needs to be laid out.
    pub fn needs_layout(&self) -> bool {
        self.update.contains(Update::LAYOUT)
    }

    /// Get whether the view needs to be drawn.
    pub fn needs_draw(&self) -> bool {
        self.update.contains(Update::DRAW)
    }

    /// Mark the view as laid out.
    ///
    /// This will remove the [`Update::LAYOUT`] flag.
    pub fn mark_layed_out(&mut self) {
        self.update.remove(Update::LAYOUT);
    }

    /// Mark the view as drawn.
    ///
    /// This will remove the [`Update::DRAW`] flag.
    pub fn mark_drawn(&mut self) {
        self.update.remove(Update::DRAW);
    }

    /// Get the [`Update`] of the view.
    pub fn update(&self) -> Update {
        self.update
    }
}
