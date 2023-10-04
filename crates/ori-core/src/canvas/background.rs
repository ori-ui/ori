use crate::image::{Image, Texture};

use super::Color;

/// A background.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Background {
    /// The image of the background.
    pub image: Option<Texture>,
    /// The color of the background.
    pub color: Color,
}

impl Background {
    /// Create a new [`Background`].
    pub fn new(background: impl Into<Background>) -> Self {
        background.into()
    }

    /// Create a new [`Background`] with an image.
    pub fn image(image: impl Into<Texture>) -> Self {
        Self {
            image: Some(image.into()),
            color: Color::WHITE,
        }
    }

    /// Create a new [`Background`] with a color.
    pub fn color(color: impl Into<Color>) -> Self {
        Self {
            image: None,
            color: color.into(),
        }
    }
}

impl From<Image> for Background {
    fn from(image: Image) -> Self {
        Self {
            image: Some(image.into()),
            color: Color::WHITE,
        }
    }
}

#[cfg(feature = "wgpu")]
impl From<crate::image::WgpuTexture> for Background {
    fn from(texture: crate::image::WgpuTexture) -> Self {
        Self {
            image: Some(texture.into()),
            color: Color::WHITE,
        }
    }
}

impl From<Texture> for Background {
    fn from(texture: Texture) -> Self {
        Self {
            image: Some(texture),
            color: Color::WHITE,
        }
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Self { image: None, color }
    }
}
