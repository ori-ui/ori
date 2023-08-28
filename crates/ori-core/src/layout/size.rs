use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use glam::Vec2;

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
    pub const fn splat(value: f32) -> Self {
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
    pub const fn to_vec(self) -> Vec2 {
        Vec2::new(self.width, self.height)
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

impl From<Vec2> for Size {
    fn from(vec: Vec2) -> Self {
        Self::new(vec.x, vec.y)
    }
}

impl From<f32> for Size {
    fn from(value: f32) -> Self {
        Self::splat(value)
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

impl From<Size> for Vec2 {
    fn from(size: Size) -> Self {
        size.to_vec()
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.width + rhs.width, self.height + rhs.height)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.width - rhs.width, self.height - rhs.height)
    }
}

impl SubAssign for Size {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<f32> for Size {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.width * rhs, self.height * rhs)
    }
}

impl MulAssign<f32> for Size {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Div<f32> for Size {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.width / rhs, self.height / rhs)
    }
}

impl DivAssign<f32> for Size {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}

impl Add<Size> for Vec2 {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Self::new(self.x + rhs.width, self.y + rhs.height)
    }
}

impl AddAssign<Size> for Vec2 {
    fn add_assign(&mut self, rhs: Size) {
        *self = *self + rhs;
    }
}

impl Sub<Size> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        Self::new(self.x - rhs.width, self.y - rhs.height)
    }
}

impl SubAssign<Size> for Vec2 {
    fn sub_assign(&mut self, rhs: Size) {
        *self = *self - rhs;
    }
}

impl Mul<Size> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Size) -> Self::Output {
        Self::new(self.x * rhs.width, self.y * rhs.height)
    }
}

impl MulAssign<Size> for Vec2 {
    fn mul_assign(&mut self, rhs: Size) {
        *self = *self * rhs;
    }
}

impl Div<Size> for Vec2 {
    type Output = Self;

    fn div(self, rhs: Size) -> Self::Output {
        Self::new(self.x / rhs.width, self.y / rhs.height)
    }
}

impl DivAssign<Size> for Vec2 {
    fn div_assign(&mut self, rhs: Size) {
        *self = *self / rhs;
    }
}
