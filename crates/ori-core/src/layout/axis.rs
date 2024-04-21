/// An axis is a direction in which a layout is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Axis {
    /// The horizontal axis.
    Horizontal,
    /// The vertical axis.
    Vertical,
}

impl Axis {
    /// Get the major component of a pair.
    pub fn major(&self, size: impl Into<(f32, f32)>) -> f32 {
        let (x, y) = size.into();
        match self {
            Axis::Horizontal => x,
            Axis::Vertical => y,
        }
    }

    /// Get the minor component of a pair.
    pub fn minor(&self, size: impl Into<(f32, f32)>) -> f32 {
        let (x, y) = size.into();
        match self {
            Axis::Horizontal => y,
            Axis::Vertical => x,
        }
    }

    /// Unpack a pair into it's (major, minor) components.
    pub fn unpack(&self, size: impl Into<(f32, f32)>) -> (f32, f32) {
        let (x, y) = size.into();
        match self {
            Axis::Horizontal => (x, y),
            Axis::Vertical => (y, x),
        }
    }

    /// Pack a major and minor component into a pair.
    pub fn pack<T: From<(f32, f32)>>(&self, major: f32, minor: f32) -> T {
        match self {
            Axis::Horizontal => T::from((major, minor)),
            Axis::Vertical => T::from((minor, major)),
        }
    }
}
