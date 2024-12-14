use std::{
    any::{Any, TypeId},
    ops::Range,
};

use crate::layout::{Point, Rect, Size};

use super::{FontSource, Paragraph};

/// A trait for managing fonts and text layout.
pub trait Fonts: Any {
    /// Load the given font source.
    ///
    /// If `name` is provided, the fonts will be registered under that name
    /// instead of the default name provided by the source.
    fn load(&mut self, source: FontSource<'_>, name: Option<&str>);

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
    /// The ascent of the line.
    pub ascent: f32,

    /// The descent of the line.
    pub descent: f32,

    /// The left edge of the line.
    pub left: f32,

    /// The width of the line.
    pub width: f32,

    /// The height of the line.
    pub height: f32,

    /// The baseline of the line.
    pub baseline: f32,

    /// The range of the line in the original text.
    pub range: Range<usize>,

    /// The glyphs in the line.
    pub glyphs: Vec<GlyphCluster>,
}

impl TextLayoutLine {
    /// The left edge of the line.
    ///
    /// This is the same as 'left'.
    /// other edge methods.
    pub fn left(&self) -> f32 {
        self.left
    }

    /// The right edge of the line.
    ///
    /// This is the same as `left + width`.
    pub fn right(&self) -> f32 {
        self.left + self.width
    }

    /// The top edge of the line.
    ///
    /// This is the same as `baseline - ascent`.
    pub fn top(&self) -> f32 {
        self.baseline - self.ascent
    }

    /// The bottom edge of the line.
    ///
    /// This is the same as `baseline + descent`.
    pub fn bottom(&self) -> f32 {
        self.baseline + self.descent
    }

    /// The height of the line.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// The height of the line.
    ///
    /// This is the same as `ascent + descent`.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// The bounds of the line.
    pub fn bounds(&self) -> Rect {
        Rect::new(
            Point::new(self.left(), self.top()),
            Point::new(self.right(), self.bottom()),
        )
    }
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
