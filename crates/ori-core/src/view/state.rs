use std::{
    any::Any,
    fmt::{Debug, Display},
    num::NonZero,
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
    id: NonZero<u64>,
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

            if let Some(id) = NonZero::new(id) {
                break Self { id };
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
#[derive(Debug)]
pub struct ViewState {
    pub(crate) id: ViewId,

    /* flags */
    pub(crate) prev_flags: ViewFlags,
    pub(crate) flags: ViewFlags,
    pub(crate) update: Update,

    /* properties */
    pub(crate) properties: Properties,

    /* layout */
    pub(crate) size: Size,
    pub(crate) transform: Affine,

    /* cursor */
    pub(crate) cursor: Option<Cursor>,
    pub(crate) inherited_cursor: Option<Cursor>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new(ViewId::new())
    }
}

impl ViewState {
    /// Create a new [`ViewState`] with the given [`ViewId`].
    pub fn new(id: ViewId) -> Self {
        Self {
            id,

            /* flags */
            prev_flags: ViewFlags::default(),
            flags: ViewFlags::default(),
            update: Update::LAYOUT | Update::DRAW,

            /* properties */
            properties: Properties::new(),

            /* layout */
            size: Size::ZERO,
            transform: Affine::IDENTITY,

            /* cursor */
            cursor: None,
            inherited_cursor: None,
        }
    }

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
    }

    /// Get whether the view is focused.
    pub fn is_focused(&self) -> bool {
        self.flags.contains(ViewFlags::FOCUSED)
    }

    /// Set whether the view is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.flags.set(ViewFlags::FOCUSED, focused);
    }

    /// Get whether the view is active.
    pub fn is_active(&self) -> bool {
        self.flags.contains(ViewFlags::ACTIVE)
    }

    /// Set whether the view is active.
    pub fn set_active(&mut self, active: bool) {
        self.flags.set(ViewFlags::ACTIVE, active);
    }

    /// Get whether the view has a hot child.
    pub fn has_hot(&self) -> bool {
        let flags = self.flags & (ViewFlags::HOT | ViewFlags::HAS_HOT);
        flags != ViewFlags::empty()
    }

    /// Get whether the view has a focused child.
    pub fn has_focused(&self) -> bool {
        let flags = self.flags & (ViewFlags::FOCUSED | ViewFlags::HAS_FOCUSED);
        flags != ViewFlags::empty()
    }

    /// Get whether the view has an active child.
    pub fn has_active(&self) -> bool {
        let flags = self.flags & (ViewFlags::ACTIVE | ViewFlags::HAS_ACTIVE);
        flags != ViewFlags::empty()
    }

    /// Get whether the view is focusable.
    pub fn is_focusable(&self) -> bool {
        self.flags.contains(ViewFlags::FOCUSABLE)
    }

    /// Set whether the view is focusable.
    pub fn set_focusable(&mut self, focusable: bool) {
        self.flags.set(ViewFlags::FOCUSABLE, focusable);
    }

    /// Check if the view has the property `T`.
    pub fn contains_property<T: 'static>(&self) -> bool {
        self.properties.contains::<T>()
    }

    /// Insert a property into the view.
    pub fn insert_property<T: 'static>(&mut self, item: T) {
        self.properties.insert(item);
    }

    /// Remove a property from the view.
    pub fn remove_property<T: 'static>(&mut self) -> Option<T> {
        self.properties.remove::<T>()
    }

    /// Get the property `T` of the view.
    pub fn get_property<T: 'static>(&self) -> Option<&T> {
        self.properties.get()
    }

    /// Get the property `T` of the view mutably.
    pub fn get_property_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.properties.get_mut()
    }

    /// Get the property `T` of the view or insert it with a value.
    pub fn property_or_insert_with<T: 'static, F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
        self.properties.get_or_insert_with(f)
    }

    /// Get the property `T` of the view or insert it with a value.
    pub fn property_or<T: 'static>(&mut self, item: T) -> &mut T {
        self.properties.get_or_insert(item)
    }

    /// Get the property `T` of the view or insert it with a default value.
    pub fn property_or_default<T: 'static + Default>(&mut self) -> &mut T {
        self.properties.get_or_default()
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

pub(crate) struct Properties {
    items: Vec<Box<dyn Any>>,
}

impl Properties {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn insert<T: 'static>(&mut self, item: T) {
        if let Some(index) = self.get_index::<T>() {
            self.items[index] = Box::new(item);
        } else {
            self.items.push(Box::new(item));
        }
    }

    fn remove<T: 'static>(&mut self) -> Option<T> {
        if let Some(index) = self.get_index::<T>() {
            Some(*self.items.remove(index).downcast().unwrap())
        } else {
            None
        }
    }

    fn contains<T: 'static>(&self) -> bool {
        self.items.iter().any(|item| item.is::<T>())
    }

    fn get<T: 'static>(&self) -> Option<&T> {
        self.items.iter().find_map(|item| item.downcast_ref())
    }

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.items.iter_mut().find_map(|item| item.downcast_mut())
    }

    fn get_index<T: 'static>(&self) -> Option<usize> {
        self.items.iter().position(|item| item.is::<T>())
    }

    fn get_or_insert_with<T: 'static, F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
        if let Some(index) = self.get_index::<T>() {
            self.items[index].downcast_mut().unwrap()
        } else {
            let item = f();
            self.insert(item);
            self.items.last_mut().unwrap().downcast_mut().unwrap()
        }
    }

    fn get_or_insert<T: 'static>(&mut self, item: T) -> &mut T {
        self.get_or_insert_with(|| item)
    }

    fn get_or_default<T: 'static + Default>(&mut self) -> &mut T {
        self.get_or_insert_with(Default::default)
    }
}

impl Debug for Properties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Properties").finish()
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
