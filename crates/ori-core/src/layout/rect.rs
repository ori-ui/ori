use std::ops::{Add, AddAssign, BitAnd, BitAndAssign, Sub, SubAssign};

use glam::Vec2;

use super::{Affine, Size};

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
            min: self.min.round(),
            max: self.max.round(),
        }
    }

    /// Clamp the rectangle to the given rectangle.
    pub fn clamp(self, other: Self) -> Self {
        Self {
            min: self.min.clamp(other.min, other.max),
            max: self.max.clamp(other.min, other.max),
        }
    }

    /// Get the size of the rectangle.
    pub fn size(self) -> Size {
        Size::from(self.max - self.min)
    }

    /// Get the width of the rectangle.
    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    /// Get the height of the rectangle.
    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    /// Get the area of the rectangle.
    pub fn area(self) -> f32 {
        self.width() * self.height()
    }

    /// Get the center point of the rectangle.
    pub fn center(self) -> Vec2 {
        self.min + self.size() / 2.0
    }

    /// Get the top left point of the rectangle.
    pub fn top_left(self) -> Vec2 {
        self.min
    }

    /// Get the top center point of the rectangle.
    pub fn top(self) -> Vec2 {
        Vec2::new(self.center().x, self.min.y)
    }

    /// Get the top right point of the rectangle.
    pub fn top_right(self) -> Vec2 {
        Vec2::new(self.max.x, self.min.y)
    }

    /// Get the left center point of the rectangle.
    pub fn left(self) -> Vec2 {
        Vec2::new(self.min.x, self.center().y)
    }

    /// Get the right center point of the rectangle.
    pub fn right(self) -> Vec2 {
        Vec2::new(self.max.x, self.center().y)
    }

    /// Get the bottom left point of the rectangle.
    pub fn bottom_left(self) -> Vec2 {
        Vec2::new(self.min.x, self.max.y)
    }

    /// Get the bottom center point of the rectangle.
    pub fn bottom(self) -> Vec2 {
        Vec2::new(self.center().x, self.max.y)
    }

    /// Get the bottom right point of the rectangle.
    pub fn bottom_right(self) -> Vec2 {
        self.max
    }

    /// Compute whether the rectangle contains the given point.
    pub fn contains(self, point: Vec2) -> bool {
        let x = point.x >= self.min.x && point.x <= self.max.x;
        let y = point.y >= self.min.y && point.y <= self.max.y;
        x && y
    }

    /// Compute the closest point in the rectangle to the given point.
    pub fn contain(self, point: Vec2) -> Vec2 {
        let x = point.x.max(self.min.x).min(self.max.x);
        let y = point.y.max(self.min.y).min(self.max.y);
        Vec2::new(x, y)
    }

    /// Compute the intersection of the rectangle with the given rectangle.
    pub fn try_intersect(self, other: Self) -> Option<Self> {
        let min_x = f32::max(self.min.x, other.min.x);
        let min_y = f32::max(self.min.y, other.min.y);
        let max_x = f32::min(self.max.x, other.max.x);
        let max_y = f32::min(self.max.y, other.max.y);

        if min_x <= max_x && min_y <= max_y {
            Some(Self {
                min: Vec2::new(min_x, min_y),
                max: Vec2::new(max_x, max_y),
            })
        } else {
            None
        }
    }

    /// Compute the intersection of the rectangle with the given rectangle.
    ///
    /// If the rectangles do not intersect, the zero rectangle is returned.
    pub fn intersect(self, other: Self) -> Self {
        self.try_intersect(other).unwrap_or(Self::ZERO)
    }

    /// Transform the rectangle by the given affine transform.
    pub fn transform(self, transform: Affine) -> Self {
        let top_left = transform * self.top_left();
        let top_right = transform * self.top_right();
        let bottom_left = transform * self.bottom_left();
        let bottom_right = transform * self.bottom_right();

        let min_x = f32::min(
            f32::min(top_left.x, top_right.x),
            f32::min(bottom_left.x, bottom_right.x),
        );
        let min_y = f32::min(
            f32::min(top_left.y, top_right.y),
            f32::min(bottom_left.y, bottom_right.y),
        );

        let max_x = f32::max(
            f32::max(top_left.x, top_right.x),
            f32::max(bottom_left.x, bottom_right.x),
        );
        let max_y = f32::max(
            f32::max(top_left.y, top_right.y),
            f32::max(bottom_left.y, bottom_right.y),
        );

        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
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

impl Add<Size> for Rect {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Self {
            min: self.min,
            max: self.max + rhs,
        }
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

impl Sub<Size> for Rect {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        Self {
            min: self.min,
            max: self.max - rhs,
        }
    }
}

impl AddAssign<Vec2> for Rect {
    fn add_assign(&mut self, rhs: Vec2) {
        self.min += rhs;
        self.max += rhs;
    }
}

impl AddAssign<Size> for Rect {
    fn add_assign(&mut self, rhs: Size) {
        self.max += rhs;
    }
}

impl SubAssign<Vec2> for Rect {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.min -= rhs;
        self.max -= rhs;
    }
}

impl SubAssign<Size> for Rect {
    fn sub_assign(&mut self, rhs: Size) {
        self.max -= rhs;
    }
}

impl BitAnd for Rect {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersect(rhs)
    }
}

impl BitAndAssign for Rect {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersect(rhs);
    }
}
