use std::{
    fmt::{Debug, Display},
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

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
        const LAYOUT = 1 << 1;
        /// The view needs to be drawn.
        const DRAW = 1 << 2;
        /// The view needs an animation frame.
        const ANIMATE = 1 << 3;
    }
}

bitflags::bitflags! {
    /// Flags that indicate state of a view.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct ViewFlags: u8 {
        /// The view is hot.
        const HOT = 1 << 0;
        /// The view is focused.
        const FOCUSED = 1 << 1;
        /// The view is active.
        const ACTIVE = 1 << 2;
        /// The view has a hot child.
        const HAS_HOT = 1 << 3;
        /// The view has a focused child.
        const HAS_FOCUSED = 1 << 4;
        /// The view has an active child.
        const HAS_ACTIVE = 1 << 5;
        /// The view is focusable.
        const FOCUSABLE = 1 << 6;

        /// Equivalent to `Self::HOT | Self::FOCUSED | Self::ACTIVE`.
        const IS = Self::HOT.bits() | Self::FOCUSED.bits() | Self::ACTIVE.bits();

        /// Equivalent to `Self::HAS_HOT | Self::HAS_FOCUSED | Self::HAS_ACTIVE`.
        const HAS = Self::HAS_HOT.bits() | Self::HAS_FOCUSED.bits() | Self::HAS_ACTIVE.bits();
    }
}

impl ViewFlags {
    fn has(self) -> Self {
        (self & Self::HAS) | Self::from_bits_retain((self & Self::IS).bits() << 3)
    }
}

/// An opaque unique identifier for a view.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewId {
    id: NonZeroU64,
}

impl Default for ViewId {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewId {
    /// Create a new [`ViewId`].
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        loop {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

            if id == 0 {
                continue;
            }

            break Self {
                // SAFETY: `id` is never 0.
                id: unsafe { NonZeroU64::new_unchecked(id) },
            };
        }
    }

    /// Get the underlying id as a [`u64`].
    pub fn as_u64(&self) -> u64 {
        self.id.get()
    }
}

impl Debug for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ViewId(0x{:x})", self.id)
    }
}

impl Display for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.id)
    }
}

/// State associated with a [`View`](super::View).
#[derive(Clone, Debug)]
pub struct ViewState {
    pub(crate) id: ViewId,
    /* flags */
    pub(crate) prev_flags: ViewFlags,
    pub(crate) flags: ViewFlags,
    pub(crate) update: Update,
    /* layout */
    pub(crate) flex: f32,
    pub(crate) is_tight: bool,
    pub(crate) size: Size,
    pub(crate) transform: Affine,
    /* cursor */
    pub(crate) cursor: Option<Cursor>,
    pub(crate) inherited_cursor: Option<Cursor>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            id: ViewId::new(),
            /* flags */
            prev_flags: ViewFlags::default(),
            flags: ViewFlags::default(),
            update: Update::LAYOUT | Update::DRAW,
            /* layout */
            flex: 0.0,
            is_tight: false,
            size: Size::ZERO,
            transform: Affine::IDENTITY,
            /* cursor */
            cursor: None,
            inherited_cursor: None,
        }
    }
}

impl ViewState {
    /// Prepare the view.
    pub fn prepare(&mut self) {
        self.flags.remove(ViewFlags::HAS);
        self.flags |= self.flags.has();

        self.inherited_cursor = self.cursor;
    }

    /// Propagate the state of a child view.
    pub fn propagate(&mut self, child: &mut Self) {
        self.update |= child.update;
        self.flags |= child.flags.has();
        self.inherited_cursor = self.cursor().or(child.cursor());
    }

    /// Get the id of the view.
    pub fn id(&self) -> ViewId {
        self.id
    }

    /// Get whether the view is hot.
    pub fn is_hot(&self) -> bool {
        self.flags.contains(ViewFlags::HOT)
    }

    /// Set whether the view is hot.
    pub fn set_hot(&mut self, hot: bool) {
        self.flags.set(ViewFlags::HOT, hot);
        self.flags.set(ViewFlags::HAS_HOT, hot);
    }

    /// Get whether the view is focused.
    pub fn is_focused(&self) -> bool {
        self.flags.contains(ViewFlags::FOCUSED)
    }

    /// Set whether the view is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.flags.set(ViewFlags::FOCUSED, focused);
        self.flags.set(ViewFlags::HAS_FOCUSED, focused);
    }

    /// Get whether the view is active.
    pub fn is_active(&self) -> bool {
        self.flags.contains(ViewFlags::ACTIVE)
    }

    /// Set whether the view is active.
    pub fn set_active(&mut self, active: bool) {
        self.flags.set(ViewFlags::ACTIVE, active);
        self.flags.set(ViewFlags::HAS_ACTIVE, active);
    }

    /// Get whether the view has a hot child.
    pub fn has_hot(&self) -> bool {
        self.flags.contains(ViewFlags::HAS_HOT)
    }

    /// Get whether the view has a focused child.
    pub fn has_focused(&self) -> bool {
        self.flags.contains(ViewFlags::HAS_FOCUSED)
    }

    /// Get whether the view has an active child.
    pub fn has_active(&self) -> bool {
        self.flags.contains(ViewFlags::HAS_ACTIVE)
    }

    /// Get whether the view is focusable.
    pub fn is_focusable(&self) -> bool {
        self.flags.contains(ViewFlags::FOCUSABLE)
    }

    /// Set whether the view is focusable.
    pub fn set_focusable(&mut self, focusable: bool) {
        self.flags.set(ViewFlags::FOCUSABLE, focusable);
    }

    /// Get the flex of the view.
    pub fn flex(&self) -> f32 {
        self.flex
    }

    /// Set the flex of the view.
    pub fn set_flex(&mut self, flex: f32) {
        self.flex = flex;
    }

    /// Get whether the view is flexible.
    pub fn is_flex(&self) -> bool {
        self.flex > 0.0
    }

    /// Get whether the view is tight.
    pub fn is_tight(&self) -> bool {
        self.is_tight
    }

    /// Set whether the view is tight.
    pub fn set_tight(&mut self, tight: bool) {
        self.is_tight = tight;
    }

    /// Set the size of the view.
    pub fn set_size(&mut self, size: Size) {
        self.size = size;
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

    /// Request a layout of the view tree.
    pub fn request_layout(&mut self) {
        self.update |= Update::LAYOUT | Update::DRAW;
    }

    /// Request a draw of the view tree.
    pub fn request_draw(&mut self) {
        self.update |= Update::DRAW;
    }

    /// Request an animation frame of the view tree.
    pub fn request_animate(&mut self) {
        self.update |= Update::ANIMATE;
    }

    /// Get whether the view needs to be laid out.
    pub fn needs_layout(&self) -> bool {
        self.update.contains(Update::LAYOUT)
    }

    /// Get whether the view needs to be drawn.
    pub fn needs_draw(&self) -> bool {
        self.update.contains(Update::DRAW)
    }

    /// Get whether the view needs an animation frame.
    pub fn needs_animate(&self) -> bool {
        self.update.contains(Update::ANIMATE)
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

    /// Mark the view as animated.
    ///
    /// This will remove the [`Update::ANIMATE`] flag.
    pub fn mark_animated(&mut self) {
        self.update.remove(Update::ANIMATE);
    }

    /// Get the flags of the view.
    pub fn flags(&self) -> ViewFlags {
        self.flags
    }

    /// Get the [`Update`] of the view.
    pub fn update(&self) -> Update {
        self.update
    }

    /// Get the cursor of the view.
    pub fn cursor(&self) -> Option<Cursor> {
        self.cursor.or(self.inherited_cursor)
    }

    /// Set the cursor of the view.
    pub fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.cursor = cursor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagate() {
        assert_eq!(ViewFlags::HOT.has(), ViewFlags::HAS_HOT);
        assert_eq!(ViewFlags::FOCUSED.has(), ViewFlags::HAS_FOCUSED);
        assert_eq!(ViewFlags::ACTIVE.has(), ViewFlags::HAS_ACTIVE);
    }
}
