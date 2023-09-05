use super::{Size, Vector};

/// Alignment of content inside a container.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Alignment {
    /// The horizontal alignment.
    pub x: f32,
    /// The vertical alignment.
    pub y: f32,
}

impl Alignment {
    /// Align the content at the center of the container.
    pub const CENTER: Self = Self::new(0.5, 0.5);
    /// Align the content at the top left of the container.
    pub const TOP_LEFT: Self = Self::new(0.0, 0.0);
    /// Align the content at the top of the container.
    pub const TOP: Self = Self::new(0.5, 0.0);
    /// Align the content at the top right of the container.
    pub const TOP_RIGHT: Self = Self::new(1.0, 0.0);
    /// Align the content at the left of the container.
    pub const LEFT: Self = Self::new(0.0, 0.5);
    /// Align the content at the right of the container.
    pub const RIGHT: Self = Self::new(1.0, 0.5);
    /// Align the content at the bottom left of the container.
    pub const BOTTOM_LEFT: Self = Self::new(0.0, 1.0);
    /// Align the content at the bottom of the container.
    pub const BOTTOM: Self = Self::new(0.5, 1.0);
    /// Align the content at the bottom right of the container.
    pub const BOTTOM_RIGHT: Self = Self::new(1.0, 1.0);

    /// Create a new alignment.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Align the content inside the container.
    pub fn align(self, content: Size, container: Size) -> Vector {
        Vector::new(
            self.x * (container.width - content.width),
            self.y * (container.height - content.height),
        )
    }
}

impl From<(f32, f32)> for Alignment {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<[f32; 2]> for Alignment {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}
