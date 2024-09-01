use std::ops::{Add, AddAssign, BitAnd, BitAndAssign, Sub, SubAssign};

use super::{Affine, Point, Size, Vector};

/// A rectangle defined by its minimum and maximum points.
#[derive(Clone, Copy, Debug, Default, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    /// The minimum point of the rectangle.
    pub min: Point,
    /// The maximum point of the rectangle.
    pub max: Point,
}

impl Rect {
    /// A rectangle with zero area.
    pub const ZERO: Self = Self::new(Point::ZERO, Point::ZERO);

    /// Create a new rectangle with the given minimum and maximum points.
    pub const fn new(min: Point, max: Point) -> Self {
        Self { min, max }
    }

    /// Create a new rectangle with the given minimum point and size.
    pub fn min_size(min: Point, size: Size) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    /// Create a new rectangle with the given maximum point and size.
    pub fn max_size(max: Point, size: Size) -> Self {
        Self {
            min: max - size,
            max,
        }
    }

    /// Create a new rectangle with the given center point and size.
    pub fn center_size(center: Point, size: Size) -> Self {
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
    pub fn clamp(self, other: impl Into<Self>) -> Self {
        let other = other.into();

        Self {
            min: self.min.clamp(other.min, other.max),
            max: self.max.clamp(other.min, other.max),
        }
    }

    /// Get the offset of the rectangle.
    pub fn offset(self) -> Vector {
        self.min.to_vector()
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

    /// Get the top left point of the rectangle.
    pub fn top_left(self) -> Point {
        self.min
    }

    /// Get the top center point of the rectangle.
    pub fn top_center(self) -> Point {
        Point::new(self.center().x, self.min.y)
    }

    /// Get the top right point of the rectangle.
    pub fn top_right(self) -> Point {
        Point::new(self.max.x, self.min.y)
    }

    /// Get the left center point of the rectangle.
    pub fn center_left(self) -> Point {
        Point::new(self.min.x, self.center().y)
    }

    /// Get the center point of the rectangle.
    pub fn center(self) -> Point {
        self.min + self.size() / 2.0
    }

    /// Get the right center point of the rectangle.
    pub fn center_right(self) -> Point {
        Point::new(self.max.x, self.center().y)
    }

    /// Get the bottom left point of the rectangle.
    pub fn bottom_left(self) -> Point {
        Point::new(self.min.x, self.max.y)
    }

    /// Get the bottom center point of the rectangle.
    pub fn bottom_center(self) -> Point {
        Point::new(self.center().x, self.max.y)
    }

    /// Get the bottom right point of the rectangle.
    pub fn bottom_right(self) -> Point {
        self.max
    }

    /// Get the top edge of the rectangle.
    pub fn top(self) -> f32 {
        self.min.y
    }

    /// Get the left edge of the rectangle.
    pub fn left(self) -> f32 {
        self.min.x
    }

    /// Get the right edge of the rectangle.
    pub fn right(self) -> f32 {
        self.max.x
    }

    /// Get the bottom edge of the rectangle.
    pub fn bottom(self) -> f32 {
        self.max.y
    }

    /// Shrink the rectangle by the given amount.
    pub fn shrink(self, padding: f32) -> Self {
        Self {
            min: self.min + Vector::new(padding, padding),
            max: self.max - Vector::new(padding, padding),
        }
    }

    /// Expand the rectangle by the given amount.
    pub fn inflate(self, padding: f32) -> Self {
        self.shrink(-padding)
    }

    /// Compute whether the rectangle contains the given point.
    pub fn contains(self, point: Point) -> bool {
        let x = point.x >= self.min.x && point.x <= self.max.x;
        let y = point.y >= self.min.y && point.y <= self.max.y;
        x && y
    }

    /// Compute the closest point in the rectangle to the given point.
    pub fn contain(self, point: Point) -> Point {
        let x = point.x.max(self.min.x).min(self.max.x);
        let y = point.y.max(self.min.y).min(self.max.y);
        Point::new(x, y)
    }

    /// Expand the rectangle to contain the given point.
    pub fn include(self, point: Point) -> Self {
        let min_x = f32::min(self.min.x, point.x);
        let min_y = f32::min(self.min.y, point.y);
        let max_x = f32::max(self.max.x, point.x);
        let max_y = f32::max(self.max.y, point.y);

        Self {
            min: Point::new(min_x, min_y),
            max: Point::new(max_x, max_y),
        }
    }

    /// Compute the intersection of the rectangle with the given rectangle.
    pub fn try_intersection(self, other: Self) -> Option<Self> {
        let min_x = f32::max(self.min.x, other.min.x);
        let min_y = f32::max(self.min.y, other.min.y);
        let max_x = f32::min(self.max.x, other.max.x);
        let max_y = f32::min(self.max.y, other.max.y);

        if min_x <= max_x && min_y <= max_y {
            Some(Self {
                min: Point::new(min_x, min_y),
                max: Point::new(max_x, max_y),
            })
        } else {
            None
        }
    }

    /// Compute the intersection of the rectangle with the given rectangle.
    ///
    /// If the rectangles do not intersect [`Rect::ZERO`] is returned.
    pub fn intersection(self, other: Self) -> Self {
        self.try_intersection(other).unwrap_or(Self::ZERO)
    }

    /// Check if the rectangle intersects the given rectangle.
    pub fn intersects(self, other: Self) -> bool {
        self.try_intersection(other).is_some()
    }

    /// Compute the union of the rectangle with the given rectangle.
    pub fn union(self, other: Self) -> Self {
        let min_x = f32::min(self.min.x, other.min.x);
        let min_y = f32::min(self.min.y, other.min.y);
        let max_x = f32::max(self.max.x, other.max.x);
        let max_y = f32::max(self.max.y, other.max.y);

        Self {
            min: Point::new(min_x, min_y),
            max: Point::new(max_x, max_y),
        }
    }

    /// Transform the rectangle by the given affine transform.
    pub fn transform(self, transform: Affine) -> Self {
        let tl = transform * self.top_left();
        let tr = transform * self.top_right();
        let bl = transform * self.bottom_left();
        let br = transform * self.bottom_right();

        let min_x = f32::min(f32::min(tl.x, tr.x), f32::min(bl.x, br.x));
        let min_y = f32::min(f32::min(tl.y, tr.y), f32::min(bl.y, br.y));

        let max_x = f32::max(f32::max(tl.x, tr.x), f32::max(bl.x, br.x));
        let max_y = f32::max(f32::max(tl.y, tr.y), f32::max(bl.y, br.y));

        Self {
            min: Point::new(min_x, min_y),
            max: Point::new(max_x, max_y),
        }
    }
}

impl From<Size> for Rect {
    fn from(size: Size) -> Self {
        Self::new(Point::ZERO, size.into())
    }
}

impl From<[f32; 4]> for Rect {
    fn from([min_x, min_y, max_x, max_y]: [f32; 4]) -> Self {
        Self {
            min: Point::new(min_x, min_y),
            max: Point::new(max_x, max_y),
        }
    }
}

impl From<Rect> for [f32; 4] {
    fn from(rect: Rect) -> Self {
        [rect.min.x, rect.min.y, rect.max.x, rect.max.y]
    }
}

impl Add<Vector> for Rect {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self::Output {
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

impl Sub<Vector> for Rect {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self::Output {
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

impl AddAssign<Vector> for Rect {
    fn add_assign(&mut self, rhs: Vector) {
        self.min += rhs;
        self.max += rhs;
    }
}

impl AddAssign<Size> for Rect {
    fn add_assign(&mut self, rhs: Size) {
        self.max += rhs;
    }
}

impl SubAssign<Vector> for Rect {
    fn sub_assign(&mut self, rhs: Vector) {
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
        self.intersection(rhs)
    }
}

impl BitAndAssign for Rect {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}
