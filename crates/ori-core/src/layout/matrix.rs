use std::ops::Mul;

use super::{Point, Vector};

/// A 2x2 matrix.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Hash)]
pub struct Matrix {
    /// The x axis of the matrix.
    pub x: Vector,
    /// The y axis of the matrix.
    pub y: Vector,
}

impl Matrix {
    /// The identity matrix.
    pub const IDENTITY: Self = Self::from_scale(Vector::all(1.0));

    /// Create a new matrix.
    pub const fn new(x: Vector, y: Vector) -> Self {
        Self { x, y }
    }

    /// Create a new matrix from an angle.
    pub fn from_angle(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();

        Self::new(Vector::new(cos, sin), Vector::new(-sin, cos))
    }

    /// Create a new matrix from a scale.
    pub const fn from_scale(scale: Vector) -> Self {
        Self::new(Vector::new(scale.x, 0.0), Vector::new(0.0, scale.y))
    }

    /// Compute the determinant of the matrix.
    pub fn determinant(self) -> f32 {
        self.x.x * self.y.y - self.x.y * self.y.x
    }

    /// Compute the inverse of the matrix.
    pub fn inverse(self) -> Self {
        let det = self.determinant();

        if det == 0.0 {
            return Self::IDENTITY;
        }

        let inv_det = 1.0 / det;

        Self::new(
            Vector::new(self.y.y * inv_det, -self.x.y * inv_det),
            Vector::new(-self.y.x * inv_det, self.x.x * inv_det),
        )
    }
}

impl From<Matrix> for [f32; 4] {
    fn from(matrix: Matrix) -> Self {
        [matrix.x.x, matrix.x.y, matrix.y.x, matrix.y.y]
    }
}

impl Mul<Vector> for Matrix {
    type Output = Vector;

    fn mul(self, point: Vector) -> Self::Output {
        Vector::new(
            self.x.x * point.x + self.y.x * point.y,
            self.x.y * point.x + self.y.y * point.y,
        )
    }
}

impl Mul<Point> for Matrix {
    type Output = Point;

    fn mul(self, point: Point) -> Self::Output {
        Vector::to_point(self * point.to_vector())
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self::new(self * other.x, self * other.y)
    }
}
