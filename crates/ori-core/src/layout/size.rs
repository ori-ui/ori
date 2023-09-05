use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign},
};

use super::{Point, Vector};

/// A 2 dimensional size.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Size {
    /// The width.
    pub width: f32,
    /// The height.
    pub height: f32,
}

impl Size {
    /// The zero size.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// The unbounded size.
    pub const UNBOUNDED: Self = Self::new(f32::INFINITY, f32::INFINITY);

    /// The infinite size.
    pub const INFINITY: Self = Self::new(f32::INFINITY, f32::INFINITY);

    /// Alias for [`Self::INFINITY`].
    pub const FILL: Self = Self::INFINITY;

    /// Create a new size.
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a new size with the same width and height.
    pub const fn all(value: f32) -> Self {
        Self::new(value, value)
    }

    /// Get the min of self and other by element.
    pub fn min(self, other: Self) -> Self {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }

    /// Get the max of self and other by element.
    pub fn max(self, other: Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    /// Clamp self to the range [min, max] by element.
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            self.width.clamp(min.width, max.width),
            self.height.clamp(min.height, max.height),
        )
    }

    /// Convert the size to a vector.
    pub const fn to_point(self) -> Point {
        Point::new(self.width, self.height)
    }

    /// Convert the size to a vector.
    pub const fn to_vector(self) -> Vector {
        Vector::new(self.width, self.height)
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Self::new(width, height)
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Self::new(width, height)
    }
}

impl From<Point> for Size {
    fn from(vec: Point) -> Self {
        Self::new(vec.x, vec.y)
    }
}

impl From<Vector> for Size {
    fn from(vec: Vector) -> Self {
        vec.to_size()
    }
}

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

impl From<Size> for (f32, f32) {
    fn from(size: Size) -> Self {
        (size.width, size.height)
    }
}

impl From<Size> for [f32; 2] {
    fn from(size: Size) -> Self {
        [size.width, size.height]
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

macro_rules! impl_math_op {
    ($op_trait:ident, $op_assign_trait:ident, $op_fn:ident, $op_assign_fn:ident, $op:tt) => {
        impl $op_trait for Size {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self::Output {
                Self::new(self.width $op rhs.width, self.height $op rhs.height)
            }
        }

        impl $op_assign_trait for Size {
            fn $op_assign_fn(&mut self, rhs: Self) {
                *self = *self $op rhs;
            }
        }

        impl $op_trait<f32> for Size {
            type Output = Self;

            fn $op_fn(self, rhs: f32) -> Self::Output {
                Self::new(self.width $op rhs, self.height $op rhs)
            }
        }

        impl $op_assign_trait<f32> for Size {
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
