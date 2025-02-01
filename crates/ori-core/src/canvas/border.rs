use crate::style::Styled;

/// Radi of the corners on a rounded rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorderRadius {
    /// The top left corner radius.
    pub top_left: f32,
    /// The top right corner radius.
    pub top_right: f32,
    /// The bottom right corner radius.
    pub bottom_right: f32,
    /// The bottom left corner radius.
    pub bottom_left: f32,
}

impl BorderRadius {
    /// A [`BorderRadius`] with zero radius on all corners.
    pub const ZERO: Self = Self::all(0.0);

    /// Create a new [`BorderRadius`].
    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    /// Create a new [`BorderRadius`] with the same radius on all corners.
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Get the maximum radius of the corners.
    pub fn max_element(&self) -> f32 {
        self.top_left
            .max(self.top_right)
            .max(self.bottom_right)
            .max(self.bottom_left)
    }

    /// Get the minimum radius of the corners.
    pub fn min_element(&self) -> f32 {
        self.top_left
            .min(self.top_right)
            .min(self.bottom_right)
            .min(self.bottom_left)
    }

    /// Expand the radius of the corners.
    pub fn expand(&self, radius: f32) -> Self {
        Self {
            top_left: self.top_left + radius,
            top_right: self.top_right + radius,
            bottom_right: self.bottom_right + radius,
            bottom_left: self.bottom_left + radius,
        }
    }
}

impl From<(f32, f32, f32, f32)> for BorderRadius {
    fn from((top_left, top_right, bottom_right, bottom_left): (f32, f32, f32, f32)) -> Self {
        Self::new(top_left, top_right, bottom_right, bottom_left)
    }
}

impl From<[f32; 4]> for BorderRadius {
    fn from([top_left, top_right, bottom_right, bottom_left]: [f32; 4]) -> Self {
        Self::new(top_left, top_right, bottom_right, bottom_left)
    }
}

impl From<f32> for BorderRadius {
    fn from(radius: f32) -> Self {
        Self::all(radius)
    }
}

impl From<BorderRadius> for [f32; 4] {
    fn from(radius: BorderRadius) -> Self {
        [
            radius.top_left,
            radius.top_right,
            radius.bottom_right,
            radius.bottom_left,
        ]
    }
}

impl From<(f32, f32, f32, f32)> for Styled<BorderRadius> {
    fn from(x: (f32, f32, f32, f32)) -> Self {
        Self::value(BorderRadius::from(x))
    }
}

impl From<[f32; 4]> for Styled<BorderRadius> {
    fn from(x: [f32; 4]) -> Self {
        Self::value(BorderRadius::from(x))
    }
}

impl From<f32> for Styled<BorderRadius> {
    fn from(x: f32) -> Self {
        Self::value(BorderRadius::from(x))
    }
}

/// The border width of a rounded rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorderWidth {
    /// The top border width.
    pub top: f32,
    /// The right border width.
    pub right: f32,
    /// The bottom border width.
    pub bottom: f32,
    /// The left border width.
    pub left: f32,
}

impl BorderWidth {
    /// A [`BorderWidth`] with zero width on all borders.
    pub const ZERO: Self = Self::all(0.0);

    /// Create a new [`BorderWidth`].
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Create a new [`BorderWidth`] with the same width on all borders.
    pub const fn all(width: f32) -> Self {
        Self {
            top: width,
            right: width,
            bottom: width,
            left: width,
        }
    }

    /// Get the maximum width of the borders.
    pub fn max_element(&self) -> f32 {
        self.top.max(self.right).max(self.bottom).max(self.left)
    }

    /// Get the minimum width of the borders.
    pub fn min_element(&self) -> f32 {
        self.top.min(self.right).min(self.bottom).min(self.left)
    }

    /// Expand the width of the borders.
    pub fn expand(&self, width: f32) -> Self {
        Self {
            top: self.top + width,
            right: self.right + width,
            bottom: self.bottom + width,
            left: self.left + width,
        }
    }
}

impl From<(f32, f32, f32, f32)> for BorderWidth {
    fn from((top, right, bottom, left): (f32, f32, f32, f32)) -> Self {
        Self::new(top, right, bottom, left)
    }
}

impl From<[f32; 4]> for BorderWidth {
    fn from([top, right, bottom, left]: [f32; 4]) -> Self {
        Self::new(top, right, bottom, left)
    }
}

impl From<(f32, f32)> for BorderWidth {
    fn from((horizontal, vertical): (f32, f32)) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }
}

impl From<[f32; 2]> for BorderWidth {
    fn from([horizontal, vertical]: [f32; 2]) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }
}

impl From<f32> for BorderWidth {
    fn from(width: f32) -> Self {
        Self::all(width)
    }
}

impl From<BorderWidth> for [f32; 4] {
    fn from(width: BorderWidth) -> Self {
        [width.top, width.right, width.bottom, width.left]
    }
}

impl From<(f32, f32, f32, f32)> for Styled<BorderWidth> {
    fn from(x: (f32, f32, f32, f32)) -> Self {
        Self::value(BorderWidth::from(x))
    }
}

impl From<[f32; 4]> for Styled<BorderWidth> {
    fn from(x: [f32; 4]) -> Self {
        Self::value(BorderWidth::from(x))
    }
}

impl From<(f32, f32)> for Styled<BorderWidth> {
    fn from(x: (f32, f32)) -> Self {
        Self::value(BorderWidth::from(x))
    }
}

impl From<[f32; 2]> for Styled<BorderWidth> {
    fn from(x: [f32; 2]) -> Self {
        Self::value(BorderWidth::from(x))
    }
}

impl From<f32> for Styled<BorderWidth> {
    fn from(x: f32) -> Self {
        Self::value(BorderWidth::from(x))
    }
}
