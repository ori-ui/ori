use std::ops::{Add, Sub};

use glam::Vec2;

/// A rectangle with a minimum and maximum point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    /// A rectangle with no area.
    pub const ZERO: Self = Self::new(Vec2::ZERO, Vec2::ZERO);

    /// Creates a new rectangle with the given minimum and maximum points.
    pub const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Creates a new rectangle with the given minimum point and size.
    pub fn min_size(min: Vec2, size: Vec2) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    /// Creates a new rectangle with the given center point and size.
    pub fn center_size(center: Vec2, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Rounds the rectangle's minimum and maximum points to the nearest integers.
    pub fn round(self) -> Self {
        Self {
            min: self.min.round(),
            max: self.max.round(),
        }
    }

    /// Rounds the rectangle's minimum and maximum points up to the nearest integers.
    pub fn ceil(self) -> Self {
        Self {
            min: self.min.floor(),
            max: self.max.ceil(),
        }
    }

    /// Rounds the rectangle's minimum and maximum points down to the nearest integers.
    pub fn floor(self) -> Self {
        Self {
            min: self.min.ceil(),
            max: self.max.floor(),
        }
    }

    /// Shrinks the rectangle by the given amount on all sides.
    pub fn shrink<T: Copy>(self, amount: T) -> Self
    where
        Vec2: Add<T, Output = Vec2>,
        Vec2: Sub<T, Output = Vec2>,
    {
        Self {
            min: self.min + amount,
            max: self.max - amount,
        }
    }

    /// Expands the rectangle by the given amount on all sides.
    pub fn expand<T: Copy>(self, amount: T) -> Self
    where
        Vec2: Add<T, Output = Vec2>,
        Vec2: Sub<T, Output = Vec2>,
    {
        Self {
            min: self.min - amount,
            max: self.max + amount,
        }
    }

    /// Returns the size of the rectangle.
    pub fn size(self) -> Vec2 {
        self.max - self.min
    }

    /// Returns the width of the rectangle.
    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    /// Returns the height of the rectangle.
    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    /// Returns the center point of the rectangle.
    pub fn center(self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Returns true if the rectangle contains the given point.
    pub fn contains(self, point: Vec2) -> bool {
        let inside_x = point.x >= self.min.x && point.x <= self.max.x;
        let inside_y = point.y >= self.min.y && point.y <= self.max.y;
        inside_x && inside_y
    }

    /// Returns true if `self` intersects `other`.
    pub fn intersects(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Returns the smallest rectangle that contains both rectangles.
    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Returns the largest rectangle that fits inside both rectangles. If the rectangles do not
    /// intersect, returns [`Rect::ZERO`].
    pub fn intersect(self, other: Self) -> Self {
        if !self.intersects(other) {
            return Self::ZERO;
        }

        Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    /// Returns the left edge of the rectangle.
    pub fn left(self) -> f32 {
        self.min.x
    }

    /// Returns the right edge of the rectangle.
    pub fn right(self) -> f32 {
        self.max.x
    }

    /// Returns the top edge of the rectangle.
    pub fn top(self) -> f32 {
        self.min.y
    }

    /// Returns the bottom edge of the rectangle.
    pub fn bottom(self) -> f32 {
        self.max.y
    }

    /// Returns the top-left corner of the rectangle.
    pub fn top_left(self) -> Vec2 {
        self.min
    }

    /// Returns the top-right corner of the rectangle.
    pub fn top_right(self) -> Vec2 {
        Vec2::new(self.max.x, self.min.y)
    }

    /// Returns the bottom-left corner of the rectangle.
    pub fn bottom_left(self) -> Vec2 {
        Vec2::new(self.min.x, self.max.y)
    }

    /// Returns the bottom-right corner of the rectangle.
    pub fn bottom_right(self) -> Vec2 {
        self.max
    }

    /// Returns the center-left point of the rectangle.
    pub fn right_center(self) -> Vec2 {
        Vec2::new(self.max.x, self.center().y)
    }

    /// Returns the center-right point of the rectangle.
    pub fn left_center(self) -> Vec2 {
        Vec2::new(self.min.x, self.center().y)
    }

    /// Returns the top-center point of the rectangle.
    pub fn top_center(self) -> Vec2 {
        Vec2::new(self.center().x, self.min.y)
    }

    /// Returns the bottom-center point of the rectangle.
    pub fn bottom_center(self) -> Vec2 {
        Vec2::new(self.center().x, self.max.y)
    }

    /// Translates the rectangle by the given offset.
    pub fn translate<T: Copy>(self, offset: T) -> Self
    where
        Vec2: Add<T, Output = Vec2>,
    {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// Pad the rectangle with the given amount on all sides.
    pub fn pad<T: Copy>(self, amount: T) -> Self
    where
        Vec2: Add<T, Output = Vec2>,
        Vec2: Sub<T, Output = Vec2>,
    {
        Self {
            min: self.min - amount,
            max: self.max + amount,
        }
    }
}
