#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn major(&self, size: impl Into<(f32, f32)>) -> f32 {
        let (x, y) = size.into();
        match self {
            Axis::Horizontal => x,
            Axis::Vertical => y,
        }
    }

    pub fn minor(&self, size: impl Into<(f32, f32)>) -> f32 {
        let (x, y) = size.into();
        match self {
            Axis::Horizontal => y,
            Axis::Vertical => x,
        }
    }

    pub fn unpack(&self, size: impl Into<(f32, f32)>) -> (f32, f32) {
        let (x, y) = size.into();
        match self {
            Axis::Horizontal => (x, y),
            Axis::Vertical => (y, x),
        }
    }

    pub fn pack<T: From<(f32, f32)>>(&self, major: f32, minor: f32) -> T {
        match self {
            Axis::Horizontal => T::from((major, minor)),
            Axis::Vertical => T::from((minor, major)),
        }
    }
}
