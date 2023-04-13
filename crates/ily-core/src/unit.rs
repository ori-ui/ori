use std::ops::Range;

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
