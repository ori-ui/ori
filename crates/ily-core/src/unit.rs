use std::{fmt::Display, ops::Range};

pub use Unit::*;

/// A unit of measurement. (eg. 10px, 10pt, 10%)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Unit {
    /// Unit of measurement in pixels. (eg. 10px)
    ///
    /// This is the default unit.
    Px(f32),
    /// Unit of measurement in points. (eg. 10pt)
    ///
    /// 1pt = 1/72 inch
    Pt(f32),
    /// Unit of measurement in percent. (eg. 10%)
    ///
    /// The percent is context specific, and is often relative
    /// to the parent's size, but doesn't have to be.
    Pc(f32),
    /// Unit of measurement in em. (eg. 10em)
    ///
    /// 1em = the font size of the root.
    /// 1em = 16px by default.
    Em(f32),
}

impl Default for Unit {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Unit {
    pub const ZERO: Self = Px(0.0);

    pub fn pixels(self, range: Range<f32>, scale: f32) -> f32 {
        match self {
            Px(value) => value,
            Pt(value) => value * 96.0 / 72.0 * scale,
            Pc(value) => value * (range.end - range.start) / 100.0,
            Em(value) => value * 16.0 * scale,
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Px(value) => write!(f, "{}px", value),
            Pt(value) => write!(f, "{}pt", value),
            Pc(value) => write!(f, "{}pc", value),
            Em(value) => write!(f, "{}em", value),
        }
    }
}
