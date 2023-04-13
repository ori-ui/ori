use std::{fmt::Display, ops::Range};

pub use Unit::*;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Unit {
    Px(f32),
    Pt(f32),
    Pc(f32),
}

impl Default for Unit {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Unit {
    pub const ZERO: Self = Px(0.0);

    pub fn pixels(self, range: Range<f32>) -> f32 {
        match self {
            Px(value) => value,
            Pt(value) => value * 96.0 / 72.0,
            Pc(value) => value * (range.end - range.start) / 100.0,
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Px(value) => write!(f, "{}px", value),
            Pt(value) => write!(f, "{}pt", value),
            Pc(value) => write!(f, "{}pc", value),
        }
    }
}
