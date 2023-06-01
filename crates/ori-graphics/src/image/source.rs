use std::{fmt::Display, io};

use crate::ImageData;

/// An error that can occur when loading an image.
#[derive(Debug)]
pub enum ImageLoadError {
    /// An IO error occurred.
    Io(io::Error),
    /// An [`image::ImageError`] error occurred.
    #[cfg(feature = "image")]
    Image(image::ImageError),
}

impl From<io::Error> for ImageLoadError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(feature = "image")]
impl From<image::ImageError> for ImageLoadError {
    fn from(err: image::ImageError) -> Self {
        Self::Image(err)
    }
}

impl Display for ImageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            #[cfg(feature = "image")]
            Self::Image(err) => write!(f, "Image error: {}", err),
        }
    }
}

/// A source for an image.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImageSource {
    /// A path to an image.
    ///
    /// Requires the `image` feature.
    #[cfg(feature = "image")]
    Path(std::path::PathBuf),
    /// Image data.
    Data(ImageData),
}

impl Default for ImageSource {
    fn default() -> Self {
        Self::Data(ImageData::default())
    }
}

#[cfg(feature = "image")]
impl From<std::path::PathBuf> for ImageSource {
    fn from(path: std::path::PathBuf) -> Self {
        Self::Path(path)
    }
}

#[cfg(feature = "image")]
impl From<&std::path::Path> for ImageSource {
    fn from(path: &std::path::Path) -> Self {
        Self::Path(path.into())
    }
}

#[cfg(feature = "image")]
impl From<String> for ImageSource {
    fn from(path: String) -> Self {
        Self::Path(path.into())
    }
}

#[cfg(feature = "image")]
impl From<&str> for ImageSource {
    fn from(path: &str) -> Self {
        Self::Path(path.into())
    }
}

impl From<ImageData> for ImageSource {
    fn from(image: ImageData) -> Self {
        Self::Data(image)
    }
}

#[cfg(feature = "image")]
impl From<&[u8]> for ImageSource {
    fn from(bytes: &[u8]) -> Self {
        Self::Data(ImageData::from_bytes(bytes))
    }
}

impl ImageSource {
    /// Tries to load the [`ImageData`] from the source.
    pub fn try_load(self) -> Result<ImageData, ImageLoadError> {
        match self {
            Self::Data(image) => Ok(image),
            #[cfg(feature = "image")]
            Self::Path(path) => Ok(ImageData::try_load(path)?),
        }
    }

    /// Loads the [`ImageData`] from the source.
    ///
    /// # Panics
    /// - If loading the image fails.
    pub fn load(self) -> ImageData {
        match self {
            Self::Data(image) => image,
            #[cfg(feature = "image")]
            Self::Path(path) => ImageData::load(path),
        }
    }
}
