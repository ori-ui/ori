use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use super::{Size, Vector};

/// A point in 2D space.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    /// The x coordinate.
    pub x: f32,
    /// The y coordinate.
    pub y: f32,
}

impl Point {
    /// The zero point.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// The one point.
    pub const ONE: Self = Self::new(1.0, 1.0);

    /// The unit x point.
    pub const X: Self = Self::new(1.0, 0.0);

    /// The unit y point.
    pub const Y: Self = Self::new(0.0, 1.0);

    /// The negative unit x point.
    pub const NEG_X: Self = Self::new(-1.0, 0.0);

    /// The negative unit y point.
    pub const NEG_Y: Self = Self::new(0.0, -1.0);

    /// Create a new point.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a new point with the same x and y.
    pub const fn all(value: f32) -> Self {
        Self::new(value, value)
    }

    /// Get the min of self and other by element.
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// Get the max of self and other by element.
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// Clamp self to the range [min, max] by element.
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    /// Floor the point by element.
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    /// Ceil the point by element.
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    /// Round the point by element.
    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    /// Get the fractional component by element.
    pub fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }

    /// Check if the point is finite.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Check if the point is infinite.
    pub fn is_infinite(self) -> bool {
        self.x.is_infinite() || self.y.is_infinite()
    }

    /// Check if the point is NaN.
    pub fn is_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    /// Compute the dot distance between two points.
    pub fn distance(self, other: Self) -> f32 {
        Vector::length(other - self)
    }

    /// Linearly interpolate between two points.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Convert the point to a vector.
    pub const fn to_vector(self) -> Vector {
        Vector::new(self.x, self.y)
    }

    /// Convert the point to a size.
    pub const fn to_size(self) -> Size {
        Size::new(self.x, self.y)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<Vector> for Point {
    fn from(vec: Vector) -> Self {
        vec.to_point()
    }
}

impl From<Size> for Point {
    fn from(size: Size) -> Self {
        size.to_point()
    }
}

impl From<f32> for Point {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

impl From<Point> for (f32, f32) {
    fn from(point: Point) -> Self {
        (point.x, point.y)
    }
}

impl From<Point> for [f32; 2] {
    fn from(point: Point) -> Self {
        [point.x, point.y]
    }
}

impl Neg for Point {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

macro_rules! impl_math_op {
    ($op_trait:ident, $op_assign_trait:ident, $op_fn:ident, $op_assign_fn:ident, $op:tt) => {
        impl $op_trait<Vector> for Point {
            type Output = Self;

            fn $op_fn(self, rhs: Vector) -> Self::Output {
                Self::new(self.x $op rhs.x, self.y $op rhs.y)
            }
        }

        impl $op_assign_trait<Vector> for Point {
            fn $op_assign_fn(&mut self, rhs: Vector) {
                *self = *self $op rhs;
            }
        }

        impl $op_trait<Size> for Point {
            type Output = Self;

            fn $op_fn(self, rhs: Size) -> Self::Output {
                Self::new(self.x $op rhs.width, self.y $op rhs.height)
            }
        }

        impl $op_assign_trait<Size> for Point {
            fn $op_assign_fn(&mut self, rhs: Size) {
                *self = *self $op rhs;
            }
        }

        impl $op_trait<f32> for Point {
            type Output = Self;

            fn $op_fn(self, rhs: f32) -> Self::Output {
                Self::new(self.x $op rhs, self.y $op rhs)
            }
        }

        impl $op_assign_trait<f32> for Point {
            fn $op_assign_fn(&mut self, rhs: f32) {
                *self = *self $op rhs;
            }
        }
    };
}

impl_math_op!(Add, AddAssign, add, add_assign, +);
impl_math_op!(Sub, SubAssign, sub, sub_assign, -);
impl_math_op!(Mul, MulAssign, mul, mul_assign, *);
impl_math_op!(Div, DivAssign, div, div_assign, /);
impl_math_op!(Rem, RemAssign, rem, rem_assign, %);

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}
