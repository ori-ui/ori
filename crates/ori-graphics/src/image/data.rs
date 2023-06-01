use std::{fmt::Debug, sync::Arc};

use glam::Vec2;

/// Includes [`ImageData`] from a file.
#[macro_export]
macro_rules! include_image {
    ($($tt:tt)*) => {
        $crate::ImageData::from_bytes(include_bytes!($($tt)*))
    };
}

/// Image data, see [`ImageSource`] and [`ImageHandle`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ImageData {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl ImageData {
    /// Creates a new image.
    ///
    /// # Panics
    /// - If the length of `pixels` is not equal to `width * height * 4`.
    pub fn new(width: u32, height: u32, pixels: impl Into<Arc<[u8]>>) -> Self {
        let pixels = pixels.into();

        assert!(
            pixels.len() == (width * height * 4) as usize,
            "The length of `pixels` must be equal to `width * height * 4`"
        );

        Self {
            width,
            height,
            pixels,
        }
    }

    /// Tries to load an image from a path.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn try_load(path: impl AsRef<std::path::Path>) -> Result<Self, crate::ImageLoadError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();
        let pixels = image.into_rgba8().into_raw();

        Ok(Self {
            width,
            height,
            pixels: pixels.into(),
        })
    }

    /// Loads an image from a path.
    ///
    /// # Panics
    /// - If loading the image fails.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn load(path: impl AsRef<std::path::Path>) -> Self {
        match Self::try_load(path) {
            Ok(image) => image,
            Err(err) => {
                tracing::error!("Failed to load image: {}", err);
                Self::default()
            }
        }
    }

    /// Tries to load an image from bytes.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, crate::ImageLoadError> {
        let image = image::load_from_memory(bytes)?;
        let width = image.width();
        let height = image.height();
        let pixels = image.into_rgba8().into_raw();

        Ok(Self {
            width,
            height,
            pixels: pixels.into(),
        })
    }

    /// Loads an image from bytes.
    ///
    /// # Panics
    /// - If loading the image fails.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match Self::try_from_bytes(bytes) {
            Ok(image) => image,
            Err(err) => {
                tracing::error!("Failed to load image: {}", err);
                Self::default()
            }
        }
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the size of the image.
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// Returns the pixels of the image.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

impl Default for ImageData {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: vec![0, 0, 0, 0].into(),
        }
    }
}

impl Debug for ImageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageData")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pixels", &Arc::as_ptr(&self.pixels))
            .finish()
    }
}
