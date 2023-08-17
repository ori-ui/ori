use std::ops::{Mul, MulAssign};

use glam::{Mat2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub translation: Vec2,
    pub matrix: Mat2,
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine {
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        matrix: Mat2::IDENTITY,
    };

    pub const fn translate(translation: Vec2) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    pub fn rotate(angle: f32) -> Self {
        Self {
            matrix: Mat2::from_angle(angle),
            ..Self::IDENTITY
        }
    }

    pub const fn scale(scale: Vec2) -> Self {
        Self {
            matrix: Mat2::from_diagonal(scale),
            ..Self::IDENTITY
        }
    }

    pub fn round(self) -> Self {
        Self {
            translation: self.translation.round(),
            matrix: self.matrix,
        }
    }

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
