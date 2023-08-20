use super::Size;

/// Space available to lay out a view.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub const INFINITE: Self = Self {
        min: Size::UNBOUNDED,
        max: Size::UNBOUNDED,
    };

    /// Create a new space.
    pub const fn new(min: Size, max: Size) -> Self {
        Self { min, max }
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

    /// Get the most constraning space between `self` and `other
    pub fn with(self, other: Self) -> Self {
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
}
