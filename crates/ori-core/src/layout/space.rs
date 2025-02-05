use std::{
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, BitAnd, BitAndAssign, Sub, SubAssign},
};

use super::Size;

/// Space available to lay out a view.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Space {
    /// Minimum size the view can be.
    pub min: Size,
    /// Maximum size the view can be.
    pub max: Size,
}

impl Default for Space {
    fn default() -> Self {
        Self::UNBOUNDED
    }
}

impl Space {
    /// The zero space.
    pub const ZERO: Self = Self {
        min: Size::ZERO,
        max: Size::ZERO,
    };

    /// The unbounded space.
    pub const UNBOUNDED: Self = Self {
        min: Size::ZERO,
        max: Size::UNBOUNDED,
    };

    /// The infinite space.
    pub const FILL: Self = Self::new(Size::FILL, Size::FILL);

    /// Create a new space.
    pub const fn new(min: Size, max: Size) -> Self {
        Self { min, max }
    }

    /// Create a new space from a maximum size.
    pub const fn max(max: Size) -> Self {
        Self::new(Size::ZERO, max)
    }

    /// Create a new space with the same minimum and maximum size.
    pub fn from_size(size: Size) -> Self {
        Self::new(size, size)
    }

    /// Shrink the space by `size`.
    pub fn shrink(self, size: Size) -> Self {
        let min = self.min - size;
        let max = self.max - size;

        Self::new(min.max(Size::ZERO), max.max(Size::ZERO))
    }

    /// Expand the space by `size`.
    pub fn expand(self, size: Size) -> Self {
        Self::new(self.min + size, self.max + size)
    }

    /// Loosen the space, setting the minimum size to zero.
    pub fn loosen(self) -> Self {
        Self::new(Size::ZERO, self.max)
    }

    /// Loosen the width, setting the minimum width to zero.
    pub fn loosen_width(mut self) -> Self {
        self.min.width = 0.0;
        self
    }

    /// Loosen the height, setting the minimum height to zero.
    pub fn loosen_height(mut self) -> Self {
        self.min.height = 0.0;
        self
    }

    /// Get the most constraning space between `self` and `other
    pub fn constrain(self, other: Self) -> Self {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);

        Self::new(min.min(max), max)
    }

    /// Clamp a size to the space.
    pub fn fit(self, size: Size) -> Size {
        let width = if self.min.width.is_finite() {
            size.width.max(self.min.width)
        } else {
            size.width
        };

        let height = if self.min.height.is_finite() {
            size.height.max(self.min.height)
        } else {
            size.height
        };

        Size::new(width.min(self.max.width), height.min(self.max.height))
    }

    /// Get whether the space is finite.
    pub fn is_finite(self) -> bool {
        self.min.is_finite() && self.max.is_finite()
    }

    /// Get whether the space is infinite.
    pub fn is_infinite(self) -> bool {
        self.min.is_infinite() && self.max.is_infinite()
    }
}

impl From<Size> for Space {
    fn from(size: Size) -> Self {
        Self::new(size, size)
    }
}

impl BitAnd for Space {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.constrain(rhs)
    }
}

impl BitAndAssign for Space {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl Add<Size> for Space {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        self.expand(rhs)
    }
}

impl AddAssign<Size> for Space {
    fn add_assign(&mut self, rhs: Size) {
        *self = *self + rhs;
    }
}

impl Sub<Size> for Space {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        self.shrink(rhs)
    }
}

impl SubAssign<Size> for Space {
    fn sub_assign(&mut self, rhs: Size) {
        *self = *self - rhs;
    }
}

impl Hash for Space {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.min.hash(state);
        self.max.hash(state);
    }
}
