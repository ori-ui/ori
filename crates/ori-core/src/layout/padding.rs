use glam::Vec2;

use crate::Size;

/// A padding of a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Padding {
    /// The left padding.
    pub left: f32,
    /// The right padding.
    pub right: f32,
    /// The top padding.
    pub top: f32,
    /// The bottom padding.
    pub bottom: f32,
}

impl Padding {
    /// Create a new [`Padding`].
    pub const fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Create a new [`Padding`] with the same value for all sides.
    pub const fn all(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    /// Get the size of the padding.
    pub fn size(&self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }

    /// Get the offset of the padding.
    pub fn offset(&self) -> Vec2 {
        Vec2::new(self.left, self.top)
    }
}

impl From<(f32, f32, f32, f32)> for Padding {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl From<(f32, f32)> for Padding {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.0, value.1, value.1)
    }
}

impl From<f32> for Padding {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}
