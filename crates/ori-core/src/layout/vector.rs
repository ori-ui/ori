use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use super::{Point, Size};

/// A 2D vector.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector {
    /// The x coordinate.
    pub x: f32,
    /// The y coordinate.
    pub y: f32,
}

impl Vector {
    /// The zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// The one vector.
    pub const ONE: Self = Self::new(1.0, 1.0);

    /// The unit x vector.
    pub const X: Self = Self::new(1.0, 0.0);

    /// The unit y vector.
    pub const Y: Self = Self::new(0.0, 1.0);

    /// The negative unit x vector.
    pub const NEG_X: Self = Self::new(-1.0, 0.0);

    /// The negative unit y vector.
    pub const NEG_Y: Self = Self::new(0.0, -1.0);

    /// Create a new vector.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Create a new vector with the same x and y.
    pub const fn all(value: f32) -> Self {
        Self::new(value, value)
    }

    /// Create a new vector from an angle.
    pub fn from_angle(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(sin, cos)
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

    /// Floor the vector by element.
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    /// Ceil the vector by element.
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    /// Round the vector by element.
    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    /// Get the absolute value of the vector.
    pub fn signum(self) -> Self {
        Self::new(self.x.signum(), self.y.signum())
    }

    /// Get the length of the vector squared.
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Get the length of the vector.
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Normalize the vector.
    ///
    /// If the length of the vector is zero, the zero vector is returned.
    pub fn normalize(self) -> Self {
        let length = self.length();

        if length == 0.0 {
            Self::ZERO
        } else {
            self / length
        }
    }

    /// Get the dot product of self and other.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Get the length of the cross product of self and other.
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Hat the vector.
    pub fn hat(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Get the angle between self and other.
    pub fn angle_between(self, other: Self) -> f32 {
        let dot = self.dot(other) / f32::sqrt(self.length_squared() * other.length_squared());
        dot.acos()
    }

    /// Convert the vector to a vector.
    pub const fn to_point(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Convert the vector to a size.
    pub const fn to_size(self) -> Size {
        Size::new(self.x, self.y)
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl From<(f32, f32)> for Vector {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Vector {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl From<Point> for Vector {
    fn from(point: Point) -> Self {
        point.to_vector()
    }
}

impl From<Size> for Vector {
    fn from(size: Size) -> Self {
        size.to_vector()
    }
}

impl From<f32> for Vector {
    fn from(value: f32) -> Self {
        Self::all(value)
    }
}

impl From<Vector> for (f32, f32) {
    fn from(vector: Vector) -> Self {
        (vector.x, vector.y)
    }
}

impl From<Vector> for [f32; 2] {
    fn from(vector: Vector) -> Self {
        [vector.x, vector.y]
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

macro_rules! impl_math_op {
    ($op_trait:ident, $op_assign_trait:ident, $op_fn:ident, $op_assign_fn:ident, $op:tt) => {
        impl $op_trait for Vector {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self::Output {
                Self::new(self.x $op rhs.x, self.y $op rhs.y)
            }
        }

        impl $op_assign_trait for Vector {
            fn $op_assign_fn(&mut self, rhs: Self) {
                *self = *self $op rhs;
            }
        }

        impl $op_trait<Vector> for Size {
            type Output = Self;

            fn $op_fn(self, rhs: Vector) -> Self::Output {
                Self::new(self.width $op rhs.x, self.width $op rhs.y)
            }
        }

        impl $op_assign_trait<Vector> for Size {
            fn $op_assign_fn(&mut self, rhs: Vector) {
                *self = *self $op rhs;
            }
        }

        impl $op_trait<f32> for Vector {
            type Output = Self;

            fn $op_fn(self, rhs: f32) -> Self::Output {
                Self::new(self.x $op rhs, self.y $op rhs)
            }
        }

        impl $op_assign_trait<f32> for Vector {
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

impl Hash for Vector {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}
