mod state;
mod states;

pub use state::*;
pub use states::*;

use std::ops::{Add, Mul};

/// A style transition.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleTransition {
    pub duration: f32,
}

impl StyleTransition {
    /// Create a new style transition.
    pub const fn new(duration: f32) -> Self {
        Self { duration }
    }

    /// Create a new instant style transition.
    pub const fn instant() -> Self {
        Self::new(0.0)
    }
}

impl From<f32> for StyleTransition {
    fn from(duration: f32) -> Self {
        Self::new(duration)
    }
}

/// A value that can be transitioned.
pub trait Transitionable: Mul<f32, Output = Self> + Add<Output = Self> + PartialEq + Copy {}
impl<T: Mul<f32, Output = T> + Add<Output = T> + PartialEq + Copy> Transitionable for T {}
