use std::{
    any::{Any, TypeId},
    ops::Range,
};

use crate::layout::{Rect, Size};

use super::{FontSource, Paragraph};

/// A trait for managing fonts and text layout.
pub trait Fonts: Any {
    /// Load the given font source.
    fn load(&mut self, source: FontSource<'_>);

    /// Layout the given paragraph with the given max width.
    fn layout(&mut self, paragraph: &Paragraph, width: f32) -> Vec<TextLayoutLine>;

    /// Measure the given paragraph with the given max width.
    fn measure(&mut self, paragraph: &Paragraph, width: f32) -> Size;
}

impl dyn Fonts {
    /// Attempt to downcast a reference to a concrete type.
    pub fn downcast_ref<T: Fonts>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            let ptr = self as *const dyn Fonts as *const T;

            // SAFETY: We just checked that the type ID is correct.
            unsafe { Some(&*ptr) }
        } else {
            None
        }
    }

    /// Attempt to downcast a mutable reference to a concrete type.
    pub fn downcast_mut<T: Fonts>(&mut self) -> Option<&mut T> {
        if (*self).type_id() == TypeId::of::<T>() {
            let ptr = self as *mut dyn Fonts as *mut T;

            // SAFETY: We just checked that the type ID is correct.
            unsafe { Some(&mut *ptr) }
        } else {
            None
        }
    }
}

/// A line of text layout.
#[derive(Clone, Debug)]
pub struct TextLayoutLine {
    /// The width of the line.
    pub width: f32,

    /// The height of the line.
    pub height: f32,

    /// The baseline of the line.
    pub baseline: f32,

    /// The glyphs in the line.
    pub glyphs: Vec<GlyphCluster>,
}

/// A glyph cluster in a line of laid out text.
#[derive(Clone, Debug)]
pub struct GlyphCluster {
    /// The bounds of the cluster in local space.
    pub bounds: Rect,

    /// The byte range of the cluster in the original text.
    pub range: Range<usize>,

    /// The direction of the cluster.
    pub direction: TextDirection,
}

/// The direction of text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextDirection {
    /// Left-to-right text.
    Ltr,

    /// Right-to-left text.
    Rtl,
}
