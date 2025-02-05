use std::ops::{Mul, MulAssign};

use super::{Matrix, Point, Vector};

/// An affine transformation in 2 dimensional space.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Affine {
    /// The translation of the affine transformation.
    pub translation: Vector,

    /// The matrix of the affine transformation.
    pub matrix: Matrix,
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine {
    /// The identity transformation.
    pub const IDENTITY: Self = Self {
        translation: Vector::ZERO,
        matrix: Matrix::IDENTITY,
    };

    /// Crate a translation.
    pub const fn translate(translation: Vector) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Create a rotation.
    pub fn rotate(angle: f32) -> Self {
        Self {
            matrix: Matrix::from_angle(angle),
            ..Self::IDENTITY
        }
    }

    /// Create a scale.
    pub const fn scale(scale: Vector) -> Self {
        Self {
            matrix: Matrix::from_scale(scale),
            ..Self::IDENTITY
        }
    }

    /// Round the translation.
    pub fn round(self) -> Self {
        Self {
            translation: self.translation.round(),
            matrix: self.matrix,
        }
    }

    /// Compute the inverse transformation.
    pub fn inverse(self) -> Self {
        let matrix = self.matrix.inverse();
        let translation = matrix * -self.translation;

        Self {
            translation,
            matrix,
        }
    }
}

impl Mul<Point> for Affine {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        self.matrix * rhs + self.translation
    }
}

impl Mul<Vector> for Affine {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        self.matrix * rhs + self.translation
    }
}

impl Mul<Affine> for Affine {
    type Output = Self;

    fn mul(self, rhs: Affine) -> Self::Output {
        Self {
            translation: Point::to_vector(self * rhs.translation.to_point()),
            matrix: self.matrix * rhs.matrix,
        }
    }
}

impl MulAssign<Affine> for Affine {
    fn mul_assign(&mut self, rhs: Affine) {
        *self = *self * rhs;
    }
}
