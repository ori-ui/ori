use crate::Size;

/// Space available to lay out a view.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Space {
    /// Minimum size the view can be.
    pub min: Size,
    /// Maximum size the view can be.
    pub max: Size,
}

impl Space {
    /// Create a new space.
    pub fn new(min: Size, max: Size) -> Self {
        Self {
            min: min.max(Size::ZERO),
            max: max.max(Size::ZERO),
        }
    }

    /// Shrink the space by `size`.
    pub fn shrink(&self, size: Size) -> Self {
        Self::new(self.min - size, self.max - size)
    }

    /// Clamp a size to the space.
    pub fn fit(&self, size: Size) -> Size {
        size.clamp(self.min, self.max)
    }

    pub fn fit_container(&self, content: Size, size: Size) -> Size {
        let width = if size.height.is_infinite() {
            if self.max.height.is_infinite() {
                content.width.max(self.min.width)
            } else {
                self.max.width
            }
        } else {
            let width = size.width.max(content.width);
            width.clamp(self.min.width, self.max.width)
        };

        let height = if size.height.is_infinite() {
            if self.max.height.is_infinite() {
                content.height.max(self.min.height)
            } else {
                self.max.height
            }
        } else {
            let height = size.height.max(content.height);
            height.clamp(self.min.height, self.max.height)
        };

        if width.is_infinite() {
            tracing::warn!("width is infinite");
        }

        if height.is_infinite() {
            tracing::warn!("height is infinite");
        }

        Size::new(width, height)
    }
}
