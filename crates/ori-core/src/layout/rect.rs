use std::ops::{Add, AddAssign, Sub, SubAssign};

use glam::Vec2;

use crate::Size;

/// A rectangle defined by its minimum and maximum points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// The minimum point of the rectangle.
    pub min: Vec2,
    /// The maximum point of the rectangle.
    pub max: Vec2,
}

impl Rect {
    /// A rectangle with zero area.
    pub const ZERO: Self = Self::new(Vec2::ZERO, Vec2::ZERO);

    /// Create a new rectangle with the given minimum and maximum points.
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Create a new rectangle with the given minimum point and size.
    pub fn min_size(min: Vec2, size: Size) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    /// Create a new rectangle with the given maximum point and size.
    pub fn max_size(max: Vec2, size: Size) -> Self {
        Self {
            min: max - size,
            max,
        }
    }

    /// Create a new rectangle with the given center point and size.
    pub fn center_size(center: Vec2, size: Size) -> Self {
        Self {
            min: center - size / 2.0,
            max: center + size / 2.0,
        }
    }

    /// Round the rectangle's minimum point down and its maximum point up.
    pub fn round(self) -> Self {
        Self {
            min: self.min.floor(),
            max: self.max.ceil(),
        }
    }

    /// Get the size of the rectangle.
    pub fn size(self) -> Vec2 {
        self.max - self.min
    }

    /// Get the width of the rectangle.
    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    /// Get the height of the rectangle.
    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    /// Get the top left point of the rectangle.
    pub fn top_left(self) -> Vec2 {
        self.min
    }

    /// Get the top right point of the rectangle.
    pub fn top_right(self) -> Vec2 {
        Vec2::new(self.max.x, self.min.y)
    }

    /// Get the bottom left point of the rectangle.
    pub fn bottom_left(self) -> Vec2 {
        Vec2::new(self.min.x, self.max.y)
    }

    /// Get the bottom right point of the rectangle.
    pub fn bottom_right(self) -> Vec2 {
        self.max
    }

    /// Returns whether the rectangle contains the given point.
    pub fn contains(self, point: Vec2) -> bool {
        let x = point.x >= self.min.x && point.x <= self.max.x;
        let y = point.y >= self.min.y && point.y <= self.max.y;
        x && y
    }
}

impl Add<Vec2> for Rect {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl AddAssign<Vec2> for Rect {
    fn add_assign(&mut self, rhs: Vec2) {
        self.min += rhs;
        self.max += rhs;
    }
}

impl Sub<Vec2> for Rect {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl SubAssign<Vec2> for Rect {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.min -= rhs;
        self.max -= rhs;
    }
}
