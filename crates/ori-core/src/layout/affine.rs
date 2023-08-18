use std::ops::{Mul, MulAssign};

use glam::{Mat2, Vec2};

/// An affine transformation in 2 dimensional space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    /// The translation of the affine transformation.
    pub translation: Vec2,
    /// The matrix of the affine transformation.
    pub matrix: Mat2,
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine {
    /// The identity transformation.
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        matrix: Mat2::IDENTITY,
    };

    /// Crate a translation.
    pub const fn translate(translation: Vec2) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Create a rotation.
    pub fn rotate(angle: f32) -> Self {
        Self {
            matrix: Mat2::from_angle(angle),
            ..Self::IDENTITY
        }
    }

    /// Create a scale.
    pub const fn scale(scale: Vec2) -> Self {
        Self {
            matrix: Mat2::from_diagonal(scale),
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

impl Mul<Vec2> for Affine {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        self.matrix * rhs + self.translation
    }
}

impl Mul<Affine> for Affine {
    type Output = Self;

    fn mul(self, rhs: Affine) -> Self::Output {
        Self {
            translation: self * rhs.translation,
            matrix: self.matrix * rhs.matrix,
        }
    }
}

impl MulAssign<Affine> for Affine {
    fn mul_assign(&mut self, rhs: Affine) {
        *self = *self * rhs;
    }
}
